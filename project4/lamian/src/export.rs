use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use rusqlite::Connection;
use serde::Serialize;

use crate::cli::ExportFormat;
use crate::db;
use crate::error::LamianError;

#[derive(Debug, Clone)]
pub struct ExportRequest {
    pub vault_root: PathBuf,
    pub format: ExportFormat,
    pub target: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportResult {
    pub target_path: Option<PathBuf>,
    pub figure_count: usize,
    pub output: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExportDocument {
    schema_version: i64,
    figures: Vec<ExportFigure>,
}

#[derive(Debug, Serialize)]
struct ExportFigure {
    figure_id: String,
    display_name: String,
    caption: Option<String>,
    file_path: String,
    file_hash_sha256: String,
    media_type: String,
    file_size_bytes: u64,
    created_at: String,
    updated_at: String,
    sources: Vec<ExportSource>,
    tags: Vec<String>,
    outbound_links: Vec<ExportLink>,
    note: Option<ExportNote>,
}

#[derive(Debug, Serialize)]
struct ExportSource {
    source_type: String,
    source_key: String,
    source_title: Option<String>,
    source_authors: Option<String>,
    source_published_at: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct ExportLink {
    to_figure_id: String,
    relation_type: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct ExportNote {
    note_markdown: String,
    updated_at: String,
}

#[derive(Debug)]
struct ExportFigureBase {
    figure_id: String,
    display_name: String,
    caption: Option<String>,
    file_path: String,
    file_hash_sha256: String,
    media_type: String,
    file_size_bytes: u64,
    created_at: String,
    updated_at: String,
}

pub fn export_metadata(request: ExportRequest) -> Result<ExportResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let target_path = normalize_target_path(request.target)?;

    let mut connection = db::open_vault_connection(&request.vault_root)?;

    let export_document = load_export_document(&mut connection)?;
    let figure_count = export_document.figures.len();
    let serialized_output = serialize_export(&export_document, request.format)?;

    if let Some(path) = target_path.as_ref() {
        fs::write(path, serialized_output.as_bytes())?;
        return Ok(ExportResult {
            target_path,
            figure_count,
            output: None,
        });
    }

    Ok(ExportResult {
        target_path: None,
        figure_count,
        output: Some(serialized_output),
    })
}

fn normalize_target_path(target: Option<PathBuf>) -> Result<Option<PathBuf>, LamianError> {
    match target {
        None => Ok(None),
        Some(path) => {
            if path.as_os_str().is_empty() {
                return Err(LamianError::MissingExportField { field: "target" });
            }

            if path.exists() && path.is_dir() {
                return Err(LamianError::InvalidExportValue {
                    field: "target",
                    reason: "target path must point to a file",
                    value: path.display().to_string(),
                });
            }

            if let Some(parent_directory) = path.parent() {
                if !parent_directory.as_os_str().is_empty() {
                    fs::create_dir_all(parent_directory)?;
                }
            }

            Ok(Some(path))
        }
    }
}

fn serialize_export(
    export_document: &ExportDocument,
    format: ExportFormat,
) -> Result<String, LamianError> {
    let mut content = match format {
        ExportFormat::Json => serde_json::to_string_pretty(export_document).map_err(|error| {
            LamianError::ExportSerializationFailed {
                format: "json",
                reason: error.to_string(),
            }
        })?,
        ExportFormat::Yaml => serde_yaml::to_string(export_document).map_err(|error| {
            LamianError::ExportSerializationFailed {
                format: "yaml",
                reason: error.to_string(),
            }
        })?,
    };

    if !content.ends_with('\n') {
        content.push('\n');
    }

    Ok(content)
}

fn load_export_document(connection: &mut Connection) -> Result<ExportDocument, LamianError> {
    let schema_version = latest_schema_version(connection)?;
    let figures = load_export_figures(connection)?;

    Ok(ExportDocument {
        schema_version,
        figures,
    })
}

fn latest_schema_version(connection: &Connection) -> Result<i64, LamianError> {
    let version = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;
    Ok(version)
}

fn load_export_figures(connection: &mut Connection) -> Result<Vec<ExportFigure>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT
    figure_id,
    display_name,
    caption,
    file_path,
    file_hash_sha256,
    media_type,
    file_size_bytes,
    created_at,
    updated_at
FROM figures
ORDER BY figure_id ASC
"#,
    )?;
    let mut rows = statement.query([])?;
    let mut figure_rows = Vec::new();

    while let Some(row) = rows.next()? {
        let figure_id: String = row.get(0)?;
        let file_size_bytes_i64: i64 = row.get(6)?;

        let file_size_bytes =
            u64::try_from(file_size_bytes_i64).map_err(|_| LamianError::InvalidExportValue {
                field: "file_size_bytes",
                reason: "file size in database cannot be negative",
                value: file_size_bytes_i64.to_string(),
            })?;

        figure_rows.push(ExportFigureBase {
            figure_id,
            display_name: row.get(1)?,
            caption: row.get(2)?,
            file_path: row.get(3)?,
            file_hash_sha256: row.get(4)?,
            media_type: row.get(5)?,
            file_size_bytes,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        });
    }

    if figure_rows.is_empty() {
        return Ok(Vec::new());
    }

    let mut sources_by_figure_id = load_sources_by_figure(connection)?;
    let mut tags_by_figure_id = load_tags_by_figure(connection)?;
    let mut outbound_links_by_figure_id = load_outbound_links_by_figure(connection)?;
    let mut notes_by_figure_id = load_notes_by_figure(connection)?;

    let mut figures = Vec::with_capacity(figure_rows.len());
    for figure_row in figure_rows {
        figures.push(ExportFigure {
            sources: sources_by_figure_id
                .remove(&figure_row.figure_id)
                .unwrap_or_default(),
            tags: tags_by_figure_id
                .remove(&figure_row.figure_id)
                .unwrap_or_default(),
            outbound_links: outbound_links_by_figure_id
                .remove(&figure_row.figure_id)
                .unwrap_or_default(),
            note: notes_by_figure_id.remove(&figure_row.figure_id),
            figure_id: figure_row.figure_id,
            display_name: figure_row.display_name,
            caption: figure_row.caption,
            file_path: figure_row.file_path,
            file_hash_sha256: figure_row.file_hash_sha256,
            media_type: figure_row.media_type,
            file_size_bytes: figure_row.file_size_bytes,
            created_at: figure_row.created_at,
            updated_at: figure_row.updated_at,
        });
    }

    Ok(figures)
}

fn load_sources_by_figure(
    connection: &Connection,
) -> Result<HashMap<String, Vec<ExportSource>>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT
    figure_id,
    source_type,
    source_key,
    source_title,
    source_authors,
    source_published_at,
    created_at
FROM sources
ORDER BY figure_id ASC, source_id ASC
"#,
    )?;
    let mut rows = statement.query([])?;
    let mut sources_by_figure_id = HashMap::new();

    while let Some(row) = rows.next()? {
        let figure_id: String = row.get(0)?;
        let source = ExportSource {
            source_type: row.get(1)?,
            source_key: row.get(2)?,
            source_title: row.get(3)?,
            source_authors: row.get(4)?,
            source_published_at: row.get(5)?,
            created_at: row.get(6)?,
        };
        sources_by_figure_id
            .entry(figure_id)
            .or_insert_with(Vec::new)
            .push(source);
    }

    Ok(sources_by_figure_id)
}

fn load_tags_by_figure(
    connection: &Connection,
) -> Result<HashMap<String, Vec<String>>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT figure_tags.figure_id, tags.tag_name
FROM figure_tags
JOIN tags ON tags.tag_id = figure_tags.tag_id
ORDER BY figure_tags.figure_id ASC, tags.tag_name ASC
"#,
    )?;
    let mut rows = statement.query([])?;
    let mut tags_by_figure_id = HashMap::new();

    while let Some(row) = rows.next()? {
        let figure_id: String = row.get(0)?;
        let tag_name: String = row.get(1)?;
        tags_by_figure_id
            .entry(figure_id)
            .or_insert_with(Vec::new)
            .push(tag_name);
    }

    Ok(tags_by_figure_id)
}

fn load_outbound_links_by_figure(
    connection: &Connection,
) -> Result<HashMap<String, Vec<ExportLink>>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT from_figure_id, to_figure_id, relation_type, created_at
FROM links
ORDER BY from_figure_id ASC, to_figure_id ASC, relation_type ASC, created_at ASC
"#,
    )?;
    let mut rows = statement.query([])?;
    let mut outbound_links_by_figure_id = HashMap::new();

    while let Some(row) = rows.next()? {
        let from_figure_id: String = row.get(0)?;
        let outbound_link = ExportLink {
            to_figure_id: row.get(1)?,
            relation_type: row.get(2)?,
            created_at: row.get(3)?,
        };
        outbound_links_by_figure_id
            .entry(from_figure_id)
            .or_insert_with(Vec::new)
            .push(outbound_link);
    }

    Ok(outbound_links_by_figure_id)
}

fn load_notes_by_figure(
    connection: &Connection,
) -> Result<HashMap<String, ExportNote>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT figure_id, note_markdown, updated_at
FROM notes
ORDER BY figure_id ASC
"#,
    )?;
    let mut rows = statement.query([])?;
    let mut notes_by_figure_id = HashMap::new();

    while let Some(row) = rows.next()? {
        let figure_id: String = row.get(0)?;
        let note = ExportNote {
            note_markdown: row.get(1)?,
            updated_at: row.get(2)?,
        };
        notes_by_figure_id.insert(figure_id, note);
    }

    Ok(notes_by_figure_id)
}

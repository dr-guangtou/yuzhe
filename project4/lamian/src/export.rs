use std::fs;
use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension};
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

#[derive(Debug, Clone)]
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

pub fn export_metadata(request: ExportRequest) -> Result<ExportResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let target_path = normalize_target_path(request.target)?;

    let vault_paths = db::resolve_vault_paths(&request.vault_root);
    if !vault_paths.database_path.exists() {
        return Err(LamianError::VaultNotInitialized {
            vault_root: request.vault_root,
        });
    }

    let mut connection = Connection::open(vault_paths.database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

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

            if let Some(parent_directory) = path.parent()
                && !parent_directory.as_os_str().is_empty()
            {
                fs::create_dir_all(parent_directory)?;
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
    let mut figures = Vec::new();

    while let Some(row) = rows.next()? {
        let figure_id: String = row.get(0)?;
        let file_size_bytes_i64: i64 = row.get(6)?;

        let file_size_bytes =
            u64::try_from(file_size_bytes_i64).map_err(|_| LamianError::InvalidExportValue {
                field: "file_size_bytes",
                reason: "file size in database cannot be negative",
                value: file_size_bytes_i64.to_string(),
            })?;

        figures.push(ExportFigure {
            figure_id: figure_id.clone(),
            display_name: row.get(1)?,
            caption: row.get(2)?,
            file_path: row.get(3)?,
            file_hash_sha256: row.get(4)?,
            media_type: row.get(5)?,
            file_size_bytes,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            sources: load_sources_for_figure(connection, &figure_id)?,
            tags: load_tags_for_figure(connection, &figure_id)?,
            outbound_links: load_links_for_figure(connection, &figure_id)?,
            note: load_note_for_figure(connection, &figure_id)?,
        });
    }

    Ok(figures)
}

fn load_sources_for_figure(
    connection: &Connection,
    figure_id: &str,
) -> Result<Vec<ExportSource>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT
    source_type,
    source_key,
    source_title,
    source_authors,
    source_published_at,
    created_at
FROM sources
WHERE figure_id = ?1
ORDER BY source_id ASC
"#,
    )?;
    let mut rows = statement.query([figure_id])?;
    let mut sources = Vec::new();

    while let Some(row) = rows.next()? {
        sources.push(ExportSource {
            source_type: row.get(0)?,
            source_key: row.get(1)?,
            source_title: row.get(2)?,
            source_authors: row.get(3)?,
            source_published_at: row.get(4)?,
            created_at: row.get(5)?,
        });
    }

    Ok(sources)
}

fn load_tags_for_figure(
    connection: &Connection,
    figure_id: &str,
) -> Result<Vec<String>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT tags.tag_name
FROM figure_tags
JOIN tags ON tags.tag_id = figure_tags.tag_id
WHERE figure_tags.figure_id = ?1
ORDER BY tags.tag_name ASC
"#,
    )?;
    let mut rows = statement.query([figure_id])?;
    let mut tags = Vec::new();

    while let Some(row) = rows.next()? {
        tags.push(row.get(0)?);
    }

    Ok(tags)
}

fn load_links_for_figure(
    connection: &Connection,
    figure_id: &str,
) -> Result<Vec<ExportLink>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT to_figure_id, relation_type, created_at
FROM links
WHERE from_figure_id = ?1
ORDER BY to_figure_id ASC, relation_type ASC, created_at ASC
"#,
    )?;
    let mut rows = statement.query([figure_id])?;
    let mut links = Vec::new();

    while let Some(row) = rows.next()? {
        links.push(ExportLink {
            to_figure_id: row.get(0)?,
            relation_type: row.get(1)?,
            created_at: row.get(2)?,
        });
    }

    Ok(links)
}

fn load_note_for_figure(
    connection: &Connection,
    figure_id: &str,
) -> Result<Option<ExportNote>, LamianError> {
    let note = connection
        .query_row(
            "SELECT note_markdown, updated_at FROM notes WHERE figure_id = ?1",
            [figure_id],
            |row| {
                Ok(ExportNote {
                    note_markdown: row.get(0)?,
                    updated_at: row.get(1)?,
                })
            },
        )
        .optional()?;

    Ok(note)
}

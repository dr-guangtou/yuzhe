use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension};

use crate::db;
use crate::error::LamianError;

#[derive(Debug, Clone)]
pub struct ShowFigureRequest {
    pub vault_root: PathBuf,
    pub figure_id: String,
}

#[derive(Debug, Clone)]
pub struct ShowFigureResult {
    pub figure_id: String,
    pub display_name: String,
    pub caption: Option<String>,
    pub file_path: String,
    pub file_hash_sha256: String,
    pub media_type: String,
    pub file_size_bytes: u64,
    pub created_at: String,
    pub updated_at: String,
    pub sources: Vec<ShowFigureSource>,
    pub tags: Vec<String>,
    pub outbound_links: Vec<ShowFigureLink>,
    pub note: Option<ShowFigureNote>,
}

#[derive(Debug, Clone)]
pub struct ShowFigureSource {
    pub source_type: String,
    pub source_key: String,
    pub source_title: Option<String>,
    pub source_authors: Option<String>,
    pub source_published_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct ShowFigureLink {
    pub to_figure_id: String,
    pub relation_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct ShowFigureNote {
    pub note_markdown: String,
    pub updated_at: String,
}

#[derive(Debug)]
struct ShowFigureBaseRow {
    figure_id: String,
    display_name: String,
    caption: Option<String>,
    file_path: String,
    file_hash_sha256: String,
    media_type: String,
    file_size_bytes_i64: i64,
    created_at: String,
    updated_at: String,
}

pub fn show_figure(request: ShowFigureRequest) -> Result<ShowFigureResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let normalized_figure_id = normalize_figure_id(&request.figure_id)?;
    let connection = db::open_vault_connection(&request.vault_root)?;

    let mut result = load_figure_base(&connection, &normalized_figure_id)?;
    result.sources = load_figure_sources(&connection, &normalized_figure_id)?;
    result.tags = load_figure_tags(&connection, &normalized_figure_id)?;
    result.outbound_links = load_outbound_links(&connection, &normalized_figure_id)?;
    result.note = load_figure_note(&connection, &normalized_figure_id)?;

    Ok(result)
}

fn normalize_figure_id(figure_id: &str) -> Result<String, LamianError> {
    let normalized_figure_id = figure_id.trim();
    if normalized_figure_id.is_empty() {
        return Err(LamianError::MissingShowField { field: "figure_id" });
    }
    Ok(normalized_figure_id.to_string())
}

fn load_figure_base(
    connection: &Connection,
    figure_id: &str,
) -> Result<ShowFigureResult, LamianError> {
    let base_row: Option<ShowFigureBaseRow> = connection
        .query_row(
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
WHERE figure_id = ?1
"#,
            [figure_id],
            |row| {
                Ok(ShowFigureBaseRow {
                    figure_id: row.get(0)?,
                    display_name: row.get(1)?,
                    caption: row.get(2)?,
                    file_path: row.get(3)?,
                    file_hash_sha256: row.get(4)?,
                    media_type: row.get(5)?,
                    file_size_bytes_i64: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .optional()?;

    let base_row = base_row.ok_or_else(|| LamianError::UnknownFigureId {
        figure_id: figure_id.to_string(),
    })?;

    let file_size_bytes =
        u64::try_from(base_row.file_size_bytes_i64).map_err(|_| LamianError::InvalidShowValue {
            field: "file_size_bytes",
            reason: "file size in database cannot be negative",
            value: base_row.file_size_bytes_i64.to_string(),
        })?;

    Ok(ShowFigureResult {
        figure_id: base_row.figure_id,
        display_name: base_row.display_name,
        caption: base_row.caption,
        file_path: base_row.file_path,
        file_hash_sha256: base_row.file_hash_sha256,
        media_type: base_row.media_type,
        file_size_bytes,
        created_at: base_row.created_at,
        updated_at: base_row.updated_at,
        sources: Vec::new(),
        tags: Vec::new(),
        outbound_links: Vec::new(),
        note: None,
    })
}

fn load_figure_sources(
    connection: &Connection,
    figure_id: &str,
) -> Result<Vec<ShowFigureSource>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT source_type, source_key, source_title, source_authors, source_published_at, created_at
FROM sources
WHERE figure_id = ?1
ORDER BY source_id ASC
"#,
    )?;

    let mut rows = statement.query([figure_id])?;
    let mut sources = Vec::new();
    while let Some(row) = rows.next()? {
        sources.push(ShowFigureSource {
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

fn load_figure_tags(connection: &Connection, figure_id: &str) -> Result<Vec<String>, LamianError> {
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

fn load_outbound_links(
    connection: &Connection,
    figure_id: &str,
) -> Result<Vec<ShowFigureLink>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT to_figure_id, relation_type, created_at
FROM links
WHERE from_figure_id = ?1
ORDER BY to_figure_id ASC, relation_type ASC, link_id ASC
"#,
    )?;
    let mut rows = statement.query(params![figure_id])?;

    let mut outbound_links = Vec::new();
    while let Some(row) = rows.next()? {
        outbound_links.push(ShowFigureLink {
            to_figure_id: row.get(0)?,
            relation_type: row.get(1)?,
            created_at: row.get(2)?,
        });
    }

    Ok(outbound_links)
}

fn load_figure_note(
    connection: &Connection,
    figure_id: &str,
) -> Result<Option<ShowFigureNote>, LamianError> {
    connection
        .query_row(
            "SELECT note_markdown, updated_at FROM notes WHERE figure_id = ?1",
            [figure_id],
            |row| {
                Ok(ShowFigureNote {
                    note_markdown: row.get(0)?,
                    updated_at: row.get(1)?,
                })
            },
        )
        .optional()
        .map_err(LamianError::from)
}

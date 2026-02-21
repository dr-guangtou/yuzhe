use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

use crate::db;
use crate::error::LamianError;

#[derive(Debug, Clone)]
pub struct UpdateRequest {
    pub vault_root: PathBuf,
    pub figure_id: String,
    pub name: Option<String>,
    pub caption: Option<String>,
    pub note_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub figure_id: String,
    pub updated_fields: Vec<&'static str>,
}

pub fn update_figure(request: UpdateRequest) -> Result<UpdateResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let normalized_figure_id = normalize_figure_id(&request.figure_id)?;
    let normalized_name = normalize_optional_text_field("name", request.name)?;
    let normalized_caption = normalize_optional_text_field("caption", request.caption)?;
    let note_markdown = request
        .note_file
        .as_deref()
        .map(read_note_file)
        .transpose()?;

    if normalized_name.is_none() && normalized_caption.is_none() && note_markdown.is_none() {
        return Err(LamianError::MissingUpdatePayload);
    }

    let mut connection = db::open_vault_connection(&request.vault_root)?;

    require_figure_exists(&connection, &normalized_figure_id)?;

    persist_updates(
        &mut connection,
        &normalized_figure_id,
        normalized_name.as_deref(),
        normalized_caption.as_deref(),
        note_markdown.as_deref(),
    )?;

    let mut updated_fields = Vec::new();
    if normalized_name.is_some() {
        updated_fields.push("name");
    }
    if normalized_caption.is_some() {
        updated_fields.push("caption");
    }
    if note_markdown.is_some() {
        updated_fields.push("note_file");
    }

    Ok(UpdateResult {
        figure_id: normalized_figure_id,
        updated_fields,
    })
}

fn normalize_figure_id(figure_id: &str) -> Result<String, LamianError> {
    let normalized_figure_id = figure_id.trim();
    if normalized_figure_id.is_empty() {
        return Err(LamianError::MissingUpdateField { field: "figure_id" });
    }
    Ok(normalized_figure_id.to_string())
}

fn normalize_optional_text_field(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<String>, LamianError> {
    match value {
        None => Ok(None),
        Some(raw_value) => {
            let normalized_value = raw_value.trim();
            if normalized_value.is_empty() {
                return Err(LamianError::MissingUpdateField { field });
            }
            Ok(Some(normalized_value.to_string()))
        }
    }
}

fn read_note_file(path: &Path) -> Result<String, LamianError> {
    if !path.exists() {
        return Err(LamianError::InputFileNotFound {
            path: path.to_path_buf(),
        });
    }
    if !path.is_file() {
        return Err(LamianError::InputPathNotFile {
            path: path.to_path_buf(),
        });
    }

    let bytes = fs::read(path)?;
    String::from_utf8(bytes).map_err(|_| LamianError::InvalidUpdateValue {
        field: "note_file",
        reason: "note file content must be valid UTF-8 text",
        value: path.display().to_string(),
    })
}

fn require_figure_exists(connection: &Connection, figure_id: &str) -> Result<(), LamianError> {
    let existing_figure_id: Option<String> = connection
        .query_row(
            "SELECT figure_id FROM figures WHERE figure_id = ?1",
            [figure_id],
            |row| row.get(0),
        )
        .optional()?;

    if existing_figure_id.is_none() {
        return Err(LamianError::UnknownFigureId {
            figure_id: figure_id.to_string(),
        });
    }

    Ok(())
}

fn persist_updates(
    connection: &mut Connection,
    figure_id: &str,
    name: Option<&str>,
    caption: Option<&str>,
    note_markdown: Option<&str>,
) -> Result<(), LamianError> {
    let transaction = connection.transaction()?;

    if name.is_some() || caption.is_some() {
        transaction.execute(
            r#"
UPDATE figures
SET display_name = COALESCE(?2, display_name),
    caption = COALESCE(?3, caption),
    updated_at = CURRENT_TIMESTAMP
WHERE figure_id = ?1
"#,
            params![figure_id, name, caption],
        )?;
    }

    if let Some(note_content) = note_markdown {
        transaction.execute(
            r#"
INSERT INTO notes (figure_id, note_markdown, updated_at)
VALUES (?1, ?2, CURRENT_TIMESTAMP)
ON CONFLICT(figure_id) DO UPDATE SET
    note_markdown = excluded.note_markdown,
    updated_at = CURRENT_TIMESTAMP
"#,
            params![figure_id, note_content],
        )?;
    }

    transaction.commit()?;
    Ok(())
}

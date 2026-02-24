use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::db;
use crate::error::LamianError;

#[derive(Debug, Clone)]
pub struct UpdateRequest {
    pub vault_root: PathBuf,
    pub figure_id: String,
    pub name: Option<String>,
    pub caption: Option<String>,
    pub clear_caption: bool,
    pub note_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
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
    let caption_update = normalize_caption_update(request.caption, request.clear_caption)?;
    let note_markdown = request
        .note_file
        .as_deref()
        .map(read_note_file)
        .transpose()?;

    if normalized_name.is_none()
        && matches!(caption_update, CaptionUpdate::Unchanged)
        && note_markdown.is_none()
    {
        return Err(LamianError::MissingUpdatePayload);
    }

    let mut connection = db::open_vault_connection(&request.vault_root)?;

    require_figure_exists(&connection, &normalized_figure_id)?;

    persist_updates(
        &mut connection,
        &normalized_figure_id,
        normalized_name.as_deref(),
        &caption_update,
        note_markdown.as_deref(),
    )?;

    let mut updated_fields = Vec::new();
    if normalized_name.is_some() {
        updated_fields.push("name");
    }
    if !matches!(caption_update, CaptionUpdate::Unchanged) {
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

enum CaptionUpdate {
    Unchanged,
    Set(String),
    Clear,
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

fn normalize_caption_update(
    caption: Option<String>,
    clear_caption: bool,
) -> Result<CaptionUpdate, LamianError> {
    if clear_caption {
        if caption.is_some() {
            return Err(LamianError::InvalidUpdateValue {
                field: "caption",
                reason: "cannot combine --caption with --clear-caption",
                value: "both provided".to_string(),
            });
        }
        return Ok(CaptionUpdate::Clear);
    }

    match normalize_optional_text_field("caption", caption)? {
        Some(value) => Ok(CaptionUpdate::Set(value)),
        None => Ok(CaptionUpdate::Unchanged),
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
    caption_update: &CaptionUpdate,
    note_markdown: Option<&str>,
) -> Result<(), LamianError> {
    let transaction = connection.transaction()?;

    let caption_value = match caption_update {
        CaptionUpdate::Set(value) => Some(value.as_str()),
        _ => None,
    };
    let clear_caption_value = if matches!(caption_update, CaptionUpdate::Clear) {
        1_i64
    } else {
        0_i64
    };

    if name.is_some() || !matches!(caption_update, CaptionUpdate::Unchanged) {
        transaction.execute(
            r#"
UPDATE figures
SET display_name = COALESCE(?2, display_name),
    caption = CASE
        WHEN ?4 = 1 THEN NULL
        ELSE COALESCE(?3, caption)
    END,
    updated_at = CURRENT_TIMESTAMP
WHERE figure_id = ?1
"#,
            params![figure_id, name, caption_value, clear_caption_value],
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

use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension};

use crate::db;
use crate::error::LamianError;

#[derive(Debug, Clone)]
pub struct UpdateSourceRequest {
    pub vault_root: PathBuf,
    pub figure_id: String,
    pub title: Option<String>,
    pub authors: Option<String>,
    pub published_at: Option<String>,
    pub clear_title: bool,
    pub clear_authors: bool,
    pub clear_published_at: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateSourceResult {
    pub figure_id: String,
    pub updated_fields: Vec<&'static str>,
}

enum SourceFieldUpdate {
    Unchanged,
    Set(String),
    Clear,
}

pub fn update_source_metadata(
    request: UpdateSourceRequest,
) -> Result<UpdateSourceResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let normalized_figure_id = normalize_figure_id(&request.figure_id)?;
    let title_update = normalize_source_field_update("title", request.title, request.clear_title)?;
    let authors_update =
        normalize_source_field_update("authors", request.authors, request.clear_authors)?;
    let published_at_update = normalize_source_field_update(
        "published_at",
        request.published_at,
        request.clear_published_at,
    )?;

    if matches!(title_update, SourceFieldUpdate::Unchanged)
        && matches!(authors_update, SourceFieldUpdate::Unchanged)
        && matches!(published_at_update, SourceFieldUpdate::Unchanged)
    {
        return Err(LamianError::MissingSourcePayload);
    }

    let mut connection = db::open_vault_connection(&request.vault_root)?;
    require_figure_exists(&connection, &normalized_figure_id)?;
    require_source_exists(&connection, &normalized_figure_id)?;

    persist_source_updates(
        &mut connection,
        &normalized_figure_id,
        &title_update,
        &authors_update,
        &published_at_update,
    )?;

    let mut updated_fields = Vec::new();
    if !matches!(title_update, SourceFieldUpdate::Unchanged) {
        updated_fields.push("title");
    }
    if !matches!(authors_update, SourceFieldUpdate::Unchanged) {
        updated_fields.push("authors");
    }
    if !matches!(published_at_update, SourceFieldUpdate::Unchanged) {
        updated_fields.push("published_at");
    }

    Ok(UpdateSourceResult {
        figure_id: normalized_figure_id,
        updated_fields,
    })
}

fn normalize_figure_id(figure_id: &str) -> Result<String, LamianError> {
    let normalized_figure_id = figure_id.trim();
    if normalized_figure_id.is_empty() {
        return Err(LamianError::MissingSourceField { field: "figure_id" });
    }
    Ok(normalized_figure_id.to_string())
}

fn normalize_source_field_update(
    field: &'static str,
    value: Option<String>,
    clear: bool,
) -> Result<SourceFieldUpdate, LamianError> {
    if clear {
        if value.is_some() {
            return Err(LamianError::InvalidSourceValue {
                field,
                reason: "cannot combine set and clear flags for the same field",
                value: "both provided".to_string(),
            });
        }
        return Ok(SourceFieldUpdate::Clear);
    }

    match value {
        None => Ok(SourceFieldUpdate::Unchanged),
        Some(raw_value) => {
            let normalized_value = raw_value.trim();
            if normalized_value.is_empty() {
                return Err(LamianError::MissingSourceField { field });
            }
            Ok(SourceFieldUpdate::Set(normalized_value.to_string()))
        }
    }
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

fn require_source_exists(connection: &Connection, figure_id: &str) -> Result<(), LamianError> {
    let source_exists: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM sources WHERE figure_id = ?1 LIMIT 1",
            [figure_id],
            |row| row.get(0),
        )
        .optional()?;
    if source_exists.is_none() {
        return Err(LamianError::SourceNotFound {
            figure_id: figure_id.to_string(),
        });
    }
    Ok(())
}

fn persist_source_updates(
    connection: &mut Connection,
    figure_id: &str,
    title_update: &SourceFieldUpdate,
    authors_update: &SourceFieldUpdate,
    published_at_update: &SourceFieldUpdate,
) -> Result<(), LamianError> {
    let transaction = connection.transaction()?;

    let title_value = match title_update {
        SourceFieldUpdate::Set(value) => Some(value.as_str()),
        _ => None,
    };
    let clear_title_value = if matches!(title_update, SourceFieldUpdate::Clear) {
        1_i64
    } else {
        0_i64
    };

    let authors_value = match authors_update {
        SourceFieldUpdate::Set(value) => Some(value.as_str()),
        _ => None,
    };
    let clear_authors_value = if matches!(authors_update, SourceFieldUpdate::Clear) {
        1_i64
    } else {
        0_i64
    };

    let published_at_value = match published_at_update {
        SourceFieldUpdate::Set(value) => Some(value.as_str()),
        _ => None,
    };
    let clear_published_at_value = if matches!(published_at_update, SourceFieldUpdate::Clear) {
        1_i64
    } else {
        0_i64
    };

    transaction.execute(
        r#"
UPDATE sources
SET source_title = CASE
        WHEN ?2 = 1 THEN NULL
        ELSE COALESCE(?3, source_title)
    END,
    source_authors = CASE
        WHEN ?4 = 1 THEN NULL
        ELSE COALESCE(?5, source_authors)
    END,
    source_published_at = CASE
        WHEN ?6 = 1 THEN NULL
        ELSE COALESCE(?7, source_published_at)
    END
WHERE figure_id = ?1
"#,
        params![
            figure_id,
            clear_title_value,
            title_value,
            clear_authors_value,
            authors_value,
            clear_published_at_value,
            published_at_value
        ],
    )?;

    transaction.execute(
        "UPDATE figures SET updated_at = CURRENT_TIMESTAMP WHERE figure_id = ?1",
        [figure_id],
    )?;

    transaction.commit()?;
    Ok(())
}

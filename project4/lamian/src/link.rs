use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension};

use crate::db;
use crate::error::LamianError;

#[derive(Debug, Clone)]
pub struct AddLinkRequest {
    pub vault_root: PathBuf,
    pub from_figure_id: String,
    pub to_figure_id: String,
    pub relation: String,
}

#[derive(Debug, Clone)]
pub struct AddLinkResult {
    pub from_figure_id: String,
    pub to_figure_id: String,
    pub normalized_relation: String,
    pub created_link: bool,
}

#[derive(Debug, Clone)]
pub struct RemoveLinkRequest {
    pub vault_root: PathBuf,
    pub from_figure_id: String,
    pub to_figure_id: String,
}

#[derive(Debug, Clone)]
pub struct RemoveLinkResult {
    pub from_figure_id: String,
    pub to_figure_id: String,
    pub removed_count: usize,
}

pub fn add_link(request: AddLinkRequest) -> Result<AddLinkResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let from_figure_id = normalize_link_figure_id("from_figure_id", &request.from_figure_id)?;
    let to_figure_id = normalize_link_figure_id("to_figure_id", &request.to_figure_id)?;
    let normalized_relation = normalize_relation(&request.relation)?;

    if from_figure_id == to_figure_id {
        return Err(LamianError::SelfLinkNotAllowed {
            figure_id: from_figure_id,
        });
    }

    let mut connection = db::open_vault_connection(&request.vault_root)?;

    require_figure_exists(&connection, &from_figure_id)?;
    require_figure_exists(&connection, &to_figure_id)?;

    let created_link = persist_link(
        &mut connection,
        &from_figure_id,
        &to_figure_id,
        &normalized_relation,
    )?;

    Ok(AddLinkResult {
        from_figure_id,
        to_figure_id,
        normalized_relation,
        created_link,
    })
}

pub fn remove_link(request: RemoveLinkRequest) -> Result<RemoveLinkResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let from_figure_id = normalize_link_figure_id("from_figure_id", &request.from_figure_id)?;
    let to_figure_id = normalize_link_figure_id("to_figure_id", &request.to_figure_id)?;

    let mut connection = db::open_vault_connection(&request.vault_root)?;

    require_figure_exists(&connection, &from_figure_id)?;
    require_figure_exists(&connection, &to_figure_id)?;

    let removed_count = delete_links(&mut connection, &from_figure_id, &to_figure_id)?;
    if removed_count == 0 {
        return Err(LamianError::LinkNotFound {
            from_figure_id,
            to_figure_id,
        });
    }

    Ok(RemoveLinkResult {
        from_figure_id,
        to_figure_id,
        removed_count,
    })
}

fn normalize_link_figure_id(field: &'static str, value: &str) -> Result<String, LamianError> {
    let normalized_value = value.trim();
    if normalized_value.is_empty() {
        return Err(LamianError::MissingLinkField { field });
    }
    Ok(normalized_value.to_string())
}

fn normalize_relation(relation: &str) -> Result<String, LamianError> {
    let normalized_relation = relation.trim().to_ascii_lowercase();
    if normalized_relation.is_empty() {
        return Err(LamianError::MissingLinkField { field: "relation" });
    }

    if !normalized_relation.chars().all(is_valid_relation_character) {
        return Err(LamianError::InvalidLinkValue {
            reason: "relation can only include letters, numbers, underscore, hyphen, and colon",
            value: normalized_relation,
        });
    }

    Ok(normalized_relation)
}

fn is_valid_relation_character(value: char) -> bool {
    value.is_ascii_lowercase()
        || value.is_ascii_digit()
        || value == '_'
        || value == '-'
        || value == ':'
}

fn require_figure_exists(connection: &Connection, figure_id: &str) -> Result<(), LamianError> {
    let existing_figure_id = connection
        .query_row(
            "SELECT figure_id FROM figures WHERE figure_id = ?1",
            [figure_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    if existing_figure_id.is_none() {
        return Err(LamianError::UnknownFigureId {
            figure_id: figure_id.to_string(),
        });
    }
    Ok(())
}

fn persist_link(
    connection: &mut Connection,
    from_figure_id: &str,
    to_figure_id: &str,
    relation: &str,
) -> Result<bool, LamianError> {
    let transaction = connection.transaction()?;

    let inserted_rows = transaction.execute(
        "INSERT OR IGNORE INTO links (from_figure_id, to_figure_id, relation_type) VALUES (?1, ?2, ?3)",
        params![from_figure_id, to_figure_id, relation],
    )?;

    transaction.commit()?;
    Ok(inserted_rows > 0)
}

fn delete_links(
    connection: &mut Connection,
    from_figure_id: &str,
    to_figure_id: &str,
) -> Result<usize, LamianError> {
    let transaction = connection.transaction()?;
    let removed_rows = transaction.execute(
        "DELETE FROM links WHERE from_figure_id = ?1 AND to_figure_id = ?2",
        params![from_figure_id, to_figure_id],
    )?;
    transaction.commit()?;
    Ok(removed_rows)
}

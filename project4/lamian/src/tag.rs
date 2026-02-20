use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension, params};

use crate::db;
use crate::error::LamianError;

#[derive(Debug, Clone)]
pub struct AddTagRequest {
    pub vault_root: PathBuf,
    pub figure_id: String,
    pub tag: String,
}

#[derive(Debug, Clone)]
pub struct AddTagResult {
    pub normalized_tag: String,
    pub created_relation: bool,
}

#[derive(Debug, Clone)]
pub struct RemoveTagRequest {
    pub vault_root: PathBuf,
    pub figure_id: String,
    pub tag: String,
}

#[derive(Debug, Clone)]
pub struct RemoveTagResult {
    pub normalized_tag: String,
    pub removed_relation: bool,
}

#[derive(Debug, Clone)]
pub struct RenameTagRequest {
    pub vault_root: PathBuf,
    pub old_tag: String,
    pub new_tag: String,
}

#[derive(Debug, Clone)]
pub struct RenameTagResult {
    pub normalized_old_tag: String,
    pub normalized_new_tag: String,
    pub renamed_count: usize,
}

pub fn add_tag_to_figure(request: AddTagRequest) -> Result<AddTagResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let figure_id = normalize_figure_id(&request.figure_id)?;
    let normalized_tag = normalize_tag(&request.tag)?;

    let vault_paths = db::resolve_vault_paths(&request.vault_root);
    if !vault_paths.database_path.exists() {
        return Err(LamianError::VaultNotInitialized {
            vault_root: request.vault_root,
        });
    }

    let mut connection = Connection::open(vault_paths.database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    if !figure_exists(&connection, &figure_id)? {
        return Err(LamianError::UnknownFigureId { figure_id });
    }

    let created_relation = persist_tag_assignment(&mut connection, &figure_id, &normalized_tag)?;

    Ok(AddTagResult {
        normalized_tag,
        created_relation,
    })
}

pub fn remove_tag_from_figure(request: RemoveTagRequest) -> Result<RemoveTagResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let figure_id = normalize_figure_id(&request.figure_id)?;
    let normalized_tag = normalize_tag(&request.tag)?;

    let vault_paths = db::resolve_vault_paths(&request.vault_root);
    if !vault_paths.database_path.exists() {
        return Err(LamianError::VaultNotInitialized {
            vault_root: request.vault_root,
        });
    }

    let mut connection = Connection::open(vault_paths.database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    if !figure_exists(&connection, &figure_id)? {
        return Err(LamianError::UnknownFigureId { figure_id });
    }

    let removed_relation = remove_tag_assignment(&mut connection, &figure_id, &normalized_tag)?;

    Ok(RemoveTagResult {
        normalized_tag,
        removed_relation,
    })
}

pub fn rename_tag(request: RenameTagRequest) -> Result<RenameTagResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let normalized_old_tag = normalize_tag(&request.old_tag)?;
    let normalized_new_tag = normalize_tag(&request.new_tag)?;

    let vault_paths = db::resolve_vault_paths(&request.vault_root);
    if !vault_paths.database_path.exists() {
        return Err(LamianError::VaultNotInitialized {
            vault_root: request.vault_root,
        });
    }

    if normalized_old_tag == normalized_new_tag {
        return Ok(RenameTagResult {
            normalized_old_tag,
            normalized_new_tag,
            renamed_count: 0,
        });
    }

    let mut connection = Connection::open(vault_paths.database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    let renamed_count =
        rename_tag_assignments(&mut connection, &normalized_old_tag, &normalized_new_tag)?;

    Ok(RenameTagResult {
        normalized_old_tag,
        normalized_new_tag,
        renamed_count,
    })
}

fn normalize_figure_id(figure_id: &str) -> Result<String, LamianError> {
    let normalized_figure_id = figure_id.trim();
    if normalized_figure_id.is_empty() {
        return Err(LamianError::MissingTagField { field: "figure_id" });
    }
    Ok(normalized_figure_id.to_string())
}

fn normalize_tag(tag: &str) -> Result<String, LamianError> {
    let normalized_tag = tag.trim().to_ascii_lowercase();
    if normalized_tag.is_empty() {
        return Err(LamianError::MissingTagField { field: "tag" });
    }

    if normalized_tag.starts_with(':')
        || normalized_tag.ends_with(':')
        || normalized_tag.contains("::")
    {
        return Err(LamianError::InvalidTagValue {
            reason: "tag hierarchy segments cannot be empty",
            value: normalized_tag,
        });
    }

    for segment in normalized_tag.split(':') {
        if segment.is_empty() {
            return Err(LamianError::InvalidTagValue {
                reason: "tag hierarchy segments cannot be empty",
                value: normalized_tag,
            });
        }

        if !segment.chars().all(is_valid_tag_character) {
            return Err(LamianError::InvalidTagValue {
                reason: "tag can only include letters, numbers, underscore, hyphen, and colon",
                value: normalized_tag,
            });
        }
    }

    Ok(normalized_tag)
}

fn is_valid_tag_character(value: char) -> bool {
    value.is_ascii_lowercase() || value.is_ascii_digit() || value == '_' || value == '-'
}

fn figure_exists(connection: &Connection, figure_id: &str) -> Result<bool, LamianError> {
    let existing_figure_id: Option<String> = connection
        .query_row(
            "SELECT figure_id FROM figures WHERE figure_id = ?1",
            [figure_id],
            |row| row.get(0),
        )
        .optional()?;

    Ok(existing_figure_id.is_some())
}

fn persist_tag_assignment(
    connection: &mut Connection,
    figure_id: &str,
    tag_name: &str,
) -> Result<bool, LamianError> {
    let tag_parent = tag_name.rsplit_once(':').map(|(parent, _)| parent);

    let transaction = connection.transaction()?;

    transaction.execute(
        "INSERT OR IGNORE INTO tags (tag_name, tag_parent) VALUES (?1, ?2)",
        params![tag_name, tag_parent],
    )?;

    let tag_id: i64 = transaction.query_row(
        "SELECT tag_id FROM tags WHERE tag_name = ?1",
        [tag_name],
        |row| row.get(0),
    )?;

    let inserted_rows = transaction.execute(
        "INSERT OR IGNORE INTO figure_tags (figure_id, tag_id) VALUES (?1, ?2)",
        params![figure_id, tag_id],
    )?;

    transaction.commit()?;
    Ok(inserted_rows > 0)
}

fn remove_tag_assignment(
    connection: &mut Connection,
    figure_id: &str,
    tag_name: &str,
) -> Result<bool, LamianError> {
    let transaction = connection.transaction()?;

    let tag_id = find_tag_id(&transaction, tag_name)?.ok_or_else(|| LamianError::TagNotFound {
        tag: tag_name.to_string(),
    })?;

    let removed_rows = transaction.execute(
        "DELETE FROM figure_tags WHERE figure_id = ?1 AND tag_id = ?2",
        params![figure_id, tag_id],
    )?;

    if removed_rows == 0 {
        return Err(LamianError::TagNotAssigned {
            figure_id: figure_id.to_string(),
            tag: tag_name.to_string(),
        });
    }

    let remaining_references: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM figure_tags WHERE tag_id = ?1",
        [tag_id],
        |row| row.get(0),
    )?;

    if remaining_references == 0 {
        transaction.execute("DELETE FROM tags WHERE tag_id = ?1", [tag_id])?;
    }

    transaction.commit()?;
    Ok(true)
}

fn rename_tag_assignments(
    connection: &mut Connection,
    old_tag: &str,
    new_tag: &str,
) -> Result<usize, LamianError> {
    let transaction = connection.transaction()?;

    let old_tag_id =
        find_tag_id(&transaction, old_tag)?.ok_or_else(|| LamianError::TagNotFound {
            tag: old_tag.to_string(),
        })?;

    if find_tag_id(&transaction, new_tag)?.is_some() {
        return Err(LamianError::TagAlreadyExists {
            tag: new_tag.to_string(),
        });
    }

    let descendants = load_descendant_tag_rows(&transaction, old_tag)?;

    for descendant in descendants {
        let new_descendant_name = format!(
            "{}{}",
            new_tag,
            descendant
                .strip_prefix(old_tag)
                .expect("descendant prefix to match old tag")
        );
        if descendant != old_tag && find_tag_id(&transaction, &new_descendant_name)?.is_some() {
            return Err(LamianError::TagAlreadyExists {
                tag: new_descendant_name,
            });
        }
    }

    let new_tag_parent = new_tag.rsplit_once(':').map(|(parent, _)| parent);
    transaction.execute(
        "UPDATE tags SET tag_name = ?1, tag_parent = ?2 WHERE tag_id = ?3",
        params![new_tag, new_tag_parent, old_tag_id],
    )?;

    let descendants = load_descendant_tag_rows(&transaction, old_tag)?;
    let mut renamed_count: usize = 1;

    for descendant in descendants {
        let new_descendant_name = format!(
            "{}{}",
            new_tag,
            descendant
                .strip_prefix(old_tag)
                .expect("descendant prefix to match old tag")
        );
        let new_descendant_parent = new_descendant_name
            .rsplit_once(':')
            .map(|(parent, _)| parent);
        transaction.execute(
            "UPDATE tags SET tag_name = ?1, tag_parent = ?2 WHERE tag_name = ?3",
            params![new_descendant_name, new_descendant_parent, descendant],
        )?;
        renamed_count += 1;
    }

    transaction.commit()?;
    Ok(renamed_count)
}

fn load_descendant_tag_rows(
    connection: &Connection,
    root_tag: &str,
) -> Result<Vec<String>, LamianError> {
    let prefix = format!("{root_tag}:");
    let mut statement = connection.prepare(
        "SELECT tag_name FROM tags WHERE tag_name LIKE ?1 ORDER BY LENGTH(tag_name) ASC, tag_name ASC",
    )?;
    let rows = statement.query_map([format!("{prefix}%")], |row| row.get::<_, String>(0))?;
    let mut values = Vec::new();
    for row in rows {
        values.push(row?);
    }
    Ok(values)
}

fn find_tag_id(connection: &Connection, tag_name: &str) -> Result<Option<i64>, LamianError> {
    let tag_id = connection
        .query_row(
            "SELECT tag_id FROM tags WHERE tag_name = ?1",
            [tag_name],
            |row| row.get(0),
        )
        .optional()?;
    Ok(tag_id)
}

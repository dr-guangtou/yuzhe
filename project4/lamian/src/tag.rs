use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension};

use crate::db;
use crate::error::LamianError;
use crate::tag_validation::{normalize_and_validate_tag, TagValidationError};

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

    let mut connection = db::open_vault_connection(&request.vault_root)?;

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

    let mut connection = db::open_vault_connection(&request.vault_root)?;

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

    if normalized_old_tag == normalized_new_tag {
        return Ok(RenameTagResult {
            normalized_old_tag,
            normalized_new_tag,
            renamed_count: 0,
        });
    }

    let mut connection = db::open_vault_connection(&request.vault_root)?;

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
    normalize_and_validate_tag(tag).map_err(|error| match error {
        TagValidationError::MissingTag => LamianError::MissingTagField { field: "tag" },
        TagValidationError::InvalidTag { reason, value } => {
            LamianError::InvalidTagValue { reason, value }
        }
    })
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

    let rename_plan = build_tag_rename_plan(&transaction, old_tag, new_tag)?;

    for plan_row in &rename_plan {
        transaction.execute(
            "UPDATE tags SET tag_name = ?1, tag_parent = ?2 WHERE tag_id = ?3",
            params![
                plan_row.renamed_tag_name.as_str(),
                plan_row.renamed_tag_parent.as_deref(),
                plan_row.tag_id
            ],
        )?;
    }

    transaction.commit()?;
    Ok(rename_plan.len())
}

fn build_tag_rename_plan(
    connection: &Connection,
    old_tag: &str,
    new_tag: &str,
) -> Result<Vec<TagRenamePlanRow>, LamianError> {
    if find_tag_id(connection, old_tag)?.is_none() {
        return Err(LamianError::TagNotFound {
            tag: old_tag.to_string(),
        });
    }

    let rename_rows = load_tag_rows_for_rename(connection, old_tag)?;
    if rename_rows.is_empty() {
        return Err(LamianError::TagNotFound {
            tag: old_tag.to_string(),
        });
    }

    if find_tag_id(connection, new_tag)?.is_some() {
        return Err(LamianError::TagAlreadyExists {
            tag: new_tag.to_string(),
        });
    }

    let mut rename_plan_rows = Vec::with_capacity(rename_rows.len());
    for rename_row in rename_rows {
        let suffix = rename_row
            .original_tag_name
            .strip_prefix(old_tag)
            .ok_or_else(|| LamianError::InvalidTagValue {
                reason: "tag rename source is outside rename root",
                value: rename_row.original_tag_name.clone(),
            })?;
        let renamed_tag_name = format!("{new_tag}{suffix}");
        let renamed_tag_parent = renamed_tag_name
            .rsplit_once(':')
            .map(|(parent, _)| parent.to_string());

        rename_plan_rows.push(TagRenamePlanRow {
            tag_id: rename_row.tag_id,
            original_tag_name: rename_row.original_tag_name,
            renamed_tag_name,
            renamed_tag_parent,
        });
    }

    for plan_row in &rename_plan_rows {
        if plan_row.original_tag_name == plan_row.renamed_tag_name {
            continue;
        }

        let existing_tag_id = find_tag_id(connection, &plan_row.renamed_tag_name)?;
        if let Some(existing_tag_id_value) = existing_tag_id {
            if existing_tag_id_value != plan_row.tag_id {
                return Err(LamianError::TagAlreadyExists {
                    tag: plan_row.renamed_tag_name.clone(),
                });
            }
        }
    }

    Ok(rename_plan_rows)
}

#[derive(Debug)]
struct TagRenameRow {
    tag_id: i64,
    original_tag_name: String,
}

#[derive(Debug)]
struct TagRenamePlanRow {
    tag_id: i64,
    original_tag_name: String,
    renamed_tag_name: String,
    renamed_tag_parent: Option<String>,
}

fn load_tag_rows_for_rename(
    connection: &Connection,
    old_tag: &str,
) -> Result<Vec<TagRenameRow>, LamianError> {
    let mut statement = connection.prepare(
        "SELECT tag_id, tag_name FROM tags WHERE tag_name = ?1 OR tag_name LIKE ?2 ORDER BY LENGTH(tag_name) ASC, tag_name ASC",
    )?;
    let rows = statement.query_map(params![old_tag, format!("{old_tag}:%")], |row| {
        Ok(TagRenameRow {
            tag_id: row.get(0)?,
            original_tag_name: row.get(1)?,
        })
    })?;
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

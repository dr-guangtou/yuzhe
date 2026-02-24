use std::path::{Path, PathBuf};

use clap::ValueEnum;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::db;
use crate::error::LamianError;
use crate::query::{run_query, QueryReferenceMode, QueryRunDetail, RunQueryRequest};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionMode {
    Static,
    Dynamic,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionReferenceMode {
    Auto,
    Id,
    Name,
}

impl CollectionMode {
    fn as_db_value(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Dynamic => "dynamic",
        }
    }

    fn from_db_value(value: &str) -> Result<Self, LamianError> {
        match value {
            "static" => Ok(Self::Static),
            "dynamic" => Ok(Self::Dynamic),
            _ => Err(LamianError::InvalidCollectionValue {
                field: "collection_mode",
                reason: "unsupported collection mode",
                value: value.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateCollectionRequest {
    pub vault_root: PathBuf,
    pub collection_name: String,
    pub query_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateCollectionResult {
    pub collection_id: i64,
    pub collection_name: String,
    pub collection_mode: CollectionMode,
    pub query_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct AddCollectionItemRequest {
    pub vault_root: PathBuf,
    pub collection_reference: String,
    pub reference_mode: CollectionReferenceMode,
    pub figure_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddCollectionItemResult {
    pub collection_id: i64,
    pub collection_name: String,
    pub collection_mode: CollectionMode,
    pub figure_id: String,
    pub created_relation: bool,
}

#[derive(Debug, Clone)]
pub struct RemoveCollectionItemRequest {
    pub vault_root: PathBuf,
    pub collection_reference: String,
    pub reference_mode: CollectionReferenceMode,
    pub figure_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoveCollectionItemResult {
    pub collection_id: i64,
    pub collection_name: String,
    pub collection_mode: CollectionMode,
    pub figure_id: String,
    pub removed_count: usize,
}

#[derive(Debug, Clone)]
pub struct ListCollectionsRequest {
    pub vault_root: PathBuf,
    pub collection_reference: Option<String>,
    pub reference_mode: CollectionReferenceMode,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListCollectionsResult {
    pub collections: Vec<CollectionDetail>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectionDetail {
    pub collection_id: i64,
    pub collection_name: String,
    pub collection_mode: CollectionMode,
    pub query_id: Option<i64>,
    pub figure_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DeleteCollectionRequest {
    pub vault_root: PathBuf,
    pub collection_reference: String,
    pub reference_mode: CollectionReferenceMode,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteCollectionResult {
    pub collection_id: i64,
    pub collection_name: String,
    pub collection_mode: CollectionMode,
    pub query_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct UpdateCollectionRequest {
    pub vault_root: PathBuf,
    pub collection_reference: String,
    pub reference_mode: CollectionReferenceMode,
    pub name: Option<String>,
    pub query_id: Option<i64>,
    pub clear_query_id: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateCollectionResult {
    pub collection_id: i64,
    pub collection_name: String,
    pub collection_mode: CollectionMode,
    pub query_id: Option<i64>,
    pub updated_fields: Vec<&'static str>,
}

#[derive(Debug, Clone)]
struct StoredCollection {
    collection_id: i64,
    collection_name: String,
    collection_mode: CollectionMode,
    query_id: Option<i64>,
}

enum QueryIdUpdate {
    Unchanged,
    Set(i64),
    Clear,
}

pub fn create_collection(
    request: CreateCollectionRequest,
) -> Result<CreateCollectionResult, LamianError> {
    validate_vault_root(&request.vault_root)?;
    let collection_name = normalize_collection_name(&request.collection_name)?;
    let collection_mode = if request.query_id.is_some() {
        CollectionMode::Dynamic
    } else {
        CollectionMode::Static
    };

    let connection = db::open_vault_connection(&request.vault_root)?;
    if collection_name_exists(&connection, &collection_name)? {
        return Err(LamianError::CollectionAlreadyExists { collection_name });
    }

    if let Some(query_id) = request.query_id {
        if !saved_query_exists(&connection, query_id)? {
            return Err(LamianError::InvalidCollectionValue {
                field: "query_id",
                reason: "saved query does not exist",
                value: query_id.to_string(),
            });
        }
    }

    connection.execute(
        r#"
INSERT INTO collections (collection_name, collection_mode, query_id)
VALUES (?1, ?2, ?3)
"#,
        params![
            &collection_name,
            collection_mode.as_db_value(),
            request.query_id
        ],
    )?;
    let collection_id = connection.last_insert_rowid();

    Ok(CreateCollectionResult {
        collection_id,
        collection_name,
        collection_mode,
        query_id: request.query_id,
    })
}

pub fn add_collection_item(
    request: AddCollectionItemRequest,
) -> Result<AddCollectionItemResult, LamianError> {
    validate_vault_root(&request.vault_root)?;
    let collection_reference = normalize_collection_reference(&request.collection_reference)?;
    let figure_id = normalize_figure_id(&request.figure_id)?;

    let connection = db::open_vault_connection(&request.vault_root)?;
    let stored_collection =
        resolve_collection(&connection, &collection_reference, request.reference_mode)?;
    ensure_static_collection(&stored_collection, "add")?;
    ensure_figure_exists(&connection, &figure_id)?;

    let changed_rows = connection.execute(
        "INSERT OR IGNORE INTO collection_items (collection_id, figure_id) VALUES (?1, ?2)",
        params![stored_collection.collection_id, &figure_id],
    )?;

    Ok(AddCollectionItemResult {
        collection_id: stored_collection.collection_id,
        collection_name: stored_collection.collection_name,
        collection_mode: stored_collection.collection_mode,
        figure_id,
        created_relation: changed_rows > 0,
    })
}

pub fn remove_collection_item(
    request: RemoveCollectionItemRequest,
) -> Result<RemoveCollectionItemResult, LamianError> {
    validate_vault_root(&request.vault_root)?;
    let collection_reference = normalize_collection_reference(&request.collection_reference)?;
    let figure_id = normalize_figure_id(&request.figure_id)?;

    let connection = db::open_vault_connection(&request.vault_root)?;
    let stored_collection =
        resolve_collection(&connection, &collection_reference, request.reference_mode)?;
    ensure_static_collection(&stored_collection, "remove")?;

    let changed_rows = connection.execute(
        "DELETE FROM collection_items WHERE collection_id = ?1 AND figure_id = ?2",
        params![stored_collection.collection_id, &figure_id],
    )?;
    let removed_count = changed_rows;

    Ok(RemoveCollectionItemResult {
        collection_id: stored_collection.collection_id,
        collection_name: stored_collection.collection_name,
        collection_mode: stored_collection.collection_mode,
        figure_id,
        removed_count,
    })
}

pub fn list_collections(
    request: ListCollectionsRequest,
) -> Result<ListCollectionsResult, LamianError> {
    validate_vault_root(&request.vault_root)?;
    let connection = db::open_vault_connection(&request.vault_root)?;

    let collections = if let Some(collection_reference_value) = request.collection_reference {
        let collection_reference = normalize_collection_reference(&collection_reference_value)?;
        let stored_collection =
            resolve_collection(&connection, &collection_reference, request.reference_mode)?;
        vec![build_collection_detail(
            &request.vault_root,
            &connection,
            &stored_collection,
        )?]
    } else {
        let all_collections = load_all_collections(&connection)?;
        let mut details = Vec::with_capacity(all_collections.len());
        for stored_collection in all_collections {
            details.push(build_collection_detail(
                &request.vault_root,
                &connection,
                &stored_collection,
            )?);
        }
        details
    };

    Ok(ListCollectionsResult { collections })
}

pub fn delete_collection(
    request: DeleteCollectionRequest,
) -> Result<DeleteCollectionResult, LamianError> {
    validate_vault_root(&request.vault_root)?;
    let collection_reference = normalize_collection_reference(&request.collection_reference)?;
    let connection = db::open_vault_connection(&request.vault_root)?;
    let stored_collection =
        resolve_collection(&connection, &collection_reference, request.reference_mode)?;

    connection.execute(
        "DELETE FROM collections WHERE collection_id = ?1",
        [stored_collection.collection_id],
    )?;

    Ok(DeleteCollectionResult {
        collection_id: stored_collection.collection_id,
        collection_name: stored_collection.collection_name,
        collection_mode: stored_collection.collection_mode,
        query_id: stored_collection.query_id,
    })
}

pub fn update_collection(
    request: UpdateCollectionRequest,
) -> Result<UpdateCollectionResult, LamianError> {
    validate_vault_root(&request.vault_root)?;
    let collection_reference = normalize_collection_reference(&request.collection_reference)?;
    let normalized_name = normalize_optional_collection_name(request.name)?;
    let query_id_update = normalize_query_id_update(request.query_id, request.clear_query_id)?;

    if normalized_name.is_none() && matches!(query_id_update, QueryIdUpdate::Unchanged) {
        return Err(LamianError::InvalidCollectionValue {
            field: "update",
            reason: "provide at least one of --name, --query-id, or --clear-query-id",
            value: "empty payload".to_string(),
        });
    }

    let connection = db::open_vault_connection(&request.vault_root)?;
    let stored_collection =
        resolve_collection(&connection, &collection_reference, request.reference_mode)?;

    if let Some(new_name) = normalized_name.as_deref() {
        if new_name != stored_collection.collection_name
            && collection_name_exists(&connection, new_name)?
        {
            return Err(LamianError::CollectionAlreadyExists {
                collection_name: new_name.to_string(),
            });
        }
    }

    let next_query_id = match query_id_update {
        QueryIdUpdate::Unchanged => stored_collection.query_id,
        QueryIdUpdate::Set(query_id) => {
            if !saved_query_exists(&connection, query_id)? {
                return Err(LamianError::InvalidCollectionValue {
                    field: "query_id",
                    reason: "saved query does not exist",
                    value: query_id.to_string(),
                });
            }
            Some(query_id)
        }
        QueryIdUpdate::Clear => None,
    };

    let next_name = normalized_name
        .as_deref()
        .unwrap_or(stored_collection.collection_name.as_str())
        .to_string();
    let next_mode = if next_query_id.is_some() {
        CollectionMode::Dynamic
    } else {
        CollectionMode::Static
    };

    connection.execute(
        r#"
UPDATE collections
SET collection_name = ?1, collection_mode = ?2, query_id = ?3
WHERE collection_id = ?4
"#,
        params![
            &next_name,
            next_mode.as_db_value(),
            next_query_id,
            stored_collection.collection_id
        ],
    )?;

    let mut updated_fields = Vec::new();
    if next_name != stored_collection.collection_name {
        updated_fields.push("name");
    }
    if next_query_id != stored_collection.query_id {
        updated_fields.push("query_id");
    }
    if next_mode.as_db_value() != stored_collection.collection_mode.as_db_value() {
        updated_fields.push("collection_mode");
    }

    Ok(UpdateCollectionResult {
        collection_id: stored_collection.collection_id,
        collection_name: next_name,
        collection_mode: next_mode,
        query_id: next_query_id,
        updated_fields,
    })
}

fn validate_vault_root(vault_root: &Path) -> Result<(), LamianError> {
    if vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: vault_root.to_path_buf(),
        });
    }
    Ok(())
}

fn normalize_collection_name(value: &str) -> Result<String, LamianError> {
    let normalized_value = value.trim();
    if normalized_value.is_empty() {
        return Err(LamianError::MissingCollectionField {
            field: "collection_name",
        });
    }
    Ok(normalized_value.to_string())
}

fn normalize_optional_collection_name(
    value: Option<String>,
) -> Result<Option<String>, LamianError> {
    match value {
        None => Ok(None),
        Some(name) => Ok(Some(normalize_collection_name(&name)?)),
    }
}

fn normalize_collection_reference(value: &str) -> Result<String, LamianError> {
    let normalized_value = value.trim();
    if normalized_value.is_empty() {
        return Err(LamianError::MissingCollectionField {
            field: "collection",
        });
    }
    Ok(normalized_value.to_string())
}

fn normalize_figure_id(value: &str) -> Result<String, LamianError> {
    let normalized_value = value.trim();
    if normalized_value.is_empty() {
        return Err(LamianError::MissingCollectionField { field: "figure_id" });
    }
    Ok(normalized_value.to_string())
}

fn normalize_query_id_update(
    query_id: Option<i64>,
    clear_query_id: bool,
) -> Result<QueryIdUpdate, LamianError> {
    if clear_query_id {
        if query_id.is_some() {
            return Err(LamianError::InvalidCollectionValue {
                field: "query_id",
                reason: "cannot combine --query-id with --clear-query-id",
                value: "both provided".to_string(),
            });
        }
        return Ok(QueryIdUpdate::Clear);
    }

    match query_id {
        None => Ok(QueryIdUpdate::Unchanged),
        Some(value) => Ok(QueryIdUpdate::Set(value)),
    }
}

fn collection_name_exists(
    connection: &Connection,
    collection_name: &str,
) -> Result<bool, LamianError> {
    let existing_id: Option<i64> = connection
        .query_row(
            "SELECT collection_id FROM collections WHERE collection_name = ?1",
            [collection_name],
            |row| row.get(0),
        )
        .optional()?;
    Ok(existing_id.is_some())
}

fn saved_query_exists(connection: &Connection, query_id: i64) -> Result<bool, LamianError> {
    let existing_id: Option<i64> = connection
        .query_row(
            "SELECT query_id FROM saved_queries WHERE query_id = ?1",
            [query_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(existing_id.is_some())
}

fn resolve_collection(
    connection: &Connection,
    collection_reference: &str,
    reference_mode: CollectionReferenceMode,
) -> Result<StoredCollection, LamianError> {
    match reference_mode {
        CollectionReferenceMode::Auto => {
            if let Ok(collection_id) = collection_reference.parse::<i64>() {
                if let Some(collection) = load_collection_by_id(connection, collection_id)? {
                    return Ok(collection);
                }
            }

            load_collection_by_name(connection, collection_reference)?.ok_or_else(|| {
                LamianError::CollectionNotFound {
                    collection_reference: collection_reference.to_string(),
                }
            })
        }
        CollectionReferenceMode::Id => {
            let collection_id = collection_reference.parse::<i64>().map_err(|_| {
                LamianError::InvalidCollectionValue {
                    field: "collection",
                    reason: "collection reference must be a numeric id when reference mode is `id`",
                    value: collection_reference.to_string(),
                }
            })?;
            load_collection_by_id(connection, collection_id)?.ok_or_else(|| {
                LamianError::CollectionNotFound {
                    collection_reference: collection_reference.to_string(),
                }
            })
        }
        CollectionReferenceMode::Name => load_collection_by_name(connection, collection_reference)?
            .ok_or_else(|| LamianError::CollectionNotFound {
                collection_reference: collection_reference.to_string(),
            }),
    }
}

fn load_collection_by_id(
    connection: &Connection,
    collection_id: i64,
) -> Result<Option<StoredCollection>, LamianError> {
    let row = connection
        .query_row(
            r#"
SELECT collection_id, collection_name, collection_mode, query_id
FROM collections
WHERE collection_id = ?1
"#,
            [collection_id],
            map_collection_row,
        )
        .optional()?;
    Ok(row)
}

fn load_collection_by_name(
    connection: &Connection,
    collection_name: &str,
) -> Result<Option<StoredCollection>, LamianError> {
    let row = connection
        .query_row(
            r#"
SELECT collection_id, collection_name, collection_mode, query_id
FROM collections
WHERE collection_name = ?1
"#,
            [collection_name],
            map_collection_row,
        )
        .optional()?;
    Ok(row)
}

fn load_all_collections(connection: &Connection) -> Result<Vec<StoredCollection>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT collection_id, collection_name, collection_mode, query_id
FROM collections
ORDER BY collection_name ASC
"#,
    )?;
    let mut rows = statement.query([])?;
    let mut collections = Vec::new();

    while let Some(row) = rows.next()? {
        collections.push(map_collection_row(row)?);
    }

    Ok(collections)
}

fn map_collection_row(row: &rusqlite::Row<'_>) -> Result<StoredCollection, rusqlite::Error> {
    let mode_raw: String = row.get(2)?;
    let collection_mode = CollectionMode::from_db_value(&mode_raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(error))
    })?;

    Ok(StoredCollection {
        collection_id: row.get(0)?,
        collection_name: row.get(1)?,
        collection_mode,
        query_id: row.get(3)?,
    })
}

fn ensure_static_collection(
    stored_collection: &StoredCollection,
    operation: &'static str,
) -> Result<(), LamianError> {
    if matches!(stored_collection.collection_mode, CollectionMode::Static) {
        return Ok(());
    }

    Err(LamianError::CollectionModeConflict {
        operation,
        collection_name: stored_collection.collection_name.clone(),
        collection_mode: "dynamic",
    })
}

fn ensure_figure_exists(connection: &Connection, figure_id: &str) -> Result<(), LamianError> {
    let existing_id: Option<String> = connection
        .query_row(
            "SELECT figure_id FROM figures WHERE figure_id = ?1",
            [figure_id],
            |row| row.get(0),
        )
        .optional()?;

    if existing_id.is_none() {
        return Err(LamianError::UnknownFigureId {
            figure_id: figure_id.to_string(),
        });
    }
    Ok(())
}

fn build_collection_detail(
    vault_root: &Path,
    connection: &Connection,
    stored_collection: &StoredCollection,
) -> Result<CollectionDetail, LamianError> {
    let figure_ids = match stored_collection.collection_mode {
        CollectionMode::Static => {
            load_static_figure_ids(connection, stored_collection.collection_id)?
        }
        CollectionMode::Dynamic => load_dynamic_figure_ids(vault_root, stored_collection.query_id)?,
    };

    Ok(CollectionDetail {
        collection_id: stored_collection.collection_id,
        collection_name: stored_collection.collection_name.clone(),
        collection_mode: stored_collection.collection_mode,
        query_id: stored_collection.query_id,
        figure_ids,
    })
}

fn load_static_figure_ids(
    connection: &Connection,
    collection_id: i64,
) -> Result<Vec<String>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT figure_id
FROM collection_items
WHERE collection_id = ?1
ORDER BY figure_id ASC
"#,
    )?;
    let mut rows = statement.query([collection_id])?;
    let mut figure_ids = Vec::new();

    while let Some(row) = rows.next()? {
        figure_ids.push(row.get(0)?);
    }

    Ok(figure_ids)
}

fn load_dynamic_figure_ids(
    vault_root: &Path,
    query_id: Option<i64>,
) -> Result<Vec<String>, LamianError> {
    let query_id = query_id.ok_or_else(|| LamianError::InvalidCollectionValue {
        field: "query_id",
        reason: "dynamic collection requires query_id",
        value: "null".to_string(),
    })?;

    let query_result = run_query(RunQueryRequest {
        vault_root: vault_root.to_path_buf(),
        query_reference: query_id.to_string(),
        reference_mode: QueryReferenceMode::Id,
        detail: QueryRunDetail::Ids,
    })?;
    Ok(query_result.figure_ids)
}

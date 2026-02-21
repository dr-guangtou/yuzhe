use std::path::PathBuf;

use clap::ValueEnum;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::db;
use crate::error::LamianError;

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum QuerySortField {
    FigureId,
    DisplayName,
    CreatedAt,
    UpdatedAt,
}

impl QuerySortField {
    fn as_db_value(self) -> &'static str {
        match self {
            Self::FigureId => "figure_id",
            Self::DisplayName => "display_name",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    fn from_db_value(value: &str) -> Result<Self, LamianError> {
        match value {
            "figure_id" => Ok(Self::FigureId),
            "display_name" => Ok(Self::DisplayName),
            "created_at" => Ok(Self::CreatedAt),
            "updated_at" => Ok(Self::UpdatedAt),
            _ => Err(LamianError::InvalidQueryValue {
                field: "sort_field",
                reason: "unsupported sort field",
                value: value.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum QuerySortOrder {
    Asc,
    Desc,
}

impl QuerySortOrder {
    fn as_db_value(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }

    fn from_db_value(value: &str) -> Result<Self, LamianError> {
        match value {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err(LamianError::InvalidQueryValue {
                field: "sort_order",
                reason: "unsupported sort order",
                value: value.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum QueryRunDetail {
    Ids,
    Full,
}

#[derive(Debug, Clone)]
pub struct SaveQueryRequest {
    pub vault_root: PathBuf,
    pub query_name: String,
    pub tag: Option<String>,
    pub source_key: Option<String>,
    pub text: Option<String>,
    pub sort_field: QuerySortField,
    pub sort_order: QuerySortOrder,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveQueryResult {
    pub query_id: i64,
    pub query_name: String,
}

#[derive(Debug, Clone)]
pub struct ListQueriesRequest {
    pub vault_root: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListQueriesResult {
    pub queries: Vec<SavedQuerySummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SavedQuerySummary {
    pub query_id: i64,
    pub query_name: String,
    pub tag: Option<String>,
    pub source_key: Option<String>,
    pub text: Option<String>,
    pub sort_field: QuerySortField,
    pub sort_order: QuerySortOrder,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct RunQueryRequest {
    pub vault_root: PathBuf,
    pub query_reference: String,
    pub detail: QueryRunDetail,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunQueryResult {
    pub query_id: i64,
    pub query_name: String,
    pub detail: QueryRunDetail,
    pub total_matches: usize,
    pub figure_ids: Vec<String>,
    pub figures: Vec<QueryFigureDetail>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryFigureDetail {
    pub figure_id: String,
    pub display_name: String,
    pub caption: Option<String>,
    pub updated_at: String,
    pub tags: Vec<String>,
    pub source_keys: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DeleteQueryRequest {
    pub vault_root: PathBuf,
    pub query_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteQueryResult {
    pub query_id: i64,
    pub query_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryFilters {
    tag: Option<String>,
    source_key: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Clone)]
struct StoredQuery {
    query_id: i64,
    query_name: String,
    filters: QueryFilters,
    sort_field: QuerySortField,
    sort_order: QuerySortOrder,
    limit: Option<u32>,
}

#[derive(Debug)]
struct FigureRow {
    figure_id: String,
    display_name: String,
    caption: Option<String>,
    updated_at: String,
}

pub fn save_query(request: SaveQueryRequest) -> Result<SaveQueryResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let query_name = normalize_query_name(&request.query_name)?;
    let filters = normalize_filters(request.tag, request.source_key, request.text)?;
    let limit = normalize_limit(request.limit)?;

    let vault_paths = db::resolve_vault_paths(&request.vault_root);
    if !vault_paths.database_path.exists() {
        return Err(LamianError::VaultNotInitialized {
            vault_root: request.vault_root,
        });
    }

    let connection = Connection::open(vault_paths.database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    if query_name_exists(&connection, &query_name)? {
        return Err(LamianError::QueryAlreadyExists { query_name });
    }

    let filters_json =
        serde_json::to_string(&filters).map_err(|error| LamianError::InvalidQueryValue {
            field: "filters",
            reason: "failed to serialize filters",
            value: error.to_string(),
        })?;

    let limit_i64 = limit.map(i64::from);
    connection.execute(
        r#"
INSERT INTO saved_queries (
    query_name,
    filters_json,
    sort_field,
    sort_order,
    limit_count
)
VALUES (?1, ?2, ?3, ?4, ?5)
"#,
        params![
            query_name,
            filters_json,
            request.sort_field.as_db_value(),
            request.sort_order.as_db_value(),
            limit_i64
        ],
    )?;

    let query_id = connection.last_insert_rowid();
    Ok(SaveQueryResult {
        query_id,
        query_name,
    })
}

pub fn list_queries(request: ListQueriesRequest) -> Result<ListQueriesResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let vault_paths = db::resolve_vault_paths(&request.vault_root);
    if !vault_paths.database_path.exists() {
        return Err(LamianError::VaultNotInitialized {
            vault_root: request.vault_root,
        });
    }

    let connection = Connection::open(vault_paths.database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    let mut statement = connection.prepare(
        r#"
SELECT query_id, query_name, filters_json, sort_field, sort_order, limit_count
FROM saved_queries
ORDER BY query_name ASC
"#,
    )?;
    let mut rows = statement.query([])?;
    let mut queries = Vec::new();

    while let Some(row) = rows.next()? {
        let query_id: i64 = row.get(0)?;
        let query_name: String = row.get(1)?;
        let filters_json: String = row.get(2)?;
        let sort_field_raw: String = row.get(3)?;
        let sort_order_raw: String = row.get(4)?;
        let limit_raw: Option<i64> = row.get(5)?;
        let filters = parse_filters_json(&filters_json)?;
        let sort_field = QuerySortField::from_db_value(&sort_field_raw)?;
        let sort_order = QuerySortOrder::from_db_value(&sort_order_raw)?;
        let limit = parse_limit_from_db(limit_raw)?;

        queries.push(SavedQuerySummary {
            query_id,
            query_name,
            tag: filters.tag,
            source_key: filters.source_key,
            text: filters.text,
            sort_field,
            sort_order,
            limit,
        });
    }

    Ok(ListQueriesResult { queries })
}

pub fn run_query(request: RunQueryRequest) -> Result<RunQueryResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let query_reference = normalize_query_reference(&request.query_reference)?;
    let vault_paths = db::resolve_vault_paths(&request.vault_root);
    if !vault_paths.database_path.exists() {
        return Err(LamianError::VaultNotInitialized {
            vault_root: request.vault_root,
        });
    }

    let connection = Connection::open(vault_paths.database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    let stored_query = resolve_stored_query(&connection, &query_reference)?;
    let figure_rows = execute_stored_query(&connection, &stored_query)?;

    let mut figure_ids = Vec::with_capacity(figure_rows.len());
    let mut figures = Vec::new();

    for row in figure_rows {
        figure_ids.push(row.figure_id.clone());
        if matches!(request.detail, QueryRunDetail::Full) {
            figures.push(QueryFigureDetail {
                figure_id: row.figure_id.clone(),
                display_name: row.display_name,
                caption: row.caption,
                updated_at: row.updated_at,
                tags: load_tags_for_figure(&connection, &row.figure_id)?,
                source_keys: load_source_keys_for_figure(&connection, &row.figure_id)?,
            });
        }
    }

    let total_matches = figure_ids.len();
    Ok(RunQueryResult {
        query_id: stored_query.query_id,
        query_name: stored_query.query_name,
        detail: request.detail,
        total_matches,
        figure_ids,
        figures,
    })
}

pub fn delete_query(request: DeleteQueryRequest) -> Result<DeleteQueryResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let query_reference = normalize_query_reference(&request.query_reference)?;
    let vault_paths = db::resolve_vault_paths(&request.vault_root);
    if !vault_paths.database_path.exists() {
        return Err(LamianError::VaultNotInitialized {
            vault_root: request.vault_root,
        });
    }

    let connection = Connection::open(vault_paths.database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    let stored_query = resolve_stored_query(&connection, &query_reference)?;
    connection.execute(
        "DELETE FROM saved_queries WHERE query_id = ?1",
        [stored_query.query_id],
    )?;

    Ok(DeleteQueryResult {
        query_id: stored_query.query_id,
        query_name: stored_query.query_name,
    })
}

fn normalize_query_name(value: &str) -> Result<String, LamianError> {
    let normalized_value = value.trim();
    if normalized_value.is_empty() {
        return Err(LamianError::MissingQueryField { field: "name" });
    }
    Ok(normalized_value.to_string())
}

fn normalize_query_reference(value: &str) -> Result<String, LamianError> {
    let normalized_value = value.trim();
    if normalized_value.is_empty() {
        return Err(LamianError::MissingQueryField { field: "query" });
    }
    Ok(normalized_value.to_string())
}

fn normalize_filters(
    tag: Option<String>,
    source_key: Option<String>,
    text: Option<String>,
) -> Result<QueryFilters, LamianError> {
    let normalized_tag = match tag {
        Some(tag_value) => Some(normalize_tag_filter(&tag_value)?),
        None => None,
    };
    let normalized_source_key = normalize_optional_filter("source_key", source_key)?;
    let normalized_text = normalize_optional_filter("text", text)?;

    if normalized_tag.is_none() && normalized_source_key.is_none() && normalized_text.is_none() {
        return Err(LamianError::MissingQueryField { field: "filters" });
    }

    Ok(QueryFilters {
        tag: normalized_tag,
        source_key: normalized_source_key,
        text: normalized_text,
    })
}

fn normalize_optional_filter(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<String>, LamianError> {
    match value {
        None => Ok(None),
        Some(raw_value) => {
            let normalized_value = raw_value.trim();
            if normalized_value.is_empty() {
                return Err(LamianError::MissingQueryField { field });
            }
            Ok(Some(normalized_value.to_string()))
        }
    }
}

fn normalize_tag_filter(value: &str) -> Result<String, LamianError> {
    let normalized_tag = value.trim().to_ascii_lowercase();
    if normalized_tag.is_empty() {
        return Err(LamianError::MissingQueryField { field: "tag" });
    }

    if normalized_tag.starts_with(':')
        || normalized_tag.ends_with(':')
        || normalized_tag.contains("::")
    {
        return Err(LamianError::InvalidQueryValue {
            field: "tag",
            reason: "tag hierarchy segments cannot be empty",
            value: normalized_tag,
        });
    }

    for segment in normalized_tag.split(':') {
        if segment.is_empty() {
            return Err(LamianError::InvalidQueryValue {
                field: "tag",
                reason: "tag hierarchy segments cannot be empty",
                value: normalized_tag,
            });
        }

        if !segment.chars().all(is_valid_tag_character) {
            return Err(LamianError::InvalidQueryValue {
                field: "tag",
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

fn normalize_limit(limit: Option<u32>) -> Result<Option<u32>, LamianError> {
    match limit {
        None => Ok(None),
        Some(0) => Err(LamianError::InvalidQueryValue {
            field: "limit",
            reason: "limit must be greater than zero",
            value: "0".to_string(),
        }),
        Some(value) => Ok(Some(value)),
    }
}

fn query_name_exists(connection: &Connection, query_name: &str) -> Result<bool, LamianError> {
    let existing_id: Option<i64> = connection
        .query_row(
            "SELECT query_id FROM saved_queries WHERE query_name = ?1",
            [query_name],
            |row| row.get(0),
        )
        .optional()?;
    Ok(existing_id.is_some())
}

fn resolve_stored_query(
    connection: &Connection,
    query_reference: &str,
) -> Result<StoredQuery, LamianError> {
    if let Ok(query_id) = query_reference.parse::<i64>()
        && let Some(query) = load_stored_query_by_id(connection, query_id)?
    {
        return Ok(query);
    }

    let query = load_stored_query_by_name(connection, query_reference)?.ok_or_else(|| {
        LamianError::QueryNotFound {
            query_reference: query_reference.to_string(),
        }
    })?;
    Ok(query)
}

fn load_stored_query_by_id(
    connection: &Connection,
    query_id: i64,
) -> Result<Option<StoredQuery>, LamianError> {
    let row = connection
        .query_row(
            r#"
SELECT query_id, query_name, filters_json, sort_field, sort_order, limit_count
FROM saved_queries
WHERE query_id = ?1
"#,
            [query_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                ))
            },
        )
        .optional()?;

    parse_stored_query_row(row)
}

fn load_stored_query_by_name(
    connection: &Connection,
    query_name: &str,
) -> Result<Option<StoredQuery>, LamianError> {
    let row = connection
        .query_row(
            r#"
SELECT query_id, query_name, filters_json, sort_field, sort_order, limit_count
FROM saved_queries
WHERE query_name = ?1
"#,
            [query_name],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                ))
            },
        )
        .optional()?;

    parse_stored_query_row(row)
}

fn parse_stored_query_row(
    row: Option<(i64, String, String, String, String, Option<i64>)>,
) -> Result<Option<StoredQuery>, LamianError> {
    let Some((query_id, query_name, filters_json, sort_field_raw, sort_order_raw, limit_raw)) = row
    else {
        return Ok(None);
    };

    let filters = parse_filters_json(&filters_json)?;
    let sort_field = QuerySortField::from_db_value(&sort_field_raw)?;
    let sort_order = QuerySortOrder::from_db_value(&sort_order_raw)?;
    let limit = parse_limit_from_db(limit_raw)?;

    Ok(Some(StoredQuery {
        query_id,
        query_name,
        filters,
        sort_field,
        sort_order,
        limit,
    }))
}

fn parse_filters_json(filters_json: &str) -> Result<QueryFilters, LamianError> {
    serde_json::from_str(filters_json).map_err(|error| LamianError::InvalidQueryValue {
        field: "filters_json",
        reason: "failed to parse stored query filters",
        value: error.to_string(),
    })
}

fn parse_limit_from_db(limit_raw: Option<i64>) -> Result<Option<u32>, LamianError> {
    let Some(limit_value) = limit_raw else {
        return Ok(None);
    };

    let limit = u32::try_from(limit_value).map_err(|_| LamianError::InvalidQueryValue {
        field: "limit_count",
        reason: "limit in database cannot be negative",
        value: limit_value.to_string(),
    })?;

    if limit == 0 {
        return Err(LamianError::InvalidQueryValue {
            field: "limit_count",
            reason: "limit in database must be greater than zero",
            value: limit_value.to_string(),
        });
    }

    Ok(Some(limit))
}

fn execute_stored_query(
    connection: &Connection,
    stored_query: &StoredQuery,
) -> Result<Vec<FigureRow>, LamianError> {
    let sort_field_sql = stored_query.sort_field.as_db_value();
    let sort_order_sql = stored_query.sort_order.as_db_value();
    let limit_clause = stored_query
        .limit
        .map(|limit| format!(" LIMIT {limit}"))
        .unwrap_or_default();

    let query_sql = format!(
        r#"
SELECT figures.figure_id, figures.display_name, figures.caption, figures.updated_at
FROM figures
WHERE (?1 IS NULL OR EXISTS (
    SELECT 1
    FROM figure_tags
    JOIN tags ON tags.tag_id = figure_tags.tag_id
    WHERE figure_tags.figure_id = figures.figure_id
      AND tags.tag_name = ?1
))
AND (?2 IS NULL OR EXISTS (
    SELECT 1
    FROM sources
    WHERE sources.figure_id = figures.figure_id
      AND LOWER(sources.source_key) = LOWER(?2)
))
AND (?3 IS NULL OR (
    LOWER(figures.display_name) LIKE ?3 ESCAPE '\'
    OR LOWER(COALESCE(figures.caption, '')) LIKE ?3 ESCAPE '\'
    OR EXISTS (
        SELECT 1
        FROM sources
        WHERE sources.figure_id = figures.figure_id
          AND (
              LOWER(sources.source_key) LIKE ?3 ESCAPE '\'
              OR LOWER(COALESCE(sources.source_title, '')) LIKE ?3 ESCAPE '\'
              OR LOWER(COALESCE(sources.source_authors, '')) LIKE ?3 ESCAPE '\'
          )
    )
    OR EXISTS (
        SELECT 1
        FROM notes
        WHERE notes.figure_id = figures.figure_id
          AND LOWER(notes.note_markdown) LIKE ?3 ESCAPE '\'
    )
    OR EXISTS (
        SELECT 1
        FROM figure_tags
        JOIN tags ON tags.tag_id = figure_tags.tag_id
        WHERE figure_tags.figure_id = figures.figure_id
          AND LOWER(tags.tag_name) LIKE ?3 ESCAPE '\'
    )
))
ORDER BY figures.{sort_field_sql} {sort_order_sql}, figures.figure_id ASC
{limit_clause}
"#
    );

    let text_like_pattern = stored_query.filters.text.as_deref().map(build_like_pattern);

    let mut statement = connection.prepare(&query_sql)?;
    let mut rows = statement.query(params![
        stored_query.filters.tag.as_deref(),
        stored_query.filters.source_key.as_deref(),
        text_like_pattern.as_deref()
    ])?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        results.push(FigureRow {
            figure_id: row.get(0)?,
            display_name: row.get(1)?,
            caption: row.get(2)?,
            updated_at: row.get(3)?,
        });
    }

    Ok(results)
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

fn load_source_keys_for_figure(
    connection: &Connection,
    figure_id: &str,
) -> Result<Vec<String>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT source_key
FROM sources
WHERE figure_id = ?1
ORDER BY source_id ASC
"#,
    )?;
    let mut rows = statement.query([figure_id])?;
    let mut source_keys = Vec::new();

    while let Some(row) = rows.next()? {
        source_keys.push(row.get(0)?);
    }

    Ok(source_keys)
}

fn build_like_pattern(value: &str) -> String {
    let lowered_value = value.to_ascii_lowercase();
    let escaped_value = escape_like_pattern(&lowered_value);
    format!("%{escaped_value}%")
}

fn escape_like_pattern(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '%' => escaped.push_str("\\%"),
            '_' => escaped.push_str("\\_"),
            _ => escaped.push(character),
        }
    }
    escaped
}

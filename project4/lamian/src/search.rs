use std::path::PathBuf;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::db;
use crate::error::LamianError;
use crate::tag_validation::{normalize_and_validate_tag, TagValidationError};

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub vault_root: PathBuf,
    pub tag: Option<String>,
    pub tag_prefix: Option<String>,
    pub source_key: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub figures: Vec<SearchFigure>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchFigure {
    pub figure_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone)]
struct SearchFilters {
    vault_root: PathBuf,
    tag: Option<String>,
    tag_prefix_pattern: Option<String>,
    source_key: Option<String>,
    text_like_pattern: Option<String>,
}

pub fn search_figures(request: SearchRequest) -> Result<SearchResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let filters = SearchFilters::from_request(request)?;

    let mut connection = db::open_vault_connection(&filters.vault_root)?;

    let figures = search_rows(&mut connection, &filters)?;
    Ok(SearchResult { figures })
}

impl SearchFilters {
    fn from_request(request: SearchRequest) -> Result<Self, LamianError> {
        let tag = match request.tag {
            Some(tag_value) => Some(normalize_tag_filter(&tag_value)?),
            None => None,
        };
        let tag_prefix_pattern = match request.tag_prefix {
            Some(tag_prefix_value) => Some(build_tag_prefix_pattern(&normalize_tag_prefix_filter(
                &tag_prefix_value,
            )?)),
            None => None,
        };
        let source_key = normalize_optional_filter("source_key", request.source_key)?;
        let text = normalize_optional_filter("text", request.text)?;
        let text_like_pattern = text.as_deref().map(build_like_pattern);

        Ok(Self {
            vault_root: request.vault_root,
            tag,
            tag_prefix_pattern,
            source_key,
            text_like_pattern,
        })
    }
}

fn search_rows(
    connection: &mut Connection,
    filters: &SearchFilters,
) -> Result<Vec<SearchFigure>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT figures.figure_id, figures.display_name
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
    FROM figure_tags
    JOIN tags ON tags.tag_id = figure_tags.tag_id
    WHERE figure_tags.figure_id = figures.figure_id
      AND tags.tag_name LIKE ?2 ESCAPE '\'
))
AND (?3 IS NULL OR EXISTS (
    SELECT 1
    FROM sources
    WHERE sources.figure_id = figures.figure_id
      AND LOWER(sources.source_key) = LOWER(?3)
))
AND (?4 IS NULL OR (
    LOWER(figures.display_name) LIKE ?4 ESCAPE '\'
    OR LOWER(COALESCE(figures.caption, '')) LIKE ?4 ESCAPE '\'
    OR EXISTS (
        SELECT 1
        FROM sources
        WHERE sources.figure_id = figures.figure_id
          AND (
              LOWER(sources.source_key) LIKE ?4 ESCAPE '\'
              OR LOWER(COALESCE(sources.source_title, '')) LIKE ?4 ESCAPE '\'
              OR LOWER(COALESCE(sources.source_authors, '')) LIKE ?4 ESCAPE '\'
          )
    )
    OR EXISTS (
        SELECT 1
        FROM notes
        WHERE notes.figure_id = figures.figure_id
          AND LOWER(notes.note_markdown) LIKE ?4 ESCAPE '\'
    )
    OR EXISTS (
        SELECT 1
        FROM figure_tags
        JOIN tags ON tags.tag_id = figure_tags.tag_id
        WHERE figure_tags.figure_id = figures.figure_id
          AND LOWER(tags.tag_name) LIKE ?4 ESCAPE '\'
    )
))
ORDER BY figures.figure_id ASC
"#,
    )?;

    let mut rows = statement.query(params![
        filters.tag.as_deref(),
        filters.tag_prefix_pattern.as_deref(),
        filters.source_key.as_deref(),
        filters.text_like_pattern.as_deref()
    ])?;

    let mut figures = Vec::new();
    while let Some(row) = rows.next()? {
        figures.push(SearchFigure {
            figure_id: row.get(0)?,
            display_name: row.get(1)?,
        });
    }

    Ok(figures)
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
                return Err(LamianError::MissingSearchField { field });
            }
            Ok(Some(normalized_value.to_string()))
        }
    }
}

fn normalize_tag_filter(value: &str) -> Result<String, LamianError> {
    normalize_tag_filter_for_field(value, "tag")
}

fn normalize_tag_prefix_filter(value: &str) -> Result<String, LamianError> {
    normalize_tag_filter_for_field(value, "tag_prefix")
}

fn normalize_tag_filter_for_field(value: &str, field: &'static str) -> Result<String, LamianError> {
    normalize_and_validate_tag(value).map_err(|error| match error {
        TagValidationError::MissingTag => LamianError::MissingSearchField { field },
        TagValidationError::InvalidTag { reason, value } => LamianError::InvalidSearchValue {
            field,
            reason,
            value,
        },
    })
}

fn build_tag_prefix_pattern(tag_prefix: &str) -> String {
    let escaped_tag_prefix = escape_like_pattern(tag_prefix);
    format!("{escaped_tag_prefix}:%")
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

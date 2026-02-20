use std::path::PathBuf;

use rusqlite::{Connection, params};

use crate::db;
use crate::error::LamianError;

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub vault_root: PathBuf,
    pub tag: Option<String>,
    pub source_key: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub figures: Vec<SearchFigure>,
}

#[derive(Debug, Clone)]
pub struct SearchFigure {
    pub figure_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone)]
struct SearchFilters {
    vault_root: PathBuf,
    tag: Option<String>,
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

    let vault_paths = db::resolve_vault_paths(&filters.vault_root);
    if !vault_paths.database_path.exists() {
        return Err(LamianError::VaultNotInitialized {
            vault_root: filters.vault_root,
        });
    }

    let mut connection = Connection::open(vault_paths.database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    let figures = search_rows(&mut connection, &filters)?;
    Ok(SearchResult { figures })
}

impl SearchFilters {
    fn from_request(request: SearchRequest) -> Result<Self, LamianError> {
        let tag = match request.tag {
            Some(tag_value) => Some(normalize_tag_filter(&tag_value)?),
            None => None,
        };
        let source_key = normalize_optional_filter("source_key", request.source_key)?;
        let text = normalize_optional_filter("text", request.text)?;
        let text_like_pattern = text.as_deref().map(build_like_pattern);

        Ok(Self {
            vault_root: request.vault_root,
            tag,
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
ORDER BY figures.figure_id ASC
"#,
    )?;

    let mut rows = statement.query(params![
        filters.tag.as_deref(),
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
    let normalized_tag = value.trim().to_ascii_lowercase();
    if normalized_tag.is_empty() {
        return Err(LamianError::MissingSearchField { field: "tag" });
    }

    if normalized_tag.starts_with(':')
        || normalized_tag.ends_with(':')
        || normalized_tag.contains("::")
    {
        return Err(LamianError::InvalidSearchValue {
            field: "tag",
            reason: "tag hierarchy segments cannot be empty",
            value: normalized_tag,
        });
    }

    for segment in normalized_tag.split(':') {
        if segment.is_empty() {
            return Err(LamianError::InvalidSearchValue {
                field: "tag",
                reason: "tag hierarchy segments cannot be empty",
                value: normalized_tag,
            });
        }

        if !segment.chars().all(is_valid_tag_character) {
            return Err(LamianError::InvalidSearchValue {
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

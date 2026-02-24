use std::path::PathBuf;

use rusqlite::{params, Connection};

use crate::cli::{ListSortField, ListSortOrder};
use crate::db;
use crate::error::LamianError;

#[derive(Debug, Clone)]
pub struct ListFiguresRequest {
    pub vault_root: PathBuf,
    pub sort: ListSortField,
    pub order: ListSortOrder,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ListFiguresResult {
    pub figures: Vec<ListFigureRow>,
}

#[derive(Debug, Clone)]
pub struct ListFigureRow {
    pub figure_id: String,
    pub display_name: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn list_figures(request: ListFiguresRequest) -> Result<ListFiguresResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let limit = normalize_limit(request.limit)?;
    let mut connection = db::open_vault_connection(&request.vault_root)?;
    let figures = load_figure_rows(&mut connection, request.sort, request.order, limit)?;

    Ok(ListFiguresResult { figures })
}

fn normalize_limit(limit: Option<u32>) -> Result<Option<u32>, LamianError> {
    match limit {
        None => Ok(None),
        Some(0) => Err(LamianError::InvalidListValue {
            field: "limit",
            reason: "limit must be greater than zero",
            value: "0".to_string(),
        }),
        Some(value) => Ok(Some(value)),
    }
}

fn load_figure_rows(
    connection: &mut Connection,
    sort: ListSortField,
    order: ListSortOrder,
    limit: Option<u32>,
) -> Result<Vec<ListFigureRow>, LamianError> {
    let sort_sql = match sort {
        ListSortField::FigureId => "figure_id",
        ListSortField::DisplayName => "display_name",
        ListSortField::CreatedAt => "created_at",
        ListSortField::UpdatedAt => "updated_at",
    };
    let order_sql = match order {
        ListSortOrder::Asc => "ASC",
        ListSortOrder::Desc => "DESC",
    };

    let base_query = format!(
        r#"
SELECT figure_id, display_name, created_at, updated_at
FROM figures
ORDER BY {sort_sql} {order_sql}, figure_id ASC
"#
    );
    let query_sql = match limit {
        Some(_) => format!("{base_query}LIMIT ?1"),
        None => base_query,
    };

    let mut statement = connection.prepare(&query_sql)?;
    let mut rows = match limit {
        Some(limit_value) => statement.query(params![i64::from(limit_value)])?,
        None => statement.query([])?,
    };

    let mut figures = Vec::new();
    while let Some(row) = rows.next()? {
        figures.push(ListFigureRow {
            figure_id: row.get(0)?,
            display_name: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
        });
    }

    Ok(figures)
}

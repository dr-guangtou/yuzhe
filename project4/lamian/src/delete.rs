use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

use crate::db;
use crate::error::LamianError;

const MANAGED_FIGURES_DIR_NAME: &str = "figures";

#[derive(Debug, Clone)]
pub struct DeleteFigureRequest {
    pub vault_root: PathBuf,
    pub figure_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteFigureResult {
    pub figure_id: String,
    pub removed_managed_file: bool,
    pub removed_orphan_tag_count: usize,
}

pub fn delete_figure(request: DeleteFigureRequest) -> Result<DeleteFigureResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let normalized_figure_id = normalize_figure_id(&request.figure_id)?;
    let managed_figure_root = db::resolve_vault_paths(&request.vault_root)
        .lamian_root
        .join(MANAGED_FIGURES_DIR_NAME);

    let mut connection = db::open_vault_connection(&request.vault_root)?;
    let figure_file_path = load_figure_file_path(&connection, &normalized_figure_id)?;
    let resolved_file_path = resolve_figure_file_path(&request.vault_root, &figure_file_path);
    let should_remove_managed_file = resolved_file_path.starts_with(&managed_figure_root);

    let removed_orphan_tag_count =
        delete_figure_transactional(&mut connection, &normalized_figure_id)?;

    let removed_managed_file = if should_remove_managed_file {
        remove_managed_file_best_effort(&resolved_file_path)
    } else {
        false
    };

    Ok(DeleteFigureResult {
        figure_id: normalized_figure_id,
        removed_managed_file,
        removed_orphan_tag_count,
    })
}

fn normalize_figure_id(figure_id: &str) -> Result<String, LamianError> {
    let normalized_figure_id = figure_id.trim();
    if normalized_figure_id.is_empty() {
        return Err(LamianError::MissingDeleteField { field: "figure_id" });
    }
    Ok(normalized_figure_id.to_string())
}

fn load_figure_file_path(connection: &Connection, figure_id: &str) -> Result<String, LamianError> {
    let file_path: Option<String> = connection
        .query_row(
            "SELECT file_path FROM figures WHERE figure_id = ?1",
            [figure_id],
            |row| row.get(0),
        )
        .optional()?;

    file_path.ok_or_else(|| LamianError::UnknownFigureId {
        figure_id: figure_id.to_string(),
    })
}

fn delete_figure_transactional(
    connection: &mut Connection,
    figure_id: &str,
) -> Result<usize, LamianError> {
    let transaction = connection.transaction()?;

    transaction.execute("DELETE FROM figures WHERE figure_id = ?1", [figure_id])?;
    let removed_orphan_tags = remove_orphan_tags(&transaction)?;

    transaction.commit()?;
    Ok(removed_orphan_tags)
}

fn remove_orphan_tags(connection: &Connection) -> Result<usize, LamianError> {
    let removed_rows = connection.execute(
        r#"
DELETE FROM tags
WHERE tag_id NOT IN (
    SELECT DISTINCT tag_id
    FROM figure_tags
)
AND tag_name NOT IN (
    SELECT DISTINCT tag_parent
    FROM tags
    WHERE tag_parent IS NOT NULL
)
"#,
        params![],
    )?;

    Ok(removed_rows)
}

fn resolve_figure_file_path(vault_root: &Path, file_path_value: &str) -> PathBuf {
    let path = PathBuf::from(file_path_value);
    if path.is_absolute() {
        path
    } else {
        vault_root.join(path)
    }
}

fn remove_managed_file_best_effort(path: &Path) -> bool {
    match std::fs::remove_file(path) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(_) => false,
    }
}

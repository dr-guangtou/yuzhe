use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use rusqlite::{Connection, OptionalExtension};

use crate::db;
use crate::error::LamianError;

const OPEN_LAUNCHER_ENVIRONMENT_VARIABLE: &str = "LAMIAN_OPEN_LAUNCHER";

#[derive(Debug, Clone)]
pub struct OpenFigureRequest {
    pub vault_root: PathBuf,
    pub figure_id: String,
}

#[derive(Debug, Clone)]
pub struct OpenFigureResult {
    pub figure_id: String,
    pub resolved_file_path: PathBuf,
}

pub fn open_figure(request: OpenFigureRequest) -> Result<OpenFigureResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let normalized_figure_id = normalize_figure_id(&request.figure_id)?;
    let connection = db::open_vault_connection(&request.vault_root)?;
    let stored_file_path = load_figure_file_path(&connection, &normalized_figure_id)?;
    let resolved_file_path = resolve_figure_file_path(&request.vault_root, &stored_file_path);

    launch_in_system_viewer(&resolved_file_path)?;

    Ok(OpenFigureResult {
        figure_id: normalized_figure_id,
        resolved_file_path,
    })
}

fn normalize_figure_id(figure_id: &str) -> Result<String, LamianError> {
    let normalized_figure_id = figure_id.trim();
    if normalized_figure_id.is_empty() {
        return Err(LamianError::MissingOpenField { field: "figure_id" });
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

fn resolve_figure_file_path(vault_root: &Path, file_path_value: &str) -> PathBuf {
    let candidate_path = PathBuf::from(file_path_value);
    if candidate_path.is_absolute() {
        candidate_path
    } else {
        vault_root.join(candidate_path)
    }
}

fn launch_in_system_viewer(path: &Path) -> Result<(), LamianError> {
    let (status, launcher) =
        if let Some(custom_launcher) = std::env::var_os(OPEN_LAUNCHER_ENVIRONMENT_VARIABLE) {
            let launcher = PathBuf::from(&custom_launcher)
                .to_string_lossy()
                .to_string();
            let status = Command::new(custom_launcher).arg(path).status()?;
            (status, launcher)
        } else {
            launch_with_default_viewer(path)?
        };

    if status.success() {
        Ok(())
    } else {
        let exit_code = match status.code() {
            Some(code) => code.to_string(),
            None => String::from("terminated by signal"),
        };
        Err(LamianError::OpenViewerLaunchFailed {
            launcher,
            exit_code,
        })
    }
}

#[cfg(target_os = "macos")]
fn launch_with_default_viewer(path: &Path) -> Result<(ExitStatus, String), std::io::Error> {
    let status = Command::new("open").arg(path).status()?;
    Ok((status, String::from("open")))
}

#[cfg(target_os = "windows")]
fn launch_with_default_viewer(path: &Path) -> Result<(ExitStatus, String), std::io::Error> {
    let status = Command::new("cmd")
        .args(["/C", "start", ""])
        .arg(path)
        .status()?;
    Ok((status, String::from("cmd /C start")))
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
fn launch_with_default_viewer(path: &Path) -> Result<(ExitStatus, String), std::io::Error> {
    let status = Command::new("xdg-open").arg(path).status()?;
    Ok((status, String::from("xdg-open")))
}

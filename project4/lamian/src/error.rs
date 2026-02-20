use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LamianError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Sql(#[from] rusqlite::Error),

    #[error("missing required --vault argument for `{command}` command")]
    MissingVaultArgument { command: &'static str },

    #[error("command not implemented yet: {command}")]
    NotImplemented { command: &'static str },

    #[error("invalid vault path: {path:?}")]
    InvalidVaultPath { path: PathBuf },
}

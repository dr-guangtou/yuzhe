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

    #[error(
        "vault is not initialized at path: {vault_root:?}; run `lamian init --vault <path>` first"
    )]
    VaultNotInitialized { vault_root: PathBuf },

    #[error("input file does not exist: {path:?}")]
    InputFileNotFound { path: PathBuf },

    #[error("input path is not a regular file: {path:?}")]
    InputPathNotFile { path: PathBuf },

    #[error("input file path does not include a valid file name: {path:?}")]
    InvalidInputFileName { path: PathBuf },

    #[error("missing required provenance field: {field}")]
    MissingProvenanceField { field: &'static str },

    #[error("invalid provenance value for {field}: {reason}; received: {value}")]
    InvalidProvenanceValue {
        field: &'static str,
        reason: &'static str,
        value: String,
    },

    #[error("unsupported media type for file: {path:?}")]
    UnsupportedMediaType { path: PathBuf },
}

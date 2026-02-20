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

    #[error("missing required tag field: {field}")]
    MissingTagField { field: &'static str },

    #[error("invalid tag value: {reason}; received: {value}")]
    InvalidTagValue { reason: &'static str, value: String },

    #[error("unknown figure id: {figure_id}")]
    UnknownFigureId { figure_id: String },

    #[error("tag not found: {tag}")]
    TagNotFound { tag: String },

    #[error("tag `{tag}` is not assigned to figure `{figure_id}`")]
    TagNotAssigned { figure_id: String, tag: String },

    #[error("tag already exists: {tag}")]
    TagAlreadyExists { tag: String },

    #[error("missing required link field: {field}")]
    MissingLinkField { field: &'static str },

    #[error("invalid link value: {reason}; received: {value}")]
    InvalidLinkValue { reason: &'static str, value: String },

    #[error("self-link is not allowed for figure `{figure_id}`")]
    SelfLinkNotAllowed { figure_id: String },

    #[error("link not found from `{from_figure_id}` to `{to_figure_id}`")]
    LinkNotFound {
        from_figure_id: String,
        to_figure_id: String,
    },

    #[error("missing required search field: {field}")]
    MissingSearchField { field: &'static str },

    #[error("invalid search value for {field}: {reason}; received: {value}")]
    InvalidSearchValue {
        field: &'static str,
        reason: &'static str,
        value: String,
    },
}

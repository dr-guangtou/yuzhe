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

    #[error("invalid list value for {field}: {reason}; received: {value}")]
    InvalidListValue {
        field: &'static str,
        reason: &'static str,
        value: String,
    },

    #[error("missing required show field: {field}")]
    MissingShowField { field: &'static str },

    #[error("invalid show value for {field}: {reason}; received: {value}")]
    InvalidShowValue {
        field: &'static str,
        reason: &'static str,
        value: String,
    },

    #[error("missing required open field: {field}")]
    MissingOpenField { field: &'static str },

    #[error("failed to open figure path with launcher `{launcher}` (exit code: {exit_code})")]
    OpenViewerLaunchFailed { launcher: String, exit_code: String },

    #[error("missing required delete field: {field}")]
    MissingDeleteField { field: &'static str },

    #[error("missing required source field: {field}")]
    MissingSourceField { field: &'static str },

    #[error("missing source update payload: provide at least one source metadata field")]
    MissingSourcePayload,

    #[error("invalid source value for {field}: {reason}; received: {value}")]
    InvalidSourceValue {
        field: &'static str,
        reason: &'static str,
        value: String,
    },

    #[error("source not found for figure id: {figure_id}")]
    SourceNotFound { figure_id: String },

    #[error("missing required update field: {field}")]
    MissingUpdateField { field: &'static str },

    #[error("missing update payload: provide at least one of --name, --caption, or --note-file")]
    MissingUpdatePayload,

    #[error("invalid update value for {field}: {reason}; received: {value}")]
    InvalidUpdateValue {
        field: &'static str,
        reason: &'static str,
        value: String,
    },

    #[error("missing required export field: {field}")]
    MissingExportField { field: &'static str },

    #[error("invalid export value for {field}: {reason}; received: {value}")]
    InvalidExportValue {
        field: &'static str,
        reason: &'static str,
        value: String,
    },

    #[error("failed to serialize export as {format}: {reason}")]
    ExportSerializationFailed {
        format: &'static str,
        reason: String,
    },

    #[error("missing required query field: {field}")]
    MissingQueryField { field: &'static str },

    #[error("invalid query value for {field}: {reason}; received: {value}")]
    InvalidQueryValue {
        field: &'static str,
        reason: &'static str,
        value: String,
    },

    #[error("saved query already exists: {query_name}")]
    QueryAlreadyExists { query_name: String },

    #[error("saved query not found: {query_reference}")]
    QueryNotFound { query_reference: String },

    #[error("missing required collection field: {field}")]
    MissingCollectionField { field: &'static str },

    #[error("invalid collection value for {field}: {reason}; received: {value}")]
    InvalidCollectionValue {
        field: &'static str,
        reason: &'static str,
        value: String,
    },

    #[error("collection already exists: {collection_name}")]
    CollectionAlreadyExists { collection_name: String },

    #[error("collection not found: {collection_reference}")]
    CollectionNotFound { collection_reference: String },

    #[error(
        "collection mode conflict for operation `{operation}` on `{collection_name}`: mode is `{collection_mode}`"
    )]
    CollectionModeConflict {
        operation: &'static str,
        collection_name: String,
        collection_mode: &'static str,
    },

    #[error("missing required import field: {field}")]
    MissingImportField { field: &'static str },

    #[error("invalid import value for {field}: {reason}; received: {value}")]
    InvalidImportValue {
        field: &'static str,
        reason: &'static str,
        value: String,
    },

    #[error("import completed with failures: {failed_count} item(s) failed")]
    ImportCompletedWithFailures { failed_count: usize },

    #[error("missing required bundle field: {field}")]
    MissingBundleField { field: &'static str },

    #[error("invalid bundle value for {field}: {reason}; received: {value}")]
    InvalidBundleValue {
        field: &'static str,
        reason: &'static str,
        value: String,
    },

    #[error("bundle checksum mismatch for `{entry_path}`: expected {expected}, actual {actual}")]
    BundleChecksumMismatch {
        entry_path: String,
        expected: String,
        actual: String,
    },

    #[error("doctor found unresolved issue(s): {issue_count}")]
    DoctorIssuesFound { issue_count: usize },

    #[error("invalid verify value for {field}: {reason}; received: {value}")]
    InvalidVerifyValue {
        field: &'static str,
        reason: &'static str,
        value: String,
    },

    #[error("verify found unresolved issue(s): {issue_count}")]
    VerifyIssuesFound { issue_count: usize },

    #[error("failed to serialize command output as JSON: {reason}")]
    JsonOutputSerializationFailed { reason: String },
}

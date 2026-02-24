use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::read::GzDecoder;
use flate2::{Compression, GzBuilder};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tar::{Archive, Builder, Header, HeaderMode};

use crate::cli::{BundleImportConflictPolicy, ExportFormat};
use crate::db;
use crate::error::LamianError;
use crate::export::{export_metadata, ExportRequest};
use crate::tag_validation::{normalize_and_validate_tag, TagValidationError};

const BUNDLE_VERSION: u32 = 1;
const MANIFEST_ENTRY_PATH: &str = "manifest.json";
const METADATA_ENTRY_PATH: &str = "metadata.json";
const MANAGED_FILES_PREFIX: &str = "files/";
const MANAGED_FIGURES_DIR_NAME: &str = "figures";
const IMPORT_STAGING_DIR_NAME: &str = "bundle_import_staging";
const IMPORT_JOURNAL_FILE_NAME: &str = "bundle_import_journal.json";

#[derive(Debug, Clone)]
pub struct BundleExportRequest {
    pub vault_root: PathBuf,
    pub target_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct BundleExportResult {
    pub target_path: String,
    pub schema_version: i64,
    pub figure_count: usize,
    pub managed_file_count: usize,
    pub metadata_checksum_sha256: String,
    pub manifest_checksum_sha256: String,
}

#[derive(Debug, Clone)]
pub struct BundleImportRequest {
    pub vault_root: PathBuf,
    pub bundle_path: PathBuf,
    pub fail_on_link_loss: bool,
    pub dry_run: bool,
    pub on_conflict: BundleImportConflictPolicy,
}

#[derive(Debug, Clone, Serialize)]
pub struct BundleImportResult {
    pub bundle_path: String,
    pub dry_run: bool,
    pub on_conflict: String,
    pub schema_version: i64,
    pub total_figures: usize,
    pub imported_figures: usize,
    pub skipped_existing_figures: usize,
    pub managed_files_written: usize,
    pub outbound_links_seen: usize,
    pub outbound_links_written: usize,
    pub outbound_links_dropped_missing_target: usize,
}

#[derive(Debug, Clone)]
pub struct BundleInspectRequest {
    pub bundle_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct BundleInspectResult {
    pub bundle_path: String,
    pub bundle_version: u32,
    pub schema_version: i64,
    pub figure_count: usize,
    pub managed_file_count: usize,
    pub outbound_links_seen: usize,
    pub metadata_checksum_sha256: String,
    pub manifest_checksum_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BundleManifest {
    bundle_version: u32,
    schema_version: i64,
    figure_count: usize,
    metadata_checksum_sha256: String,
    managed_files: Vec<ManagedFileManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManagedFileManifestEntry {
    figure_id: String,
    bundle_path: String,
    file_hash_sha256: String,
    file_size_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleMetadataDocument {
    schema_version: i64,
    figures: Vec<BundleFigure>,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleFigure {
    figure_id: String,
    display_name: String,
    caption: Option<String>,
    file_path: String,
    file_hash_sha256: String,
    media_type: String,
    file_size_bytes: u64,
    created_at: String,
    updated_at: String,
    sources: Vec<BundleSource>,
    tags: Vec<String>,
    outbound_links: Vec<BundleLink>,
    note: Option<BundleNote>,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleSource {
    source_type: String,
    source_key: String,
    source_title: Option<String>,
    source_authors: Option<String>,
    source_published_at: Option<String>,
    created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleLink {
    to_figure_id: String,
    relation_type: String,
    created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleNote {
    note_markdown: String,
    updated_at: String,
}

#[derive(Debug)]
struct ManagedFileExportEntry {
    manifest_entry: ManagedFileManifestEntry,
    source_path: PathBuf,
}

#[derive(Debug)]
struct PendingManagedFileWrite {
    bundle_path: String,
    destination_path: PathBuf,
    overwrite_existing: bool,
}

#[derive(Debug, Clone)]
struct StagedManagedFile {
    bundle_path: String,
    staged_path: PathBuf,
    destination_path: PathBuf,
    overwrite_existing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BundleImportJournal {
    status: BundleImportJournalStatus,
    figure_ids: Vec<String>,
    staged_files: Vec<BundleImportJournalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BundleImportJournalStatus {
    Staged,
    Committed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BundleImportJournalEntry {
    bundle_path: String,
    staged_path: String,
    destination_path: String,
}

#[derive(Debug)]
struct PendingLinkInsert {
    from_figure_id: String,
    to_figure_id: String,
    relation_type: String,
    created_at: String,
}

#[derive(Debug)]
struct PreparedFigureImport {
    figure_index: usize,
    persisted_file_path: PathBuf,
    operation: FigureImportOperation,
}

#[derive(Debug, Clone, Copy)]
enum FigureImportOperation {
    Insert,
    Replace,
}

#[derive(Debug)]
struct BundleImportPreparation {
    prepared_figure_imports: Vec<PreparedFigureImport>,
    skipped_existing_figures: usize,
    pending_file_writes: Vec<PendingManagedFileWrite>,
    pending_links: Vec<PendingLinkInsert>,
}

#[derive(Debug)]
struct ValidatedBundle {
    manifest: BundleManifest,
    metadata_document: BundleMetadataDocument,
    manifest_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct LinkInsertSummary {
    outbound_links_seen: usize,
    outbound_links_written: usize,
    outbound_links_dropped_missing_target: usize,
}

pub fn bundle_export(request: BundleExportRequest) -> Result<BundleExportResult, LamianError> {
    validate_vault_root(&request.vault_root)?;
    let target_path = normalize_bundle_target_path(&request.target_path)?;

    let export_result = export_metadata(ExportRequest {
        vault_root: request.vault_root.clone(),
        format: ExportFormat::Json,
        target: None,
    })?;
    let metadata_content = export_result
        .output
        .ok_or(LamianError::InvalidBundleValue {
            field: "metadata",
            reason: "bundle export requires inline metadata output",
            value: "missing export payload".to_string(),
        })?;
    let metadata_bytes = metadata_content.into_bytes();
    let metadata_document = parse_bundle_metadata(&metadata_bytes)?;
    let managed_file_entries = build_managed_file_entries(&request.vault_root, &metadata_document)?;
    let metadata_checksum_sha256 = sha256_bytes(&metadata_bytes);

    let manifest = BundleManifest {
        bundle_version: BUNDLE_VERSION,
        schema_version: metadata_document.schema_version,
        figure_count: metadata_document.figures.len(),
        metadata_checksum_sha256: metadata_checksum_sha256.clone(),
        managed_files: managed_file_entries
            .iter()
            .map(|entry| entry.manifest_entry.clone())
            .collect(),
    };
    let manifest_bytes = serialize_json_with_trailing_newline(&manifest)?;
    let manifest_checksum_sha256 = sha256_bytes(&manifest_bytes);

    write_bundle_archive(
        &target_path,
        &manifest_bytes,
        &metadata_bytes,
        &managed_file_entries,
    )?;

    Ok(BundleExportResult {
        target_path: target_path.display().to_string(),
        schema_version: metadata_document.schema_version,
        figure_count: metadata_document.figures.len(),
        managed_file_count: managed_file_entries.len(),
        metadata_checksum_sha256,
        manifest_checksum_sha256,
    })
}

pub fn bundle_import(request: BundleImportRequest) -> Result<BundleImportResult, LamianError> {
    validate_vault_root(&request.vault_root)?;
    validate_bundle_source_path(&request.bundle_path)?;
    recover_pending_bundle_import(&request.vault_root)?;

    let vault_paths = db::resolve_vault_paths(&request.vault_root);
    let validated_bundle = load_and_validate_bundle(&request.bundle_path)?;
    let manifest = &validated_bundle.manifest;
    let metadata_document = &validated_bundle.metadata_document;

    let mut connection = db::open_vault_connection(&request.vault_root)?;

    let managed_files_by_figure_id = index_managed_files_by_figure_id(manifest)?;
    verify_reference_file_paths(metadata_document, &managed_files_by_figure_id)?;
    let managed_figure_root = vault_paths.lamian_root.join(MANAGED_FIGURES_DIR_NAME);
    let import_staging_root = vault_paths.lamian_root.join(IMPORT_STAGING_DIR_NAME);
    let import_journal_path = vault_paths.lamian_root.join(IMPORT_JOURNAL_FILE_NAME);
    let total_figures = metadata_document.figures.len();
    let preparation = prepare_bundle_import(
        &connection,
        metadata_document,
        &managed_files_by_figure_id,
        &managed_figure_root,
        request.on_conflict,
    )?;
    let imported_figures = preparation.prepared_figure_imports.len();
    let skipped_existing_figures = preparation.skipped_existing_figures;
    let link_insert_summary = summarize_pending_links(
        &connection,
        &preparation.pending_links,
        metadata_document,
        &preparation.prepared_figure_imports,
        request.fail_on_link_loss,
    )?;

    if request.dry_run {
        return Ok(BundleImportResult {
            bundle_path: request.bundle_path.display().to_string(),
            dry_run: true,
            on_conflict: bundle_conflict_policy_name(request.on_conflict).to_string(),
            schema_version: metadata_document.schema_version,
            total_figures,
            imported_figures,
            skipped_existing_figures,
            managed_files_written: preparation.pending_file_writes.len(),
            outbound_links_seen: link_insert_summary.outbound_links_seen,
            outbound_links_written: link_insert_summary.outbound_links_written,
            outbound_links_dropped_missing_target: link_insert_summary
                .outbound_links_dropped_missing_target,
        });
    }

    let mut managed_files_written = 0_usize;
    let transaction = connection.transaction()?;
    let mut imported_figure_ids = Vec::new();
    for prepared_figure_import in &preparation.prepared_figure_imports {
        let figure = &metadata_document.figures[prepared_figure_import.figure_index];
        if matches!(
            prepared_figure_import.operation,
            FigureImportOperation::Replace
        ) {
            replace_figure_record(
                &transaction,
                figure,
                &prepared_figure_import.persisted_file_path,
            )?;
            clear_replaced_figure_dependents(&transaction, &figure.figure_id)?;
        } else {
            insert_figure_record(
                &transaction,
                figure,
                &prepared_figure_import.persisted_file_path,
            )?;
        }
        insert_source_records(&transaction, figure)?;
        insert_tag_records(&transaction, figure)?;
        insert_note_record(&transaction, figure)?;
        imported_figure_ids.push(figure.figure_id.clone());
    }

    let link_insert_summary = insert_link_records(
        &transaction,
        &preparation.pending_links,
        request.fail_on_link_loss,
    )?;

    let stage_session_root = import_staging_root.join(build_import_session_id());
    let staged_files = stage_managed_files(
        &request.bundle_path,
        &preparation.pending_file_writes,
        &managed_figure_root,
        &stage_session_root,
    )?;

    let mut import_journal = if staged_files.is_empty() {
        None
    } else {
        let journal = build_bundle_import_journal(
            BundleImportJournalStatus::Staged,
            imported_figure_ids,
            &staged_files,
        );
        write_import_journal(&import_journal_path, &journal)?;
        Some(journal)
    };

    if let Err(error) = transaction.commit() {
        cleanup_staged_files(&staged_files);
        let _ = remove_import_journal_if_exists(&import_journal_path);
        let _ = cleanup_empty_import_staging_root(&import_staging_root);
        return Err(error.into());
    }

    if let Some(journal) = import_journal.as_mut() {
        journal.status = BundleImportJournalStatus::Committed;
        write_import_journal(&import_journal_path, journal)?;
        promote_staged_managed_files(&staged_files, &mut managed_files_written)?;
        remove_import_journal_if_exists(&import_journal_path)?;
    }
    cleanup_empty_import_staging_root(&import_staging_root)?;

    Ok(BundleImportResult {
        bundle_path: request.bundle_path.display().to_string(),
        dry_run: false,
        on_conflict: bundle_conflict_policy_name(request.on_conflict).to_string(),
        schema_version: metadata_document.schema_version,
        total_figures,
        imported_figures,
        skipped_existing_figures,
        managed_files_written,
        outbound_links_seen: link_insert_summary.outbound_links_seen,
        outbound_links_written: link_insert_summary.outbound_links_written,
        outbound_links_dropped_missing_target: link_insert_summary
            .outbound_links_dropped_missing_target,
    })
}

pub fn bundle_inspect(request: BundleInspectRequest) -> Result<BundleInspectResult, LamianError> {
    validate_bundle_source_path(&request.bundle_path)?;
    let validated_bundle = load_and_validate_bundle(&request.bundle_path)?;
    let outbound_links_seen = validated_bundle
        .metadata_document
        .figures
        .iter()
        .map(|figure| figure.outbound_links.len())
        .sum();

    Ok(BundleInspectResult {
        bundle_path: request.bundle_path.display().to_string(),
        bundle_version: validated_bundle.manifest.bundle_version,
        schema_version: validated_bundle.manifest.schema_version,
        figure_count: validated_bundle.manifest.figure_count,
        managed_file_count: validated_bundle.manifest.managed_files.len(),
        outbound_links_seen,
        metadata_checksum_sha256: validated_bundle.manifest.metadata_checksum_sha256,
        manifest_checksum_sha256: sha256_bytes(&validated_bundle.manifest_bytes),
    })
}

fn validate_vault_root(vault_root: &Path) -> Result<(), LamianError> {
    if vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: vault_root.to_path_buf(),
        });
    }
    Ok(())
}

fn normalize_bundle_target_path(path: &Path) -> Result<PathBuf, LamianError> {
    if path.as_os_str().is_empty() {
        return Err(LamianError::MissingBundleField { field: "target" });
    }

    if path.exists() && path.is_dir() {
        return Err(LamianError::InvalidBundleValue {
            field: "target",
            reason: "target path must point to a file",
            value: path.display().to_string(),
        });
    }

    if let Some(parent_directory) = path.parent() {
        if !parent_directory.as_os_str().is_empty() {
            fs::create_dir_all(parent_directory)?;
        }
    }

    Ok(path.to_path_buf())
}

fn validate_bundle_source_path(path: &Path) -> Result<(), LamianError> {
    if path.as_os_str().is_empty() {
        return Err(LamianError::MissingBundleField {
            field: "bundle_path",
        });
    }
    if !path.exists() {
        return Err(LamianError::InputFileNotFound {
            path: path.to_path_buf(),
        });
    }
    if !path.is_file() {
        return Err(LamianError::InputPathNotFile {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn load_and_validate_bundle(bundle_path: &Path) -> Result<ValidatedBundle, LamianError> {
    let archive_entries = read_bundle_archive(bundle_path)?;
    let manifest_bytes = archive_entries
        .manifest_bytes
        .ok_or(LamianError::InvalidBundleValue {
            field: "archive",
            reason: "bundle manifest entry is missing",
            value: MANIFEST_ENTRY_PATH.to_string(),
        })?;
    let metadata_bytes = archive_entries
        .metadata_bytes
        .ok_or(LamianError::InvalidBundleValue {
            field: "archive",
            reason: "bundle metadata entry is missing",
            value: METADATA_ENTRY_PATH.to_string(),
        })?;

    let manifest = parse_bundle_manifest(&manifest_bytes)?;
    let metadata_document = parse_bundle_metadata(&metadata_bytes)?;
    verify_manifest_against_metadata(&manifest, &metadata_bytes, &metadata_document)?;
    verify_managed_file_entries(bundle_path, &manifest, &metadata_document)?;

    Ok(ValidatedBundle {
        manifest,
        metadata_document,
        manifest_bytes,
    })
}

fn parse_bundle_manifest(bytes: &[u8]) -> Result<BundleManifest, LamianError> {
    let manifest = serde_json::from_slice::<BundleManifest>(bytes).map_err(|error| {
        LamianError::InvalidBundleValue {
            field: "manifest",
            reason: "failed to parse manifest JSON",
            value: error.to_string(),
        }
    })?;

    if manifest.bundle_version != BUNDLE_VERSION {
        return Err(LamianError::InvalidBundleValue {
            field: "manifest.bundle_version",
            reason: "unsupported bundle version",
            value: manifest.bundle_version.to_string(),
        });
    }

    Ok(manifest)
}

fn parse_bundle_metadata(bytes: &[u8]) -> Result<BundleMetadataDocument, LamianError> {
    serde_json::from_slice::<BundleMetadataDocument>(bytes).map_err(|error| {
        LamianError::InvalidBundleValue {
            field: "metadata",
            reason: "failed to parse metadata JSON",
            value: error.to_string(),
        }
    })
}

fn serialize_json_with_trailing_newline<T: Serialize>(value: &T) -> Result<Vec<u8>, LamianError> {
    let mut content =
        serde_json::to_string_pretty(value).map_err(|error| LamianError::InvalidBundleValue {
            field: "manifest",
            reason: "failed to serialize manifest JSON",
            value: error.to_string(),
        })?;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    Ok(content.into_bytes())
}

fn build_managed_file_entries(
    vault_root: &Path,
    metadata_document: &BundleMetadataDocument,
) -> Result<Vec<ManagedFileExportEntry>, LamianError> {
    let vault_paths = db::resolve_vault_paths(vault_root);
    let managed_figure_root = vault_paths.lamian_root.join(MANAGED_FIGURES_DIR_NAME);
    let mut managed_file_entries = Vec::new();
    let mut observed_bundle_paths = HashSet::new();

    for figure in &metadata_document.figures {
        let file_path = PathBuf::from(&figure.file_path);
        let relative_path = match file_path.strip_prefix(&managed_figure_root) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if !file_path.exists() || !file_path.is_file() {
            return Err(LamianError::InvalidBundleValue {
                field: "managed_file",
                reason: "managed file referenced by figure is missing on disk",
                value: file_path.display().to_string(),
            });
        }

        let relative_path_string = relative_path.to_string_lossy().replace('\\', "/");
        let bundle_path = format!("{MANAGED_FILES_PREFIX}{relative_path_string}");
        if !observed_bundle_paths.insert(bundle_path.clone()) {
            return Err(LamianError::InvalidBundleValue {
                field: "managed_file",
                reason: "bundle contains duplicate managed file entry",
                value: bundle_path,
            });
        }

        let (file_hash_sha256, file_size_bytes) = hash_and_size_for_file(&file_path)?;
        managed_file_entries.push(ManagedFileExportEntry {
            manifest_entry: ManagedFileManifestEntry {
                figure_id: figure.figure_id.clone(),
                bundle_path,
                file_hash_sha256,
                file_size_bytes,
            },
            source_path: file_path,
        });
    }

    managed_file_entries.sort_by(|left, right| {
        left.manifest_entry
            .bundle_path
            .cmp(&right.manifest_entry.bundle_path)
    });
    Ok(managed_file_entries)
}

fn write_bundle_archive(
    target_path: &Path,
    manifest_bytes: &[u8],
    metadata_bytes: &[u8],
    managed_file_entries: &[ManagedFileExportEntry],
) -> Result<(), LamianError> {
    let target_file = File::create(target_path)?;
    let gzip_encoder = GzBuilder::new()
        .mtime(0)
        .write(target_file, Compression::default());
    let mut archive = Builder::new(gzip_encoder);
    archive.mode(HeaderMode::Deterministic);

    append_archive_entry(&mut archive, MANIFEST_ENTRY_PATH, manifest_bytes)?;
    append_archive_entry(&mut archive, METADATA_ENTRY_PATH, metadata_bytes)?;
    for managed_file_entry in managed_file_entries {
        append_archive_file_entry(
            &mut archive,
            &managed_file_entry.manifest_entry.bundle_path,
            &managed_file_entry.source_path,
            managed_file_entry.manifest_entry.file_size_bytes,
        )?;
    }

    let gzip_encoder = archive.into_inner()?;
    gzip_encoder.finish()?;
    Ok(())
}

fn append_archive_entry<W: std::io::Write>(
    archive: &mut Builder<W>,
    path: &str,
    content: &[u8],
) -> Result<(), LamianError> {
    let mut header = Header::new_gnu();
    header.set_entry_type(tar::EntryType::Regular);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_size(content.len() as u64);
    header.set_cksum();

    archive.append_data(&mut header, path, Cursor::new(content))?;
    Ok(())
}

fn append_archive_file_entry<W: std::io::Write>(
    archive: &mut Builder<W>,
    path: &str,
    source_path: &Path,
    file_size_bytes: u64,
) -> Result<(), LamianError> {
    let mut header = Header::new_gnu();
    header.set_entry_type(tar::EntryType::Regular);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_size(file_size_bytes);
    header.set_cksum();

    let source_file = File::open(source_path)?;
    archive.append_data(&mut header, path, source_file)?;
    Ok(())
}

struct ArchiveReadResult {
    manifest_bytes: Option<Vec<u8>>,
    metadata_bytes: Option<Vec<u8>>,
}

fn read_bundle_archive(path: &Path) -> Result<ArchiveReadResult, LamianError> {
    let source_file = File::open(path)?;
    let gzip_decoder = GzDecoder::new(source_file);
    let mut archive = Archive::new(gzip_decoder);

    let mut manifest_bytes = None;
    let mut metadata_bytes = None;
    let mut observed_managed_bundle_paths = HashSet::new();

    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let raw_path = entry.path()?;
        let path_string = normalize_archive_entry_path(raw_path.as_ref());
        let entry_type = entry.header().entry_type();
        if !entry_type.is_file() {
            return Err(LamianError::InvalidBundleValue {
                field: "archive",
                reason:
                    "bundle contains unsupported tar member type; only regular files are allowed",
                value: format!(
                    "path={path_string}, entry_type={}",
                    entry_type.as_byte() as char
                ),
            });
        }

        if path_string == MANIFEST_ENTRY_PATH {
            if manifest_bytes.is_some() {
                return Err(LamianError::InvalidBundleValue {
                    field: "archive",
                    reason: "bundle includes duplicate manifest entry",
                    value: path_string,
                });
            }
            let mut content = Vec::new();
            entry.read_to_end(&mut content)?;
            manifest_bytes = Some(content);
            continue;
        }

        if path_string == METADATA_ENTRY_PATH {
            if metadata_bytes.is_some() {
                return Err(LamianError::InvalidBundleValue {
                    field: "archive",
                    reason: "bundle includes duplicate metadata entry",
                    value: path_string,
                });
            }
            let mut content = Vec::new();
            entry.read_to_end(&mut content)?;
            metadata_bytes = Some(content);
            continue;
        }

        if !path_string.starts_with(MANAGED_FILES_PREFIX) {
            return Err(LamianError::InvalidBundleValue {
                field: "archive",
                reason: "bundle includes unexpected archive entry outside manifest/metadata/files",
                value: path_string,
            });
        }

        if !observed_managed_bundle_paths.insert(path_string.clone()) {
            return Err(LamianError::InvalidBundleValue {
                field: "archive",
                reason: "bundle includes duplicate managed file entry",
                value: path_string,
            });
        }
    }

    Ok(ArchiveReadResult {
        manifest_bytes,
        metadata_bytes,
    })
}

fn normalize_archive_entry_path(path: &Path) -> String {
    let mut value = path.to_string_lossy().replace('\\', "/");
    while let Some(stripped) = value.strip_prefix("./") {
        value = stripped.to_string();
    }
    value
}

fn verify_manifest_against_metadata(
    manifest: &BundleManifest,
    metadata_bytes: &[u8],
    metadata_document: &BundleMetadataDocument,
) -> Result<(), LamianError> {
    let metadata_checksum_sha256 = sha256_bytes(metadata_bytes);
    if metadata_checksum_sha256 != manifest.metadata_checksum_sha256 {
        return Err(LamianError::BundleChecksumMismatch {
            entry_path: METADATA_ENTRY_PATH.to_string(),
            expected: manifest.metadata_checksum_sha256.clone(),
            actual: metadata_checksum_sha256,
        });
    }

    if manifest.schema_version != metadata_document.schema_version {
        return Err(LamianError::InvalidBundleValue {
            field: "schema_version",
            reason: "manifest schema version does not match metadata schema version",
            value: format!(
                "manifest={}, metadata={}",
                manifest.schema_version, metadata_document.schema_version
            ),
        });
    }

    if manifest.figure_count != metadata_document.figures.len() {
        return Err(LamianError::InvalidBundleValue {
            field: "figure_count",
            reason: "manifest figure count does not match metadata payload",
            value: format!(
                "manifest={}, metadata={}",
                manifest.figure_count,
                metadata_document.figures.len()
            ),
        });
    }

    Ok(())
}

fn verify_managed_file_entries(
    bundle_path: &Path,
    manifest: &BundleManifest,
    metadata_document: &BundleMetadataDocument,
) -> Result<(), LamianError> {
    let mut known_figure_ids = HashSet::new();
    for figure in &metadata_document.figures {
        known_figure_ids.insert(figure.figure_id.clone());
    }

    let mut observed_figure_ids = HashSet::new();
    let mut observed_bundle_paths = HashSet::new();
    let mut manifest_entries_by_bundle_path = BTreeMap::new();

    for managed_file_entry in &manifest.managed_files {
        if !known_figure_ids.contains(&managed_file_entry.figure_id) {
            return Err(LamianError::InvalidBundleValue {
                field: "manifest.managed_files.figure_id",
                reason: "manifest references unknown figure id",
                value: managed_file_entry.figure_id.clone(),
            });
        }
        if !observed_figure_ids.insert(managed_file_entry.figure_id.clone()) {
            return Err(LamianError::InvalidBundleValue {
                field: "manifest.managed_files.figure_id",
                reason: "manifest has duplicate managed file records for figure id",
                value: managed_file_entry.figure_id.clone(),
            });
        }
        if !observed_bundle_paths.insert(managed_file_entry.bundle_path.clone()) {
            return Err(LamianError::InvalidBundleValue {
                field: "manifest.managed_files.bundle_path",
                reason: "manifest has duplicate managed file bundle path",
                value: managed_file_entry.bundle_path.clone(),
            });
        }
        manifest_entries_by_bundle_path
            .insert(managed_file_entry.bundle_path.clone(), managed_file_entry);
    }

    let source_file = File::open(bundle_path)?;
    let gzip_decoder = GzDecoder::new(source_file);
    let mut archive = Archive::new(gzip_decoder);
    let mut observed_archive_bundle_paths = HashSet::new();

    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let raw_path = entry.path()?;
        let path_string = normalize_archive_entry_path(raw_path.as_ref());
        if !path_string.starts_with(MANAGED_FILES_PREFIX) {
            continue;
        }

        if !observed_archive_bundle_paths.insert(path_string.clone()) {
            return Err(LamianError::InvalidBundleValue {
                field: "archive",
                reason: "bundle includes duplicate managed file entry",
                value: path_string,
            });
        }

        let Some(managed_file_entry) = manifest_entries_by_bundle_path.get(&path_string) else {
            continue;
        };

        let (actual_hash, actual_size) = hash_and_size_for_reader(&mut entry, &path_string)?;
        if actual_hash != managed_file_entry.file_hash_sha256 {
            return Err(LamianError::BundleChecksumMismatch {
                entry_path: managed_file_entry.bundle_path.clone(),
                expected: managed_file_entry.file_hash_sha256.clone(),
                actual: actual_hash,
            });
        }
        if actual_size != managed_file_entry.file_size_bytes {
            return Err(LamianError::InvalidBundleValue {
                field: "manifest.managed_files.file_size_bytes",
                reason: "managed file size does not match archive content",
                value: format!(
                    "{} (manifest={}, archive={})",
                    managed_file_entry.bundle_path, managed_file_entry.file_size_bytes, actual_size
                ),
            });
        }
    }

    for managed_file_entry in &manifest.managed_files {
        if !observed_archive_bundle_paths.contains(&managed_file_entry.bundle_path) {
            return Err(LamianError::InvalidBundleValue {
                field: "archive",
                reason: "managed file entry listed in manifest is missing from archive",
                value: managed_file_entry.bundle_path.clone(),
            });
        }
    }

    Ok(())
}

fn verify_reference_file_paths(
    metadata_document: &BundleMetadataDocument,
    managed_files_by_figure_id: &BTreeMap<String, ManagedFileManifestEntry>,
) -> Result<(), LamianError> {
    for figure in &metadata_document.figures {
        if managed_files_by_figure_id.contains_key(&figure.figure_id) {
            continue;
        }
        validate_reference_file_path(&figure.figure_id, &figure.file_path)?;
    }
    Ok(())
}

fn ensure_bundle_figure_id(figure_id: &str) -> Result<(), LamianError> {
    if figure_id.trim().is_empty() {
        return Err(LamianError::InvalidBundleValue {
            field: "figure.figure_id",
            reason: "bundle figure_id cannot be empty",
            value: figure_id.to_string(),
        });
    }
    Ok(())
}

fn normalize_bundle_link_figure_id(value: &str) -> Result<String, LamianError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(LamianError::InvalidBundleValue {
            field: "figure.outbound_links.to_figure_id",
            reason: "bundle outbound link figure id cannot be empty",
            value: value.to_string(),
        });
    }
    Ok(trimmed.to_string())
}

fn normalize_bundle_relation(figure_id: &str, value: &str) -> Result<String, LamianError> {
    let normalized_relation = value.trim().to_ascii_lowercase();
    if normalized_relation.is_empty() {
        return Err(LamianError::InvalidBundleValue {
            field: "figure.outbound_links.relation_type",
            reason: "bundle outbound link relation cannot be empty",
            value: format!("figure_id={figure_id}, relation={value}"),
        });
    }

    if !normalized_relation.chars().all(is_valid_relation_character) {
        return Err(LamianError::InvalidBundleValue {
            field: "figure.outbound_links.relation_type",
            reason: "bundle outbound link relation can only include letters, numbers, underscore, hyphen, and colon",
            value: format!("figure_id={figure_id}, relation={normalized_relation}"),
        });
    }

    Ok(normalized_relation)
}

fn is_valid_relation_character(value: char) -> bool {
    value.is_ascii_lowercase()
        || value.is_ascii_digit()
        || value == '_'
        || value == '-'
        || value == ':'
}

fn normalize_bundle_tag(figure_id: &str, tag_name: &str) -> Result<String, LamianError> {
    normalize_and_validate_tag(tag_name).map_err(|error| match error {
        TagValidationError::MissingTag => LamianError::InvalidBundleValue {
            field: "figure.tags",
            reason: "bundle tag cannot be empty",
            value: format!("figure_id={figure_id}, tag={tag_name}"),
        },
        TagValidationError::InvalidTag { reason, value } => LamianError::InvalidBundleValue {
            field: "figure.tags",
            reason,
            value: format!("figure_id={figure_id}, tag={value}"),
        },
    })
}

fn normalize_bundle_source_type(figure_id: &str, value: &str) -> Result<String, LamianError> {
    let normalized_value = value.trim().to_ascii_lowercase();
    if normalized_value.is_empty() {
        return Err(LamianError::InvalidBundleValue {
            field: "figure.sources.source_type",
            reason: "bundle source type cannot be empty",
            value: format!("figure_id={figure_id}, source_type={value}"),
        });
    }

    match normalized_value.as_str() {
        "doi" | "url" | "local" | "manual" => Ok(normalized_value),
        _ => Err(LamianError::InvalidBundleValue {
            field: "figure.sources.source_type",
            reason: "bundle source type is not supported",
            value: format!("figure_id={figure_id}, source_type={value}"),
        }),
    }
}

fn normalize_bundle_source_key(
    figure_id: &str,
    source_type: &str,
    source_key: &str,
) -> Result<String, LamianError> {
    let normalized_key = source_key.trim();
    if normalized_key.is_empty() {
        return Err(LamianError::InvalidBundleValue {
            field: "figure.sources.source_key",
            reason: "bundle source key cannot be empty",
            value: format!("figure_id={figure_id}, source_key={source_key}"),
        });
    }

    match source_type {
        "doi" => {
            if !is_valid_doi(normalized_key) {
                return Err(LamianError::InvalidBundleValue {
                    field: "figure.sources.source_key",
                    reason: "DOI must start with `10.` and include `/`",
                    value: format!("figure_id={figure_id}, source_key={normalized_key}"),
                });
            }
        }
        "url" => {
            if !is_valid_url(normalized_key) {
                return Err(LamianError::InvalidBundleValue {
                    field: "figure.sources.source_key",
                    reason: "URL must start with `http://` or `https://`",
                    value: format!("figure_id={figure_id}, source_key={normalized_key}"),
                });
            }
        }
        "local" | "manual" => {}
        _ => {
            return Err(LamianError::InvalidBundleValue {
                field: "figure.sources.source_key",
                reason: "bundle source type is not supported",
                value: format!("figure_id={figure_id}, source_type={source_type}"),
            });
        }
    }

    Ok(normalized_key.to_string())
}

fn is_valid_doi(value: &str) -> bool {
    value.starts_with("10.") && value.contains('/')
}

fn is_valid_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn validate_reference_file_path(figure_id: &str, file_path_value: &str) -> Result<(), LamianError> {
    let trimmed = file_path_value.trim();
    if trimmed.is_empty() {
        return Err(LamianError::InvalidBundleValue {
            field: "figure.file_path",
            reason: "bundle reference file path cannot be empty",
            value: format!("figure_id={figure_id}, file_path={file_path_value}"),
        });
    }

    if is_non_portable_reference_path(trimmed) {
        return Err(LamianError::InvalidBundleValue {
            field: "figure.file_path",
            reason: "bundle reference file path is not portable; expected relative path without parent segments",
            value: format!("figure_id={figure_id}, file_path={file_path_value}"),
        });
    }

    Ok(())
}

fn is_non_portable_reference_path(value: &str) -> bool {
    if is_unc_path(value) || is_windows_drive_path(value) {
        return true;
    }

    let normalized = value.replace('\\', "/");
    let path = Path::new(&normalized);
    if path.is_absolute() {
        return true;
    }

    path.components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
}

fn is_unc_path(value: &str) -> bool {
    value.starts_with("\\\\") || value.starts_with("//")
}

fn is_windows_drive_path(value: &str) -> bool {
    let mut chars = value.chars();
    match (chars.next(), chars.next()) {
        (Some(letter), Some(':')) if letter.is_ascii_alphabetic() => {
            matches!(chars.next(), Some('\\' | '/'))
        }
        _ => false,
    }
}

fn index_managed_files_by_figure_id(
    manifest: &BundleManifest,
) -> Result<BTreeMap<String, ManagedFileManifestEntry>, LamianError> {
    let mut map = BTreeMap::new();
    for managed_file_entry in &manifest.managed_files {
        if map
            .insert(
                managed_file_entry.figure_id.clone(),
                managed_file_entry.clone(),
            )
            .is_some()
        {
            return Err(LamianError::InvalidBundleValue {
                field: "manifest.managed_files.figure_id",
                reason: "manifest has duplicate managed file records for figure id",
                value: managed_file_entry.figure_id.clone(),
            });
        }
    }
    Ok(map)
}

fn prepare_bundle_import(
    connection: &Connection,
    metadata_document: &BundleMetadataDocument,
    managed_files_by_figure_id: &BTreeMap<String, ManagedFileManifestEntry>,
    managed_figure_root: &Path,
    on_conflict: BundleImportConflictPolicy,
) -> Result<BundleImportPreparation, LamianError> {
    let mut prepared_figure_imports = Vec::new();
    let mut skipped_existing_figures = 0_usize;
    let mut pending_file_writes = Vec::new();
    let mut pending_links = Vec::new();

    for (figure_index, figure) in metadata_document.figures.iter().enumerate() {
        ensure_bundle_figure_id(&figure.figure_id)?;
        let figure_exists = figure_exists_in_connection(connection, &figure.figure_id)?;
        let operation = if figure_exists {
            match on_conflict {
                BundleImportConflictPolicy::Skip => {
                    skipped_existing_figures += 1;
                    continue;
                }
                BundleImportConflictPolicy::Error => {
                    return Err(LamianError::InvalidBundleValue {
                        field: "on_conflict",
                        reason: "existing figure conflict encountered during bundle import",
                        value: format!("on_conflict=error, figure_id={}", figure.figure_id),
                    });
                }
                BundleImportConflictPolicy::Replace => FigureImportOperation::Replace,
            }
        } else {
            FigureImportOperation::Insert
        };

        let persisted_file_path = if let Some(managed_entry) =
            managed_files_by_figure_id.get(&figure.figure_id)
        {
            let destination_path =
                build_destination_path_for_managed_file(managed_figure_root, managed_entry)?;
            if destination_path.exists() {
                let destination_owner = figure_id_by_file_path(connection, &destination_path)?;
                let can_overwrite_destination = matches!(operation, FigureImportOperation::Replace)
                    && destination_owner.as_deref() == Some(figure.figure_id.as_str());
                if !can_overwrite_destination {
                    match on_conflict {
                        BundleImportConflictPolicy::Skip => {
                            skipped_existing_figures += 1;
                            continue;
                        }
                        BundleImportConflictPolicy::Error => {
                            return Err(LamianError::InvalidBundleValue {
                                    field: "on_conflict",
                                    reason: "managed file destination conflict encountered during bundle import",
                                    value: format!(
                                        "on_conflict=error, figure_id={}, destination_path={}",
                                        figure.figure_id,
                                        destination_path.display()
                                    ),
                                });
                        }
                        BundleImportConflictPolicy::Replace => {
                            return Err(LamianError::InvalidBundleValue {
                                    field: "on_conflict",
                                    reason: "replace mode requires destination file conflict to belong to the same figure_id",
                                    value: format!(
                                        "figure_id={}, destination_path={}, owner_figure_id={}",
                                        figure.figure_id,
                                        destination_path.display(),
                                        destination_owner.as_deref().unwrap_or("<none>")
                                    ),
                                });
                        }
                    }
                }
            }
            pending_file_writes.push(PendingManagedFileWrite {
                bundle_path: managed_entry.bundle_path.clone(),
                destination_path: destination_path.clone(),
                overwrite_existing: destination_path.exists(),
            });
            destination_path
        } else {
            PathBuf::from(&figure.file_path)
        };

        for outbound_link in &figure.outbound_links {
            let to_figure_id = normalize_bundle_link_figure_id(&outbound_link.to_figure_id)?;
            let normalized_relation =
                normalize_bundle_relation(&figure.figure_id, &outbound_link.relation_type)?;
            pending_links.push(PendingLinkInsert {
                from_figure_id: figure.figure_id.clone(),
                to_figure_id,
                relation_type: normalized_relation,
                created_at: outbound_link.created_at.clone(),
            });
        }

        prepared_figure_imports.push(PreparedFigureImport {
            figure_index,
            persisted_file_path,
            operation,
        });
    }

    Ok(BundleImportPreparation {
        prepared_figure_imports,
        skipped_existing_figures,
        pending_file_writes,
        pending_links,
    })
}

fn build_destination_path_for_managed_file(
    managed_figure_root: &Path,
    managed_file_entry: &ManagedFileManifestEntry,
) -> Result<PathBuf, LamianError> {
    if !managed_file_entry
        .bundle_path
        .starts_with(MANAGED_FILES_PREFIX)
    {
        return Err(LamianError::InvalidBundleValue {
            field: "manifest.managed_files.bundle_path",
            reason: "managed file bundle path must be under files/",
            value: managed_file_entry.bundle_path.clone(),
        });
    }

    let relative_path = managed_file_entry
        .bundle_path
        .trim_start_matches(MANAGED_FILES_PREFIX);
    if relative_path.is_empty() {
        return Err(LamianError::InvalidBundleValue {
            field: "manifest.managed_files.bundle_path",
            reason: "managed file bundle path cannot be empty",
            value: managed_file_entry.bundle_path.clone(),
        });
    }

    let relative_path = PathBuf::from(relative_path);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(LamianError::InvalidBundleValue {
            field: "manifest.managed_files.bundle_path",
            reason: "managed file bundle path must be relative and cannot contain parent segments",
            value: managed_file_entry.bundle_path.clone(),
        });
    }

    Ok(managed_figure_root.join(relative_path))
}

fn insert_figure_record(
    transaction: &Transaction<'_>,
    figure: &BundleFigure,
    persisted_file_path: &Path,
) -> Result<(), LamianError> {
    let file_size_i64 =
        i64::try_from(figure.file_size_bytes).map_err(|_| LamianError::InvalidBundleValue {
            field: "figure.file_size_bytes",
            reason: "file size cannot be represented in SQLite INTEGER",
            value: figure.file_size_bytes.to_string(),
        })?;

    transaction.execute(
        r#"
INSERT INTO figures (
    figure_id,
    display_name,
    caption,
    file_path,
    file_hash_sha256,
    media_type,
    file_size_bytes,
    created_at,
    updated_at
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
"#,
        params![
            figure.figure_id,
            figure.display_name,
            figure.caption,
            persisted_file_path.to_string_lossy(),
            figure.file_hash_sha256,
            figure.media_type,
            file_size_i64,
            figure.created_at,
            figure.updated_at
        ],
    )?;
    Ok(())
}

fn replace_figure_record(
    transaction: &Transaction<'_>,
    figure: &BundleFigure,
    persisted_file_path: &Path,
) -> Result<(), LamianError> {
    let file_size_i64 =
        i64::try_from(figure.file_size_bytes).map_err(|_| LamianError::InvalidBundleValue {
            field: "figure.file_size_bytes",
            reason: "file size cannot be represented in SQLite INTEGER",
            value: figure.file_size_bytes.to_string(),
        })?;

    let affected_rows = transaction.execute(
        r#"
UPDATE figures
SET
    display_name = ?2,
    caption = ?3,
    file_path = ?4,
    file_hash_sha256 = ?5,
    media_type = ?6,
    file_size_bytes = ?7,
    created_at = ?8,
    updated_at = ?9
WHERE figure_id = ?1
"#,
        params![
            figure.figure_id,
            figure.display_name,
            figure.caption,
            persisted_file_path.to_string_lossy(),
            figure.file_hash_sha256,
            figure.media_type,
            file_size_i64,
            figure.created_at,
            figure.updated_at
        ],
    )?;

    if affected_rows == 0 {
        return Err(LamianError::UnknownFigureId {
            figure_id: figure.figure_id.clone(),
        });
    }

    Ok(())
}

fn clear_replaced_figure_dependents(
    transaction: &Transaction<'_>,
    figure_id: &str,
) -> Result<(), LamianError> {
    transaction.execute("DELETE FROM sources WHERE figure_id = ?1", [figure_id])?;
    transaction.execute("DELETE FROM figure_tags WHERE figure_id = ?1", [figure_id])?;
    transaction.execute("DELETE FROM notes WHERE figure_id = ?1", [figure_id])?;
    transaction.execute("DELETE FROM links WHERE from_figure_id = ?1", [figure_id])?;
    Ok(())
}

fn insert_source_records(
    transaction: &Transaction<'_>,
    figure: &BundleFigure,
) -> Result<(), LamianError> {
    for source in &figure.sources {
        let normalized_source_type =
            normalize_bundle_source_type(&figure.figure_id, &source.source_type)?;
        let normalized_source_key = normalize_bundle_source_key(
            &figure.figure_id,
            &normalized_source_type,
            &source.source_key,
        )?;
        transaction.execute(
            r#"
INSERT INTO sources (
    figure_id,
    source_type,
    source_key,
    source_title,
    source_authors,
    source_published_at,
    created_at
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
"#,
            params![
                figure.figure_id,
                normalized_source_type,
                normalized_source_key,
                source.source_title,
                source.source_authors,
                source.source_published_at,
                source.created_at
            ],
        )?;
    }
    Ok(())
}

fn insert_tag_records(
    transaction: &Transaction<'_>,
    figure: &BundleFigure,
) -> Result<(), LamianError> {
    for tag_name in &figure.tags {
        let normalized_tag = normalize_bundle_tag(&figure.figure_id, tag_name)?;
        let parent_tag_name = parent_tag_name(&normalized_tag);
        transaction.execute(
            "INSERT OR IGNORE INTO tags (tag_name, tag_parent) VALUES (?1, ?2)",
            params![normalized_tag, parent_tag_name],
        )?;
        let tag_id: i64 = transaction.query_row(
            "SELECT tag_id FROM tags WHERE tag_name = ?1",
            [normalized_tag],
            |row| row.get(0),
        )?;
        transaction.execute(
            "INSERT OR IGNORE INTO figure_tags (figure_id, tag_id) VALUES (?1, ?2)",
            params![figure.figure_id, tag_id],
        )?;
    }
    Ok(())
}

fn insert_note_record(
    transaction: &Transaction<'_>,
    figure: &BundleFigure,
) -> Result<(), LamianError> {
    let Some(note) = &figure.note else {
        return Ok(());
    };

    transaction.execute(
        "INSERT INTO notes (figure_id, note_markdown, updated_at) VALUES (?1, ?2, ?3)",
        params![figure.figure_id, note.note_markdown, note.updated_at],
    )?;
    Ok(())
}

fn insert_link_records(
    transaction: &Transaction<'_>,
    pending_links: &[PendingLinkInsert],
    fail_on_link_loss: bool,
) -> Result<LinkInsertSummary, LamianError> {
    let mut outbound_links_written = 0_usize;
    let mut outbound_links_dropped_missing_target = 0_usize;

    for pending_link in pending_links {
        if !figure_exists(transaction, &pending_link.to_figure_id)? {
            if fail_on_link_loss {
                return Err(LamianError::InvalidBundleValue {
                    field: "figure.outbound_links.to_figure_id",
                    reason: "bundle outbound link target is missing during import with --fail-on-link-loss",
                    value: format!(
                        "from_figure_id={}, to_figure_id={}, relation_type={}",
                        pending_link.from_figure_id,
                        pending_link.to_figure_id,
                        pending_link.relation_type
                    ),
                });
            }
            outbound_links_dropped_missing_target += 1;
            continue;
        }
        transaction.execute(
            r#"
INSERT OR IGNORE INTO links (
    from_figure_id,
    to_figure_id,
    relation_type,
    created_at
)
VALUES (?1, ?2, ?3, ?4)
"#,
            params![
                pending_link.from_figure_id,
                pending_link.to_figure_id,
                pending_link.relation_type,
                pending_link.created_at
            ],
        )?;
        outbound_links_written += 1;
    }
    Ok(LinkInsertSummary {
        outbound_links_seen: pending_links.len(),
        outbound_links_written,
        outbound_links_dropped_missing_target,
    })
}

fn summarize_pending_links(
    connection: &Connection,
    pending_links: &[PendingLinkInsert],
    metadata_document: &BundleMetadataDocument,
    prepared_figure_imports: &[PreparedFigureImport],
    fail_on_link_loss: bool,
) -> Result<LinkInsertSummary, LamianError> {
    let mut imported_figure_ids = HashSet::new();
    for prepared_figure_import in prepared_figure_imports {
        let figure_id = metadata_document.figures[prepared_figure_import.figure_index]
            .figure_id
            .clone();
        imported_figure_ids.insert(figure_id);
    }

    let mut outbound_links_written = 0_usize;
    let mut outbound_links_dropped_missing_target = 0_usize;

    for pending_link in pending_links {
        let target_exists = if imported_figure_ids.contains(&pending_link.to_figure_id) {
            true
        } else {
            figure_exists_in_connection(connection, &pending_link.to_figure_id)?
        };

        if target_exists {
            outbound_links_written += 1;
            continue;
        }

        if fail_on_link_loss {
            return Err(LamianError::InvalidBundleValue {
                field: "figure.outbound_links.to_figure_id",
                reason:
                    "bundle outbound link target is missing during import with --fail-on-link-loss",
                value: format!(
                    "from_figure_id={}, to_figure_id={}, relation_type={}",
                    pending_link.from_figure_id,
                    pending_link.to_figure_id,
                    pending_link.relation_type
                ),
            });
        }
        outbound_links_dropped_missing_target += 1;
    }

    Ok(LinkInsertSummary {
        outbound_links_seen: pending_links.len(),
        outbound_links_written,
        outbound_links_dropped_missing_target,
    })
}

fn build_import_session_id() -> String {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("session-{}-{now_nanos}", std::process::id())
}

fn stage_managed_files(
    bundle_path: &Path,
    pending_file_writes: &[PendingManagedFileWrite],
    managed_figure_root: &Path,
    stage_session_root: &Path,
) -> Result<Vec<StagedManagedFile>, LamianError> {
    let mut staged_files = Vec::new();
    if pending_file_writes.is_empty() {
        return Ok(staged_files);
    }

    let mut pending_by_bundle_path = BTreeMap::new();
    for pending_file_write in pending_file_writes {
        if pending_by_bundle_path
            .insert(
                pending_file_write.bundle_path.clone(),
                (
                    pending_file_write.destination_path.clone(),
                    pending_file_write.overwrite_existing,
                ),
            )
            .is_some()
        {
            return Err(LamianError::InvalidBundleValue {
                field: "managed_file",
                reason: "duplicate pending managed file write for bundle path",
                value: pending_file_write.bundle_path.clone(),
            });
        }
    }
    let mut found_pending_bundle_paths = HashSet::new();

    let source_file = File::open(bundle_path)?;
    let gzip_decoder = GzDecoder::new(source_file);
    let mut archive = Archive::new(gzip_decoder);
    let mut observed_archive_bundle_paths = HashSet::new();

    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let raw_path = entry.path()?;
        let path_string = normalize_archive_entry_path(raw_path.as_ref());
        if !path_string.starts_with(MANAGED_FILES_PREFIX) {
            continue;
        }
        if !observed_archive_bundle_paths.insert(path_string.clone()) {
            cleanup_staged_files(&staged_files);
            return Err(LamianError::InvalidBundleValue {
                field: "archive",
                reason: "bundle includes duplicate managed file entry",
                value: path_string,
            });
        }

        let Some((destination_path, overwrite_existing)) = pending_by_bundle_path.get(&path_string)
        else {
            continue;
        };

        let relative_destination =
            destination_path
                .strip_prefix(managed_figure_root)
                .map_err(|_| LamianError::InvalidBundleValue {
                    field: "managed_file",
                    reason: "managed file destination must be within .lamian/figures",
                    value: destination_path.display().to_string(),
                })?;
        let staged_path = stage_session_root.join(relative_destination);

        if let Some(parent_directory) = staged_path.parent() {
            fs::create_dir_all(parent_directory)?;
        }
        let mut staged_file = match File::create(&staged_path) {
            Ok(file) => file,
            Err(error) => {
                cleanup_staged_files(&staged_files);
                return Err(error.into());
            }
        };
        if let Err(error) = std::io::copy(&mut entry, &mut staged_file) {
            cleanup_staged_files(&staged_files);
            return Err(error.into());
        }

        staged_files.push(StagedManagedFile {
            bundle_path: path_string.clone(),
            staged_path,
            destination_path: destination_path.clone(),
            overwrite_existing: *overwrite_existing,
        });
        found_pending_bundle_paths.insert(path_string);
    }

    for pending_file_write in pending_file_writes {
        if !found_pending_bundle_paths.contains(&pending_file_write.bundle_path) {
            cleanup_staged_files(&staged_files);
            return Err(LamianError::InvalidBundleValue {
                field: "archive",
                reason: "managed file entry listed in manifest is missing from archive",
                value: pending_file_write.bundle_path.clone(),
            });
        }
    }

    Ok(staged_files)
}

fn promote_staged_managed_files(
    staged_files: &[StagedManagedFile],
    managed_files_written: &mut usize,
) -> Result<(), LamianError> {
    for staged_file in staged_files {
        if staged_file.destination_path.exists() {
            if staged_file.overwrite_existing {
                if !staged_file.staged_path.exists() {
                    return Err(LamianError::InvalidBundleValue {
                        field: "import_journal",
                        reason: "staged file is missing before replacement promotion",
                        value: staged_file.bundle_path.clone(),
                    });
                }
                fs::remove_file(&staged_file.destination_path)?;
            } else {
                if staged_file.staged_path.exists() {
                    fs::remove_file(&staged_file.staged_path)?;
                }
                continue;
            }
        }
        if !staged_file.staged_path.exists() {
            return Err(LamianError::InvalidBundleValue {
                field: "import_journal",
                reason: "staged file is missing before promotion to destination",
                value: staged_file.bundle_path.clone(),
            });
        }

        if let Some(parent_directory) = staged_file.destination_path.parent() {
            fs::create_dir_all(parent_directory)?;
        }
        move_file_with_fallback(&staged_file.staged_path, &staged_file.destination_path)?;
        *managed_files_written += 1;
    }
    Ok(())
}

fn move_file_with_fallback(source_path: &Path, destination_path: &Path) -> Result<(), LamianError> {
    if fs::rename(source_path, destination_path).is_ok() {
        return Ok(());
    }
    fs::copy(source_path, destination_path)?;
    fs::remove_file(source_path)?;
    Ok(())
}

fn cleanup_staged_files(staged_files: &[StagedManagedFile]) {
    for staged_file in staged_files {
        let _ = fs::remove_file(&staged_file.staged_path);
    }
}

fn build_bundle_import_journal(
    status: BundleImportJournalStatus,
    figure_ids: Vec<String>,
    staged_files: &[StagedManagedFile],
) -> BundleImportJournal {
    BundleImportJournal {
        status,
        figure_ids,
        staged_files: staged_files
            .iter()
            .map(|staged_file| BundleImportJournalEntry {
                bundle_path: staged_file.bundle_path.clone(),
                staged_path: staged_file.staged_path.display().to_string(),
                destination_path: staged_file.destination_path.display().to_string(),
            })
            .collect(),
    }
}

fn write_import_journal(
    import_journal_path: &Path,
    journal: &BundleImportJournal,
) -> Result<(), LamianError> {
    if let Some(parent_directory) = import_journal_path.parent() {
        fs::create_dir_all(parent_directory)?;
    }
    let mut content =
        serde_json::to_string_pretty(journal).map_err(|error| LamianError::InvalidBundleValue {
            field: "import_journal",
            reason: "failed to serialize import journal",
            value: error.to_string(),
        })?;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    fs::write(import_journal_path, content.into_bytes())?;
    Ok(())
}

fn read_import_journal(import_journal_path: &Path) -> Result<BundleImportJournal, LamianError> {
    let bytes = fs::read(import_journal_path)?;
    serde_json::from_slice::<BundleImportJournal>(&bytes).map_err(|error| {
        LamianError::InvalidBundleValue {
            field: "import_journal",
            reason: "failed to parse import journal",
            value: error.to_string(),
        }
    })
}

fn remove_import_journal_if_exists(import_journal_path: &Path) -> Result<(), LamianError> {
    if import_journal_path.exists() {
        fs::remove_file(import_journal_path)?;
    }
    Ok(())
}

fn recover_pending_bundle_import(vault_root: &Path) -> Result<(), LamianError> {
    let vault_paths = db::resolve_vault_paths(vault_root);
    let import_journal_path = vault_paths.lamian_root.join(IMPORT_JOURNAL_FILE_NAME);
    let import_staging_root = vault_paths.lamian_root.join(IMPORT_STAGING_DIR_NAME);

    if !import_journal_path.exists() {
        return Ok(());
    }

    let journal = read_import_journal(&import_journal_path)?;
    let should_promote = match journal.status {
        BundleImportJournalStatus::Committed => true,
        BundleImportJournalStatus::Staged => {
            staged_journal_requires_promotion(vault_root, &journal.figure_ids)?
        }
    };

    if should_promote {
        for entry in &journal.staged_files {
            let staged_path = PathBuf::from(&entry.staged_path);
            let destination_path = PathBuf::from(&entry.destination_path);

            if destination_path.exists() {
                if staged_path.exists() {
                    fs::remove_file(&staged_path)?;
                }
                continue;
            }
            if !staged_path.exists() {
                return Err(LamianError::InvalidBundleValue {
                    field: "import_journal",
                    reason: "journal references missing staged file for promotion",
                    value: entry.bundle_path.clone(),
                });
            }

            if let Some(parent_directory) = destination_path.parent() {
                fs::create_dir_all(parent_directory)?;
            }
            move_file_with_fallback(&staged_path, &destination_path)?;
        }
    } else {
        for entry in &journal.staged_files {
            let _ = fs::remove_file(PathBuf::from(&entry.staged_path));
        }
    }

    remove_import_journal_if_exists(&import_journal_path)?;
    cleanup_empty_import_staging_root(&import_staging_root)?;
    Ok(())
}

fn staged_journal_requires_promotion(
    vault_root: &Path,
    figure_ids: &[String],
) -> Result<bool, LamianError> {
    if figure_ids.is_empty() {
        return Ok(false);
    }

    let connection = db::open_vault_connection(vault_root)?;
    let mut existing_figure_count = 0_usize;

    for figure_id in figure_ids {
        if figure_exists_in_connection(&connection, figure_id)? {
            existing_figure_count += 1;
        }
    }

    if existing_figure_count == 0 {
        return Ok(false);
    }
    if existing_figure_count == figure_ids.len() {
        return Ok(true);
    }

    Err(LamianError::InvalidBundleValue {
        field: "import_journal",
        reason: "journal indicates partial committed figure set; manual intervention required",
        value: format!(
            "{existing_figure_count}/{} figures present",
            figure_ids.len()
        ),
    })
}

fn cleanup_empty_import_staging_root(import_staging_root: &Path) -> Result<(), LamianError> {
    if !import_staging_root.exists() {
        return Ok(());
    }
    fs::remove_dir_all(import_staging_root)?;
    Ok(())
}

fn bundle_conflict_policy_name(value: BundleImportConflictPolicy) -> &'static str {
    match value {
        BundleImportConflictPolicy::Skip => "skip",
        BundleImportConflictPolicy::Error => "error",
        BundleImportConflictPolicy::Replace => "replace",
    }
}

fn figure_id_by_file_path(
    connection: &Connection,
    file_path: &Path,
) -> Result<Option<String>, LamianError> {
    connection
        .query_row(
            "SELECT figure_id FROM figures WHERE file_path = ?1",
            [file_path.to_string_lossy().to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(LamianError::from)
}

fn figure_exists_in_connection(
    connection: &rusqlite::Connection,
    figure_id: &str,
) -> Result<bool, LamianError> {
    let existing: Option<String> = connection
        .query_row(
            "SELECT figure_id FROM figures WHERE figure_id = ?1",
            [figure_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(existing.is_some())
}

fn figure_exists(transaction: &Transaction<'_>, figure_id: &str) -> Result<bool, LamianError> {
    let existing: Option<String> = transaction
        .query_row(
            "SELECT figure_id FROM figures WHERE figure_id = ?1",
            [figure_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(existing.is_some())
}

fn parent_tag_name(tag_name: &str) -> Option<String> {
    let (parent, _) = tag_name.rsplit_once(':')?;
    if parent.is_empty() {
        None
    } else {
        Some(parent.to_string())
    }
}

fn hash_and_size_for_file(path: &Path) -> Result<(String, u64), LamianError> {
    let mut file = File::open(path)?;
    hash_and_size_for_reader(&mut file, &path.display().to_string())
}

fn hash_and_size_for_reader(
    reader: &mut dyn Read,
    value_for_error: &str,
) -> Result<(String, u64), LamianError> {
    let mut hasher = Sha256::new();
    let mut total_size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
        total_size =
            total_size
                .checked_add(bytes_read as u64)
                .ok_or(LamianError::InvalidBundleValue {
                    field: "archive",
                    reason: "managed file size is too large to represent",
                    value: value_for_error.to_string(),
                })?;
    }

    Ok((format!("{:x}", hasher.finalize()), total_size))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        recover_pending_bundle_import, write_import_journal, BundleImportJournal,
        BundleImportJournalEntry, BundleImportJournalStatus, IMPORT_JOURNAL_FILE_NAME,
        IMPORT_STAGING_DIR_NAME,
    };
    use crate::db;

    #[test]
    fn recover_pending_bundle_import_promotes_committed_journal_files() {
        let temp_dir = tempfile::TempDir::new().expect("temp directory");
        let vault_root = temp_dir.path().join("vault");
        let vault_paths = db::initialize_vault(&vault_root).expect("initialize vault");

        let staged_path = vault_paths
            .lamian_root
            .join(IMPORT_STAGING_DIR_NAME)
            .join("session-a")
            .join("nested")
            .join("recovered.bin");
        let destination_path = vault_paths
            .lamian_root
            .join("figures")
            .join("nested")
            .join("recovered.bin");
        if let Some(parent_directory) = staged_path.parent() {
            fs::create_dir_all(parent_directory).expect("create staged parent");
        }
        fs::write(&staged_path, [1_u8, 2, 3, 4]).expect("write staged file");

        let journal = BundleImportJournal {
            status: BundleImportJournalStatus::Committed,
            figure_ids: Vec::new(),
            staged_files: vec![BundleImportJournalEntry {
                bundle_path: "files/nested/recovered.bin".to_string(),
                staged_path: staged_path.display().to_string(),
                destination_path: destination_path.display().to_string(),
            }],
        };
        let import_journal_path = vault_paths.lamian_root.join(IMPORT_JOURNAL_FILE_NAME);
        write_import_journal(&import_journal_path, &journal).expect("write import journal");

        recover_pending_bundle_import(&vault_root).expect("recover import journal");

        assert!(
            destination_path.exists(),
            "expected destination file after recovery: {}",
            destination_path.display()
        );
        assert!(
            !staged_path.exists(),
            "expected staged file cleanup after recovery: {}",
            staged_path.display()
        );
        assert!(
            !import_journal_path.exists(),
            "expected journal cleanup after recovery: {}",
            import_journal_path.display()
        );
    }

    #[test]
    fn recover_pending_bundle_import_discards_uncommitted_staged_files() {
        let temp_dir = tempfile::TempDir::new().expect("temp directory");
        let vault_root = temp_dir.path().join("vault");
        let vault_paths = db::initialize_vault(&vault_root).expect("initialize vault");

        let staged_path = vault_paths
            .lamian_root
            .join(IMPORT_STAGING_DIR_NAME)
            .join("session-b")
            .join("discard.bin");
        let destination_path = vault_paths.lamian_root.join("figures").join("discard.bin");
        if let Some(parent_directory) = staged_path.parent() {
            fs::create_dir_all(parent_directory).expect("create staged parent");
        }
        fs::write(&staged_path, [9_u8, 8, 7]).expect("write staged file");

        let journal = BundleImportJournal {
            status: BundleImportJournalStatus::Staged,
            figure_ids: vec!["missing-figure-id".to_string()],
            staged_files: vec![BundleImportJournalEntry {
                bundle_path: "files/discard.bin".to_string(),
                staged_path: staged_path.display().to_string(),
                destination_path: destination_path.display().to_string(),
            }],
        };
        let import_journal_path = vault_paths.lamian_root.join(IMPORT_JOURNAL_FILE_NAME);
        write_import_journal(&import_journal_path, &journal).expect("write import journal");

        recover_pending_bundle_import(&vault_root).expect("recover import journal");

        assert!(
            !destination_path.exists(),
            "destination should not be promoted for uncommitted staged journal"
        );
        assert!(
            !staged_path.exists(),
            "staged file should be removed for uncommitted staged journal"
        );
        assert!(
            !import_journal_path.exists(),
            "journal should be removed after staged cleanup"
        );
    }

    #[test]
    fn recover_pending_bundle_import_promotes_staged_files_when_figures_exist() {
        let temp_dir = tempfile::TempDir::new().expect("temp directory");
        let vault_root = temp_dir.path().join("vault");
        let vault_paths = db::initialize_vault(&vault_root).expect("initialize vault");
        let connection = db::open_vault_connection(&vault_root).expect("open vault connection");

        connection
            .execute(
                r#"
INSERT INTO figures (
    figure_id,
    display_name,
    file_path,
    file_hash_sha256,
    media_type,
    file_size_bytes,
    created_at,
    updated_at
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
"#,
                rusqlite::params![
                    "figure-1",
                    "Existing Figure",
                    "/tmp/managed.png",
                    "hash-figure-1",
                    "image/png",
                    42_i64,
                    "2026-02-21T00:00:00Z",
                    "2026-02-21T00:00:00Z"
                ],
            )
            .expect("insert figure row");

        let staged_path = vault_paths
            .lamian_root
            .join(IMPORT_STAGING_DIR_NAME)
            .join("session-c")
            .join("promote.bin");
        let destination_path = vault_paths.lamian_root.join("figures").join("promote.bin");
        if let Some(parent_directory) = staged_path.parent() {
            fs::create_dir_all(parent_directory).expect("create staged parent");
        }
        fs::write(&staged_path, [5_u8, 4, 3]).expect("write staged file");

        let journal = BundleImportJournal {
            status: BundleImportJournalStatus::Staged,
            figure_ids: vec!["figure-1".to_string()],
            staged_files: vec![BundleImportJournalEntry {
                bundle_path: "files/promote.bin".to_string(),
                staged_path: staged_path.display().to_string(),
                destination_path: destination_path.display().to_string(),
            }],
        };
        let import_journal_path = vault_paths.lamian_root.join(IMPORT_JOURNAL_FILE_NAME);
        write_import_journal(&import_journal_path, &journal).expect("write import journal");

        recover_pending_bundle_import(&vault_root).expect("recover import journal");

        assert!(
            destination_path.exists(),
            "expected destination promotion for staged journal with committed figures"
        );
        assert!(
            !staged_path.exists(),
            "staged file should be removed after promotion"
        );
        assert!(
            !import_journal_path.exists(),
            "journal should be removed after promotion"
        );
    }
}

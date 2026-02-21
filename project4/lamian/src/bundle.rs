use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::read::GzDecoder;
use flate2::{Compression, GzBuilder};
use rusqlite::{params, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tar::{Archive, Builder, Header, HeaderMode};

use crate::cli::ExportFormat;
use crate::db;
use crate::error::LamianError;
use crate::export::{export_metadata, ExportRequest};

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
}

#[derive(Debug, Clone, Serialize)]
pub struct BundleImportResult {
    pub bundle_path: String,
    pub schema_version: i64,
    pub total_figures: usize,
    pub imported_figures: usize,
    pub skipped_existing_figures: usize,
    pub managed_files_written: usize,
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
    content: Vec<u8>,
}

#[derive(Debug)]
struct PendingManagedFileWrite {
    bundle_path: String,
    destination_path: PathBuf,
}

#[derive(Debug, Clone)]
struct StagedManagedFile {
    bundle_path: String,
    staged_path: PathBuf,
    destination_path: PathBuf,
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

    let archive_entries = read_bundle_archive(&request.bundle_path)?;
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
    verify_managed_file_entries(&manifest, &metadata_document, &archive_entries.file_entries)?;

    let mut connection = db::open_vault_connection(&request.vault_root)?;

    let managed_files_by_figure_id = index_managed_files_by_figure_id(&manifest)?;
    let managed_figure_root = vault_paths.lamian_root.join(MANAGED_FIGURES_DIR_NAME);
    let import_staging_root = vault_paths.lamian_root.join(IMPORT_STAGING_DIR_NAME);
    let import_journal_path = vault_paths.lamian_root.join(IMPORT_JOURNAL_FILE_NAME);
    let total_figures = metadata_document.figures.len();

    let mut imported_figures = 0_usize;
    let mut skipped_existing_figures = 0_usize;
    let mut managed_files_written = 0_usize;
    let mut imported_figure_ids = Vec::new();

    let transaction = connection.transaction()?;
    let mut pending_file_writes = Vec::new();
    let mut pending_links = Vec::new();

    for figure in &metadata_document.figures {
        if figure_exists(&transaction, &figure.figure_id)? {
            skipped_existing_figures += 1;
            continue;
        }

        let persisted_file_path =
            if let Some(managed_entry) = managed_files_by_figure_id.get(&figure.figure_id) {
                let destination_path =
                    build_destination_path_for_managed_file(&managed_figure_root, managed_entry)?;
                if destination_path.exists() {
                    skipped_existing_figures += 1;
                    continue;
                }
                pending_file_writes.push(PendingManagedFileWrite {
                    bundle_path: managed_entry.bundle_path.clone(),
                    destination_path: destination_path.clone(),
                });
                destination_path
            } else {
                PathBuf::from(&figure.file_path)
            };

        insert_figure_record(&transaction, figure, &persisted_file_path)?;
        insert_source_records(&transaction, figure)?;
        insert_tag_records(&transaction, figure)?;
        insert_note_record(&transaction, figure)?;

        for outbound_link in &figure.outbound_links {
            pending_links.push(PendingLinkInsert {
                from_figure_id: figure.figure_id.clone(),
                to_figure_id: outbound_link.to_figure_id.clone(),
                relation_type: outbound_link.relation_type.clone(),
                created_at: outbound_link.created_at.clone(),
            });
        }

        imported_figures += 1;
        imported_figure_ids.push(figure.figure_id.clone());
    }

    insert_link_records(&transaction, &pending_links)?;

    let stage_session_root = import_staging_root.join(build_import_session_id());
    let staged_files = stage_managed_files(
        &pending_file_writes,
        &archive_entries.file_entries,
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
        schema_version: metadata_document.schema_version,
        total_figures,
        imported_figures,
        skipped_existing_figures,
        managed_files_written,
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

        let content = fs::read(&file_path)?;
        let file_size_bytes =
            u64::try_from(content.len()).map_err(|_| LamianError::InvalidBundleValue {
                field: "managed_file",
                reason: "managed file is too large to represent in bundle metadata",
                value: file_path.display().to_string(),
            })?;
        let file_hash_sha256 = sha256_bytes(&content);
        managed_file_entries.push(ManagedFileExportEntry {
            manifest_entry: ManagedFileManifestEntry {
                figure_id: figure.figure_id.clone(),
                bundle_path,
                file_hash_sha256,
                file_size_bytes,
            },
            content,
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
        append_archive_entry(
            &mut archive,
            &managed_file_entry.manifest_entry.bundle_path,
            &managed_file_entry.content,
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

struct ArchiveReadResult {
    manifest_bytes: Option<Vec<u8>>,
    metadata_bytes: Option<Vec<u8>>,
    file_entries: BTreeMap<String, Vec<u8>>,
}

fn read_bundle_archive(path: &Path) -> Result<ArchiveReadResult, LamianError> {
    let source_file = File::open(path)?;
    let gzip_decoder = GzDecoder::new(source_file);
    let mut archive = Archive::new(gzip_decoder);

    let mut manifest_bytes = None;
    let mut metadata_bytes = None;
    let mut file_entries = BTreeMap::new();

    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let raw_path = entry.path()?;
        let path_string = normalize_archive_entry_path(raw_path.as_ref());
        let mut content = Vec::new();
        entry.read_to_end(&mut content)?;

        if path_string == MANIFEST_ENTRY_PATH {
            manifest_bytes = Some(content);
            continue;
        }
        if path_string == METADATA_ENTRY_PATH {
            metadata_bytes = Some(content);
            continue;
        }
        if path_string.starts_with(MANAGED_FILES_PREFIX)
            && file_entries.insert(path_string.clone(), content).is_some()
        {
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
        file_entries,
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
    manifest: &BundleManifest,
    metadata_document: &BundleMetadataDocument,
    archive_file_entries: &BTreeMap<String, Vec<u8>>,
) -> Result<(), LamianError> {
    let mut known_figure_ids = HashSet::new();
    for figure in &metadata_document.figures {
        known_figure_ids.insert(figure.figure_id.clone());
    }

    let mut observed_figure_ids = HashSet::new();
    let mut observed_bundle_paths = HashSet::new();

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
        let content = archive_file_entries
            .get(&managed_file_entry.bundle_path)
            .ok_or(LamianError::InvalidBundleValue {
                field: "archive",
                reason: "managed file entry listed in manifest is missing from archive",
                value: managed_file_entry.bundle_path.clone(),
            })?;

        let actual_hash = sha256_bytes(content);
        if actual_hash != managed_file_entry.file_hash_sha256 {
            return Err(LamianError::BundleChecksumMismatch {
                entry_path: managed_file_entry.bundle_path.clone(),
                expected: managed_file_entry.file_hash_sha256.clone(),
                actual: actual_hash,
            });
        }

        let actual_size =
            u64::try_from(content.len()).map_err(|_| LamianError::InvalidBundleValue {
                field: "archive",
                reason: "managed file size is too large to represent",
                value: managed_file_entry.bundle_path.clone(),
            })?;
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

    Ok(())
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

fn insert_source_records(
    transaction: &Transaction<'_>,
    figure: &BundleFigure,
) -> Result<(), LamianError> {
    for source in &figure.sources {
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
                source.source_type,
                source.source_key,
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
        let parent_tag_name = parent_tag_name(tag_name);
        transaction.execute(
            "INSERT OR IGNORE INTO tags (tag_name, tag_parent) VALUES (?1, ?2)",
            params![tag_name, parent_tag_name],
        )?;
        let tag_id: i64 = transaction.query_row(
            "SELECT tag_id FROM tags WHERE tag_name = ?1",
            [tag_name],
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
) -> Result<(), LamianError> {
    for pending_link in pending_links {
        if !figure_exists(transaction, &pending_link.to_figure_id)? {
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
    }
    Ok(())
}

fn build_import_session_id() -> String {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("session-{}-{now_nanos}", std::process::id())
}

fn stage_managed_files(
    pending_file_writes: &[PendingManagedFileWrite],
    archive_file_entries: &BTreeMap<String, Vec<u8>>,
    managed_figure_root: &Path,
    stage_session_root: &Path,
) -> Result<Vec<StagedManagedFile>, LamianError> {
    let mut staged_files = Vec::new();

    for pending_file_write in pending_file_writes {
        let Some(content) = archive_file_entries.get(&pending_file_write.bundle_path) else {
            cleanup_staged_files(&staged_files);
            return Err(LamianError::InvalidBundleValue {
                field: "archive",
                reason: "managed file entry listed in manifest is missing from archive",
                value: pending_file_write.bundle_path.clone(),
            });
        };

        let relative_destination = pending_file_write
            .destination_path
            .strip_prefix(managed_figure_root)
            .map_err(|_| LamianError::InvalidBundleValue {
                field: "managed_file",
                reason: "managed file destination must be within .lamian/figures",
                value: pending_file_write.destination_path.display().to_string(),
            })?;
        let staged_path = stage_session_root.join(relative_destination);

        if let Some(parent_directory) = staged_path.parent() {
            fs::create_dir_all(parent_directory)?;
        }
        if let Err(error) = fs::write(&staged_path, content) {
            cleanup_staged_files(&staged_files);
            return Err(error.into());
        }

        staged_files.push(StagedManagedFile {
            bundle_path: pending_file_write.bundle_path.clone(),
            staged_path,
            destination_path: pending_file_write.destination_path.clone(),
        });
    }

    Ok(staged_files)
}

fn promote_staged_managed_files(
    staged_files: &[StagedManagedFile],
    managed_files_written: &mut usize,
) -> Result<(), LamianError> {
    for staged_file in staged_files {
        if staged_file.destination_path.exists() {
            if staged_file.staged_path.exists() {
                fs::remove_file(&staged_file.staged_path)?;
            }
            continue;
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

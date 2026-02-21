use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

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
    let total_figures = metadata_document.figures.len();

    let mut imported_figures = 0_usize;
    let mut skipped_existing_figures = 0_usize;
    let mut managed_files_written = 0_usize;
    let mut written_paths = Vec::new();

    let transaction_result = (|| -> Result<(), LamianError> {
        let transaction = connection.transaction()?;
        let mut pending_file_writes = Vec::new();
        let mut pending_links = Vec::new();

        for figure in &metadata_document.figures {
            if figure_exists(&transaction, &figure.figure_id)? {
                skipped_existing_figures += 1;
                continue;
            }

            let persisted_file_path = if let Some(managed_entry) =
                managed_files_by_figure_id.get(&figure.figure_id)
            {
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
        }

        insert_link_records(&transaction, &pending_links)?;
        write_managed_files(
            &pending_file_writes,
            &archive_entries.file_entries,
            &mut written_paths,
            &mut managed_files_written,
        )?;
        transaction.commit()?;
        Ok(())
    })();

    if let Err(error) = transaction_result {
        cleanup_written_files(&written_paths);
        return Err(error);
    }

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

fn write_managed_files(
    pending_file_writes: &[PendingManagedFileWrite],
    archive_file_entries: &BTreeMap<String, Vec<u8>>,
    written_paths: &mut Vec<PathBuf>,
    managed_files_written: &mut usize,
) -> Result<(), LamianError> {
    for pending_file_write in pending_file_writes {
        let Some(content) = archive_file_entries.get(&pending_file_write.bundle_path) else {
            return Err(LamianError::InvalidBundleValue {
                field: "archive",
                reason: "managed file entry listed in manifest is missing from archive",
                value: pending_file_write.bundle_path.clone(),
            });
        };

        if let Some(parent_directory) = pending_file_write.destination_path.parent() {
            fs::create_dir_all(parent_directory)?;
        }
        fs::write(&pending_file_write.destination_path, content)?;
        written_paths.push(pending_file_write.destination_path.clone());
        *managed_files_written += 1;
    }
    Ok(())
}

fn cleanup_written_files(paths: &[PathBuf]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
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

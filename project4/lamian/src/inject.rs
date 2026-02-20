use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};

use crate::db;
use crate::error::LamianError;

const FIGURE_STORE_DIR_NAME: &str = "figures";

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SourceType {
    Doi,
    Url,
    Local,
    Manual,
}

impl SourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Doi => "doi",
            Self::Url => "url",
            Self::Local => "local",
            Self::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CopyMode {
    Copy,
    Reference,
}

#[derive(Debug, Clone)]
pub struct InjectRequest {
    pub vault_root: PathBuf,
    pub file_path: PathBuf,
    pub source_type: SourceType,
    pub source_key: String,
    pub copy_mode: CopyMode,
}

#[derive(Debug, Clone)]
pub struct InjectResult {
    pub figure_id: String,
}

pub fn inject_figure(request: InjectRequest) -> Result<InjectResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let source_key = validate_provenance(request.source_type, &request.source_key)?;
    let input_file_path = canonicalize_input_file_path(&request.file_path)?;
    let display_name = display_name_from_path(&input_file_path)?;
    let media_type = detect_media_type(&input_file_path)?;
    let file_size_bytes = fs::metadata(&input_file_path)?.len();
    let file_hash_sha256 = sha256_file(&input_file_path)?;
    let figure_id = build_figure_id(&file_hash_sha256, request.source_type, &source_key);

    let vault_paths = db::resolve_vault_paths(&request.vault_root);
    if !vault_paths.database_path.exists() {
        return Err(LamianError::VaultNotInitialized {
            vault_root: request.vault_root,
        });
    }

    let mut connection = Connection::open(&vault_paths.database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    if figure_exists(&connection, &figure_id)? {
        return Ok(InjectResult { figure_id });
    }

    let persisted_file_path = match request.copy_mode {
        CopyMode::Reference => input_file_path.clone(),
        CopyMode::Copy => {
            copy_file_into_vault(&vault_paths.lamian_root, &figure_id, &input_file_path)?
        }
    };
    let persisted_file_path_string = persisted_file_path.to_string_lossy().into_owned();

    let transaction_result = (|| -> Result<(), LamianError> {
        let transaction = connection.transaction()?;

        transaction.execute(
            r#"
INSERT INTO figures (
    figure_id,
    display_name,
    file_path,
    file_hash_sha256,
    media_type,
    file_size_bytes
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6)
"#,
            params![
                &figure_id,
                &display_name,
                &persisted_file_path_string,
                &file_hash_sha256,
                &media_type,
                file_size_bytes
            ],
        )?;

        transaction.execute(
            r#"
INSERT INTO sources (
    figure_id,
    source_type,
    source_key
)
VALUES (?1, ?2, ?3)
"#,
            params![&figure_id, request.source_type.as_str(), &source_key],
        )?;

        transaction.commit()?;
        Ok(())
    })();

    if let Err(error) = transaction_result {
        if matches!(request.copy_mode, CopyMode::Copy) {
            let _ = fs::remove_file(persisted_file_path);
        }
        return Err(error);
    }

    Ok(InjectResult { figure_id })
}

fn canonicalize_input_file_path(path: &Path) -> Result<PathBuf, LamianError> {
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
    Ok(path.canonicalize()?)
}

fn display_name_from_path(path: &Path) -> Result<String, LamianError> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(std::string::ToString::to_string)
        .ok_or_else(|| LamianError::InvalidInputFileName {
            path: path.to_path_buf(),
        })
}

fn detect_media_type(path: &Path) -> Result<String, LamianError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);

    let media_type = match extension.as_deref() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("tif") | Some("tiff") => "image/tiff",
        Some("svg") => "image/svg+xml",
        _ => {
            return Err(LamianError::UnsupportedMediaType {
                path: path.to_path_buf(),
            });
        }
    };

    Ok(media_type.to_string())
}

fn validate_provenance(source_type: SourceType, source_key: &str) -> Result<String, LamianError> {
    let normalized_source_key = source_key.trim();
    if normalized_source_key.is_empty() {
        return Err(LamianError::MissingProvenanceField {
            field: "source_key",
        });
    }

    match source_type {
        SourceType::Doi => {
            if !is_valid_doi(normalized_source_key) {
                return Err(LamianError::InvalidProvenanceValue {
                    field: "source_key",
                    reason: "DOI must start with `10.` and include `/`",
                    value: normalized_source_key.to_string(),
                });
            }
        }
        SourceType::Url => {
            if !is_valid_url(normalized_source_key) {
                return Err(LamianError::InvalidProvenanceValue {
                    field: "source_key",
                    reason: "URL must start with `http://` or `https://`",
                    value: normalized_source_key.to_string(),
                });
            }
        }
        SourceType::Local | SourceType::Manual => {}
    }

    Ok(normalized_source_key.to_string())
}

fn is_valid_doi(value: &str) -> bool {
    value.starts_with("10.") && value.contains('/')
}

fn is_valid_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn build_figure_id(file_hash_sha256: &str, source_type: SourceType, source_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(file_hash_sha256.as_bytes());
    hasher.update(b"|");
    hasher.update(source_type.as_str().as_bytes());
    hasher.update(b"|");
    hasher.update(source_key.as_bytes());
    let digest = hasher.finalize();
    format!("fig_{digest:x}")
}

fn sha256_file(path: &Path) -> Result<String, LamianError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8 * 1024];

    loop {
        let read_count = file.read(&mut buffer)?;
        if read_count == 0 {
            break;
        }
        hasher.update(&buffer[..read_count]);
    }

    let digest = hasher.finalize();
    Ok(format!("{digest:x}"))
}

fn copy_file_into_vault(
    lamian_root: &Path,
    figure_id: &str,
    source_file_path: &Path,
) -> Result<PathBuf, LamianError> {
    let destination_dir = lamian_root.join(FIGURE_STORE_DIR_NAME);
    fs::create_dir_all(&destination_dir)?;

    let extension = source_file_path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .map(|value| format!(".{value}"))
        .unwrap_or_default();

    let destination_path = destination_dir.join(format!("{figure_id}{extension}"));
    fs::copy(source_file_path, &destination_path)?;
    Ok(destination_path)
}

fn figure_exists(connection: &Connection, figure_id: &str) -> Result<bool, LamianError> {
    let existing: Option<String> = connection
        .query_row(
            "SELECT figure_id FROM figures WHERE figure_id = ?1",
            [figure_id],
            |row| row.get(0),
        )
        .optional()?;

    Ok(existing.is_some())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rusqlite::Connection;
    use tempfile::TempDir;

    use super::{CopyMode, InjectRequest, SourceType, inject_figure};
    use crate::db;
    use crate::error::LamianError;

    #[test]
    fn inject_reference_mode_persists_figure_and_source_with_stable_figure_id() {
        let temp_dir = TempDir::new().expect("temp directory");
        let vault_path = temp_dir.path().join("vault");
        let vault_paths = db::initialize_vault(&vault_path).expect("initialize vault");
        let input_file_path = temp_dir.path().join("example.png");
        std::fs::write(&input_file_path, b"sample image").expect("write sample file");

        let request = InjectRequest {
            vault_root: vault_path.clone(),
            file_path: input_file_path.clone(),
            source_type: SourceType::Doi,
            source_key: "10.1234/lamian.2026.001".to_string(),
            copy_mode: CopyMode::Reference,
        };

        let first_result = inject_figure(request.clone()).expect("first inject");
        let second_result = inject_figure(request).expect("second inject");
        assert_eq!(first_result.figure_id, second_result.figure_id);

        let connection = Connection::open(vault_paths.database_path).expect("open sqlite db");
        let figure_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM figures WHERE figure_id = ?1",
                [first_result.figure_id.as_str()],
                |row| row.get(0),
            )
            .expect("query figure count");
        let source_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sources WHERE figure_id = ?1",
                [first_result.figure_id.as_str()],
                |row| row.get(0),
            )
            .expect("query source count");
        assert_eq!(figure_count, 1);
        assert_eq!(source_count, 1);
    }

    #[test]
    fn inject_copy_mode_copies_file_into_vault_storage() {
        let temp_dir = TempDir::new().expect("temp directory");
        let vault_path = temp_dir.path().join("vault");
        let vault_paths = db::initialize_vault(&vault_path).expect("initialize vault");
        let input_file_path = temp_dir.path().join("copy_source.jpg");
        std::fs::write(&input_file_path, b"sample image").expect("write sample file");

        let result = inject_figure(InjectRequest {
            vault_root: vault_path,
            file_path: input_file_path,
            source_type: SourceType::Url,
            source_key: "https://example.com/paper/figure-1".to_string(),
            copy_mode: CopyMode::Copy,
        })
        .expect("inject in copy mode");

        let connection = Connection::open(vault_paths.database_path).expect("open sqlite db");
        let stored_path: String = connection
            .query_row(
                "SELECT file_path FROM figures WHERE figure_id = ?1",
                [result.figure_id.as_str()],
                |row| row.get(0),
            )
            .expect("query stored file path");
        let stored_path = PathBuf::from(stored_path);

        assert!(stored_path.starts_with(vault_paths.lamian_root.join("figures")));
        assert!(stored_path.exists());
    }

    #[test]
    fn inject_rejects_empty_source_key() {
        let temp_dir = TempDir::new().expect("temp directory");
        let vault_path = temp_dir.path().join("vault");
        db::initialize_vault(&vault_path).expect("initialize vault");
        let input_file_path = temp_dir.path().join("example.png");
        std::fs::write(&input_file_path, b"sample image").expect("write sample file");

        let error = inject_figure(InjectRequest {
            vault_root: vault_path,
            file_path: input_file_path,
            source_type: SourceType::Manual,
            source_key: "   ".to_string(),
            copy_mode: CopyMode::Reference,
        })
        .expect_err("inject should fail");

        assert!(matches!(
            error,
            LamianError::MissingProvenanceField {
                field: "source_key"
            }
        ));
    }

    #[test]
    fn inject_rejects_invalid_doi_value() {
        let temp_dir = TempDir::new().expect("temp directory");
        let vault_path = temp_dir.path().join("vault");
        db::initialize_vault(&vault_path).expect("initialize vault");
        let input_file_path = temp_dir.path().join("example.png");
        std::fs::write(&input_file_path, b"sample image").expect("write sample file");

        let error = inject_figure(InjectRequest {
            vault_root: vault_path,
            file_path: input_file_path,
            source_type: SourceType::Doi,
            source_key: "doi:123".to_string(),
            copy_mode: CopyMode::Reference,
        })
        .expect_err("inject should fail");

        assert!(matches!(error, LamianError::InvalidProvenanceValue { .. }));
    }
}

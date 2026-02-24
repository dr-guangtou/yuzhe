use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::db;
use crate::error::LamianError;

#[derive(Debug, Clone)]
pub struct VerifyRequest {
    pub vault_root: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyResult {
    pub issue_count: usize,
    pub issues: Vec<VerifyIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyIssue {
    pub kind: VerifyIssueKind,
    pub subject: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifyIssueKind {
    MissingFile,
    HashDrift,
    SizeDrift,
}

pub fn verify_vault(request: VerifyRequest) -> Result<VerifyResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let connection = db::open_vault_connection(&request.vault_root)?;
    let issues = collect_issues(&request.vault_root, &connection)?;

    Ok(VerifyResult {
        issue_count: issues.len(),
        issues,
    })
}

fn collect_issues(
    vault_root: &Path,
    connection: &Connection,
) -> Result<Vec<VerifyIssue>, LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT figure_id, file_path, file_hash_sha256, file_size_bytes
FROM figures
ORDER BY figure_id ASC
"#,
    )?;
    let mut rows = statement.query([])?;
    let mut issues = Vec::new();

    while let Some(row) = rows.next()? {
        let figure_id: String = row.get(0)?;
        let file_path_value: String = row.get(1)?;
        let expected_hash: String = row.get(2)?;
        let expected_size_i64: i64 = row.get(3)?;
        let expected_size =
            u64::try_from(expected_size_i64).map_err(|_| LamianError::InvalidVerifyValue {
                field: "file_size_bytes",
                reason: "must be a non-negative integer",
                value: expected_size_i64.to_string(),
            })?;

        let file_path = resolve_figure_file_path(vault_root, &file_path_value);
        let metadata = match std::fs::metadata(&file_path) {
            Ok(metadata) => metadata,
            Err(_) => {
                issues.push(VerifyIssue {
                    kind: VerifyIssueKind::MissingFile,
                    subject: format!("figure:{figure_id}"),
                    detail: format!("figure file is missing on disk: `{}`", file_path.display()),
                });
                continue;
            }
        };

        if !metadata.is_file() {
            issues.push(VerifyIssue {
                kind: VerifyIssueKind::MissingFile,
                subject: format!("figure:{figure_id}"),
                detail: format!(
                    "figure path is not a regular file on disk: `{}`",
                    file_path.display()
                ),
            });
            continue;
        }

        let actual_size = metadata.len();
        if actual_size != expected_size {
            issues.push(VerifyIssue {
                kind: VerifyIssueKind::SizeDrift,
                subject: format!("figure:{figure_id}"),
                detail: format!(
                    "file size drift detected for `{}` (expected {}, actual {})",
                    file_path.display(),
                    expected_size,
                    actual_size
                ),
            });
        }

        let actual_hash = sha256_file(&file_path)?;
        if actual_hash != expected_hash {
            issues.push(VerifyIssue {
                kind: VerifyIssueKind::HashDrift,
                subject: format!("figure:{figure_id}"),
                detail: format!(
                    "file hash drift detected for `{}` (expected {}, actual {})",
                    file_path.display(),
                    expected_hash,
                    actual_hash
                ),
            });
        }
    }

    Ok(issues)
}

fn resolve_figure_file_path(vault_root: &Path, file_path_value: &str) -> PathBuf {
    let path = PathBuf::from(file_path_value);
    if path.is_absolute() {
        path
    } else {
        vault_root.join(path)
    }
}

fn sha256_file(path: &Path) -> Result<String, LamianError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8 * 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

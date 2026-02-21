use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::LamianError;
use crate::inject::{
    inject_figure, inspect_inject, CopyMode, InjectRequest, InspectInjectRequest, SourceType,
};

const TEMPLATE_PLACEHOLDER_FILE_NAME: &str = "file_name";
const TEMPLATE_PLACEHOLDER_FILE_STEM: &str = "file_stem";
const TEMPLATE_PLACEHOLDER_EXTENSION: &str = "extension";
const TEMPLATE_PLACEHOLDER_RELATIVE_PATH: &str = "relative_path";

#[derive(Debug, Clone)]
pub struct ImportRequest {
    pub vault_root: PathBuf,
    pub input_path: PathBuf,
    pub source_type: SourceType,
    pub source_key_template: String,
    pub copy_mode: CopyMode,
    pub recursive: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub processed: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub items: Vec<ImportItemResult>,
}

impl ImportResult {
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportItemResult {
    pub input_path: String,
    pub source_key: Option<String>,
    pub figure_id: Option<String>,
    pub status: ImportItemStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportItemStatus {
    Imported,
    DryRun,
    SkippedDuplicate,
    Failed,
}

#[derive(Debug, Clone)]
struct TemplateContext {
    file_name: String,
    file_stem: String,
    extension: String,
    relative_path: String,
}

pub fn import_batch(request: ImportRequest) -> Result<ImportResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let source_key_template = normalize_source_key_template(&request.source_key_template)?;
    let discovery = collect_input_files(&request.input_path, request.recursive)?;
    let mut items = Vec::new();

    let mut processed = 0_usize;
    let mut succeeded = 0_usize;
    let mut failed = 0_usize;
    let mut skipped = 0_usize;
    let mut observed_figure_ids = HashSet::new();

    for file_path in discovery.files {
        processed += 1;

        let template_context =
            match build_template_context(&file_path, discovery.base_directory.as_deref()) {
                Ok(context) => context,
                Err(error) => {
                    failed += 1;
                    items.push(ImportItemResult {
                        input_path: file_path.display().to_string(),
                        source_key: None,
                        figure_id: None,
                        status: ImportItemStatus::Failed,
                        error: Some(error.to_string()),
                    });
                    continue;
                }
            };

        let source_key = match render_source_key_template(&source_key_template, &template_context) {
            Ok(value) => value,
            Err(error) => {
                failed += 1;
                items.push(ImportItemResult {
                    input_path: file_path.display().to_string(),
                    source_key: None,
                    figure_id: None,
                    status: ImportItemStatus::Failed,
                    error: Some(error.to_string()),
                });
                continue;
            }
        };

        if request.dry_run {
            match inspect_inject(InspectInjectRequest {
                vault_root: request.vault_root.clone(),
                file_path: file_path.clone(),
                source_type: request.source_type,
                source_key: source_key.clone(),
            }) {
                Ok(result) => {
                    let is_duplicate_in_batch =
                        !observed_figure_ids.insert(result.figure_id.clone());
                    if result.duplicate_exists || is_duplicate_in_batch {
                        skipped += 1;
                        items.push(ImportItemResult {
                            input_path: result.input_file_path.display().to_string(),
                            source_key: Some(result.source_key),
                            figure_id: Some(result.figure_id),
                            status: ImportItemStatus::SkippedDuplicate,
                            error: None,
                        });
                    } else {
                        succeeded += 1;
                        items.push(ImportItemResult {
                            input_path: result.input_file_path.display().to_string(),
                            source_key: Some(result.source_key),
                            figure_id: Some(result.figure_id),
                            status: ImportItemStatus::DryRun,
                            error: None,
                        });
                    }
                }
                Err(error) => {
                    failed += 1;
                    items.push(ImportItemResult {
                        input_path: file_path.display().to_string(),
                        source_key: Some(source_key),
                        figure_id: None,
                        status: ImportItemStatus::Failed,
                        error: Some(error.to_string()),
                    });
                }
            }
            continue;
        }

        let output_input_path = file_path.display().to_string();
        match inject_figure(InjectRequest {
            vault_root: request.vault_root.clone(),
            file_path,
            source_type: request.source_type,
            source_key: source_key.clone(),
            copy_mode: request.copy_mode,
        }) {
            Ok(result) => {
                if result.created_new {
                    succeeded += 1;
                    items.push(ImportItemResult {
                        input_path: result.persisted_input_path.display().to_string(),
                        source_key: Some(source_key),
                        figure_id: Some(result.figure_id),
                        status: ImportItemStatus::Imported,
                        error: None,
                    });
                } else {
                    skipped += 1;
                    items.push(ImportItemResult {
                        input_path: result.persisted_input_path.display().to_string(),
                        source_key: Some(source_key),
                        figure_id: Some(result.figure_id),
                        status: ImportItemStatus::SkippedDuplicate,
                        error: None,
                    });
                }
            }
            Err(error) => {
                failed += 1;
                items.push(ImportItemResult {
                    input_path: output_input_path,
                    source_key: Some(source_key),
                    figure_id: None,
                    status: ImportItemStatus::Failed,
                    error: Some(error.to_string()),
                });
            }
        }
    }

    Ok(ImportResult {
        processed,
        succeeded,
        failed,
        skipped,
        items,
    })
}

struct DiscoveryResult {
    files: Vec<PathBuf>,
    base_directory: Option<PathBuf>,
}

fn collect_input_files(input_path: &Path, recursive: bool) -> Result<DiscoveryResult, LamianError> {
    if !input_path.exists() {
        return Err(LamianError::InputFileNotFound {
            path: input_path.to_path_buf(),
        });
    }

    if input_path.is_file() {
        return Ok(DiscoveryResult {
            files: vec![input_path.to_path_buf()],
            base_directory: None,
        });
    }

    if !input_path.is_dir() {
        return Err(LamianError::InvalidImportValue {
            field: "input_path",
            reason: "input path must be a file or directory",
            value: input_path.display().to_string(),
        });
    }

    let mut files = Vec::new();

    if recursive {
        let mut pending_directories = vec![input_path.to_path_buf()];
        while let Some(directory) = pending_directories.pop() {
            let mut directory_entries = read_directory_paths(&directory)?;
            directory_entries.sort();

            for path in directory_entries {
                if path.is_file() {
                    files.push(path);
                } else if path.is_dir() {
                    pending_directories.push(path);
                }
            }
        }
    } else {
        let mut directory_entries = read_directory_paths(input_path)?;
        directory_entries.sort();
        for path in directory_entries {
            if path.is_file() {
                files.push(path);
            }
        }
    }

    files.sort();
    Ok(DiscoveryResult {
        files,
        base_directory: Some(input_path.to_path_buf()),
    })
}

fn read_directory_paths(path: &Path) -> Result<Vec<PathBuf>, LamianError> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        paths.push(entry.path());
    }
    Ok(paths)
}

fn normalize_source_key_template(source_key_template: &str) -> Result<String, LamianError> {
    let trimmed = source_key_template.trim();
    if trimmed.is_empty() {
        return Err(LamianError::MissingImportField {
            field: "source_key_template",
        });
    }

    validate_template_placeholders(trimmed)?;
    Ok(trimmed.to_string())
}

fn validate_template_placeholders(template: &str) -> Result<(), LamianError> {
    let mut is_inside_placeholder = false;
    let mut placeholder_start = 0_usize;

    for (index, character) in template.char_indices() {
        if character == '{' {
            if is_inside_placeholder {
                return Err(LamianError::InvalidImportValue {
                    field: "source_key_template",
                    reason: "nested placeholders are not allowed",
                    value: template.to_string(),
                });
            }
            is_inside_placeholder = true;
            placeholder_start = index + 1;
            continue;
        }

        if character == '}' {
            if !is_inside_placeholder {
                return Err(LamianError::InvalidImportValue {
                    field: "source_key_template",
                    reason: "unmatched closing brace in template",
                    value: template.to_string(),
                });
            }

            let placeholder = &template[placeholder_start..index];
            if !is_supported_placeholder(placeholder) {
                return Err(LamianError::InvalidImportValue {
                    field: "source_key_template",
                    reason: "template includes unsupported placeholder",
                    value: placeholder.to_string(),
                });
            }
            is_inside_placeholder = false;
        }
    }

    if is_inside_placeholder {
        return Err(LamianError::InvalidImportValue {
            field: "source_key_template",
            reason: "unmatched opening brace in template",
            value: template.to_string(),
        });
    }

    Ok(())
}

fn is_supported_placeholder(placeholder: &str) -> bool {
    matches!(
        placeholder,
        TEMPLATE_PLACEHOLDER_FILE_NAME
            | TEMPLATE_PLACEHOLDER_FILE_STEM
            | TEMPLATE_PLACEHOLDER_EXTENSION
            | TEMPLATE_PLACEHOLDER_RELATIVE_PATH
    )
}

fn build_template_context(
    file_path: &Path,
    base_directory: Option<&Path>,
) -> Result<TemplateContext, LamianError> {
    let file_name = file_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .ok_or_else(|| LamianError::InvalidInputFileName {
            path: file_path.to_path_buf(),
        })?;
    let file_stem = file_path
        .file_stem()
        .map(|name| name.to_string_lossy().to_string())
        .ok_or_else(|| LamianError::InvalidInputFileName {
            path: file_path.to_path_buf(),
        })?;
    let extension = file_path
        .extension()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default();

    let relative_path = if let Some(base_directory) = base_directory {
        file_path
            .strip_prefix(base_directory)
            .map_err(|_| LamianError::InvalidImportValue {
                field: "input_path",
                reason: "failed to compute relative path for template rendering",
                value: file_path.display().to_string(),
            })?
            .to_string_lossy()
            .replace('\\', "/")
    } else {
        file_name.clone()
    };

    Ok(TemplateContext {
        file_name,
        file_stem,
        extension,
        relative_path,
    })
}

fn render_source_key_template(
    template: &str,
    context: &TemplateContext,
) -> Result<String, LamianError> {
    let mut output = String::new();
    let mut is_inside_placeholder = false;
    let mut placeholder_start = 0_usize;

    for (index, character) in template.char_indices() {
        if character == '{' {
            if is_inside_placeholder {
                return Err(LamianError::InvalidImportValue {
                    field: "source_key_template",
                    reason: "nested placeholders are not allowed",
                    value: template.to_string(),
                });
            }
            is_inside_placeholder = true;
            placeholder_start = index + 1;
            continue;
        }

        if character == '}' {
            if !is_inside_placeholder {
                return Err(LamianError::InvalidImportValue {
                    field: "source_key_template",
                    reason: "unmatched closing brace in template",
                    value: template.to_string(),
                });
            }

            let placeholder = &template[placeholder_start..index];
            output.push_str(match placeholder {
                TEMPLATE_PLACEHOLDER_FILE_NAME => &context.file_name,
                TEMPLATE_PLACEHOLDER_FILE_STEM => &context.file_stem,
                TEMPLATE_PLACEHOLDER_EXTENSION => &context.extension,
                TEMPLATE_PLACEHOLDER_RELATIVE_PATH => &context.relative_path,
                _ => {
                    return Err(LamianError::InvalidImportValue {
                        field: "source_key_template",
                        reason: "template includes unsupported placeholder",
                        value: placeholder.to_string(),
                    });
                }
            });

            is_inside_placeholder = false;
            continue;
        }

        if !is_inside_placeholder {
            output.push(character);
        }
    }

    if is_inside_placeholder {
        return Err(LamianError::InvalidImportValue {
            field: "source_key_template",
            reason: "unmatched opening brace in template",
            value: template.to_string(),
        });
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{normalize_source_key_template, render_source_key_template, TemplateContext};

    #[test]
    fn normalize_source_key_template_rejects_unknown_placeholder() {
        let error = normalize_source_key_template("paper:{unknown}")
            .expect_err("template validation should fail");
        assert!(
            error.to_string().contains("unsupported placeholder"),
            "unexpected validation error: {error}"
        );
    }

    #[test]
    fn render_source_key_template_renders_all_supported_placeholders() {
        let context = TemplateContext {
            file_name: "figure-1.png".to_string(),
            file_stem: "figure-1".to_string(),
            extension: "png".to_string(),
            relative_path: "subdir/figure-1.png".to_string(),
        };
        let rendered = render_source_key_template(
            "paper:{relative_path}:{file_name}:{file_stem}:{extension}",
            &context,
        )
        .expect("render template");

        assert_eq!(
            rendered,
            "paper:subdir/figure-1.png:figure-1.png:figure-1:png"
        );
    }

    #[test]
    fn render_source_key_template_accepts_literal_template() {
        let context = TemplateContext {
            file_name: "figure-1.png".to_string(),
            file_stem: "figure-1".to_string(),
            extension: "png".to_string(),
            relative_path: "figure-1.png".to_string(),
        };
        let rendered = render_source_key_template("10.1234/example", &context)
            .expect("render literal template");

        assert_eq!(rendered, "10.1234/example");
    }

    #[test]
    fn render_source_key_template_rejects_unmatched_opening_brace() {
        let context = TemplateContext {
            file_name: "figure-1.png".to_string(),
            file_stem: "figure-1".to_string(),
            extension: "png".to_string(),
            relative_path: "figure-1.png".to_string(),
        };
        let error =
            render_source_key_template("paper:{file_name", &context).expect_err("should fail");
        assert!(
            error.to_string().contains("unmatched opening brace"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn source_key_template_accepts_non_empty_literal() {
        let normalized = normalize_source_key_template("10.1234/example")
            .expect("literal template should be accepted");
        assert_eq!(normalized, "10.1234/example");
    }

    #[test]
    fn source_key_template_rejects_empty_value() {
        let error = normalize_source_key_template("   ").expect_err("empty template should fail");
        assert!(
            error
                .to_string()
                .contains("missing required import field: source_key_template"),
            "unexpected error: {error}"
        );
    }
}

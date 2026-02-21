use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value as JsonValue;
use tempfile::TempDir;

#[test]
fn cli_import_succeeds_with_json_summary() {
    let fixture_png = repository_fixture_path("2602.17205_1.png");
    let fixture_jpg = repository_fixture_path("500px-Elliptical_galaxy_IC_2006.jpg");

    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let import_directory = temp_dir.path().join("import_batch");
    std::fs::create_dir_all(&import_directory).expect("create import directory");
    std::fs::copy(&fixture_png, import_directory.join("figure_a.png")).expect("copy png fixture");
    std::fs::copy(&fixture_jpg, import_directory.join("figure_b.jpg")).expect("copy jpg fixture");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let import_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "import",
        import_directory.to_string_lossy().as_ref(),
        "--source-type",
        "local",
        "--source-key-template",
        "batch:{relative_path}",
        "--copy-mode",
        "reference",
    ]);

    let import_json: JsonValue =
        serde_json::from_slice(&import_output.stdout).expect("parse import json");
    assert_eq!(import_json["command"].as_str(), Some("import"));
    assert_eq!(import_json["status"].as_str(), Some("ok"));
    assert_eq!(import_json["result"]["processed"].as_u64(), Some(2));
    assert_eq!(import_json["result"]["succeeded"].as_u64(), Some(2));
    assert_eq!(import_json["result"]["failed"].as_u64(), Some(0));
    assert_eq!(import_json["result"]["skipped"].as_u64(), Some(0));

    let items = import_json["result"]["items"]
        .as_array()
        .expect("import items");
    assert_eq!(items.len(), 2);
    for item in items {
        assert_eq!(item["status"].as_str(), Some("imported"));
        assert!(item["figure_id"].as_str().is_some());
    }
}

#[test]
fn cli_import_reports_duplicate_skip() {
    let fixture_png = repository_fixture_path("2602.17205_1.png");

    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let import_directory = temp_dir.path().join("import_batch");
    std::fs::create_dir_all(&import_directory).expect("create import directory");
    std::fs::copy(&fixture_png, import_directory.join("figure_a.png")).expect("copy png fixture");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "import",
        import_directory.to_string_lossy().as_ref(),
        "--source-type",
        "local",
        "--source-key-template",
        "batch:{relative_path}",
        "--copy-mode",
        "reference",
    ]);

    let second_import_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "import",
        import_directory.to_string_lossy().as_ref(),
        "--source-type",
        "local",
        "--source-key-template",
        "batch:{relative_path}",
        "--copy-mode",
        "reference",
    ]);

    let second_import_json: JsonValue =
        serde_json::from_slice(&second_import_output.stdout).expect("parse second import json");
    assert_eq!(second_import_json["status"].as_str(), Some("ok"));
    assert_eq!(second_import_json["result"]["processed"].as_u64(), Some(1));
    assert_eq!(second_import_json["result"]["succeeded"].as_u64(), Some(0));
    assert_eq!(second_import_json["result"]["failed"].as_u64(), Some(0));
    assert_eq!(second_import_json["result"]["skipped"].as_u64(), Some(1));
    assert_eq!(
        second_import_json["result"]["items"][0]["status"].as_str(),
        Some("skipped_duplicate")
    );
}

#[test]
fn cli_import_continues_on_item_error_and_returns_non_zero() {
    let fixture_png = repository_fixture_path("2602.17205_1.png");

    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let import_directory = temp_dir.path().join("import_batch");
    std::fs::create_dir_all(&import_directory).expect("create import directory");
    std::fs::copy(&fixture_png, import_directory.join("figure_a.png")).expect("copy png fixture");
    std::fs::write(import_directory.join("not_image.txt"), "plain text").expect("write txt file");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "import",
        import_directory.to_string_lossy().as_ref(),
        "--source-type",
        "local",
        "--source-key-template",
        "batch:{relative_path}",
        "--copy-mode",
        "reference",
    ]);

    assert!(
        !output.status.success(),
        "import unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("import completed with failures"),
        "unexpected stderr for partial failure:\n{stderr}"
    );

    let import_json: JsonValue = serde_json::from_slice(&output.stdout).expect("parse import json");
    assert_eq!(import_json["status"].as_str(), Some("partial_failure"));
    assert_eq!(import_json["result"]["processed"].as_u64(), Some(2));
    assert_eq!(import_json["result"]["succeeded"].as_u64(), Some(1));
    assert_eq!(import_json["result"]["failed"].as_u64(), Some(1));
    assert_eq!(import_json["result"]["skipped"].as_u64(), Some(0));

    let items = import_json["result"]["items"]
        .as_array()
        .expect("import items");
    assert_eq!(items.len(), 2);
    assert!(
        items
            .iter()
            .any(|item| item["status"].as_str() == Some("failed")
                && item["error"]
                    .as_str()
                    .is_some_and(|error| error.contains("unsupported media type"))),
        "expected one failed item with unsupported media type error"
    );
    assert!(
        items
            .iter()
            .any(|item| item["status"].as_str() == Some("imported")),
        "expected one imported item"
    );
}

#[test]
fn cli_import_dry_run_does_not_persist_figures() {
    let fixture_png = repository_fixture_path("2602.17205_1.png");

    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let import_directory = temp_dir.path().join("import_batch");
    std::fs::create_dir_all(&import_directory).expect("create import directory");
    std::fs::copy(&fixture_png, import_directory.join("figure_a.png")).expect("copy png fixture");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "import",
        import_directory.to_string_lossy().as_ref(),
        "--source-type",
        "local",
        "--source-key-template",
        "batch:{relative_path}",
        "--copy-mode",
        "reference",
        "--dry-run",
    ]);

    let import_json: JsonValue = serde_json::from_slice(&output.stdout).expect("parse import json");
    assert_eq!(import_json["status"].as_str(), Some("ok"));
    assert_eq!(import_json["result"]["processed"].as_u64(), Some(1));
    assert_eq!(import_json["result"]["succeeded"].as_u64(), Some(1));
    assert_eq!(import_json["result"]["failed"].as_u64(), Some(0));
    assert_eq!(import_json["result"]["skipped"].as_u64(), Some(0));
    assert_eq!(
        import_json["result"]["items"][0]["status"].as_str(),
        Some("dry_run")
    );

    let export_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "json",
    ]);
    let export_json: JsonValue =
        serde_json::from_slice(&export_output.stdout).expect("parse export json");
    assert_eq!(
        export_json["figures"].as_array().map(Vec::len),
        Some(0),
        "dry-run must not write figure rows"
    );
}

fn repository_fixture_path(file_name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(file_name);

    assert!(
        is_existing_file(&path),
        "missing fixture file: {}",
        path.display()
    );
    path
}

fn is_existing_file(path: &Path) -> bool {
    match std::fs::metadata(path) {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    }
}

fn run_lamian(arguments: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lamian"))
        .args(arguments)
        .output()
        .expect("execute lamian CLI")
}

fn run_lamian_and_assert_success(
    arguments: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
) -> Output {
    let output = run_lamian(arguments);
    assert!(
        output.status.success(),
        "lamian command failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

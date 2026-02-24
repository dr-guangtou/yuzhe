use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value as JsonValue;
use tempfile::TempDir;

#[test]
fn cli_verify_reports_clean_vault() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    inject_file_and_get_figure_id(&vault_path, &fixture_path, "copy");

    let output =
        run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "verify"]);
    let payload: JsonValue = serde_json::from_slice(&output.stdout).expect("parse verify json");

    assert_eq!(payload["command"].as_str(), Some("verify"));
    assert_eq!(payload["status"].as_str(), Some("ok"));
    assert_eq!(payload["result"]["issue_count"].as_u64(), Some(0));
}

#[test]
fn cli_verify_reports_missing_file() {
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let file_path = temp_dir.path().join("transient.png");

    std::fs::write(&file_path, b"fixture-bytes").expect("write transient file");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    inject_file_and_get_figure_id(&vault_path, &file_path, "reference");
    std::fs::remove_file(&file_path).expect("remove transient file");

    let output = run_lamian(["--vault", vault_path.to_string_lossy().as_ref(), "verify"]);
    assert!(
        !output.status.success(),
        "verify unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("verify found unresolved issue(s)"),
        "unexpected verify stderr:\n{stderr}"
    );

    let payload: JsonValue = serde_json::from_slice(&output.stdout).expect("parse verify json");
    assert_eq!(payload["command"].as_str(), Some("verify"));
    assert_eq!(payload["status"].as_str(), Some("issues_found"));
    assert_eq!(payload["result"]["issue_count"].as_u64(), Some(1));
    assert_eq!(
        payload["result"]["issues"][0]["kind"].as_str(),
        Some("missing_file")
    );
}

#[test]
fn cli_verify_reports_hash_drift_without_size_drift() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    inject_file_and_get_figure_id(&vault_path, &fixture_path, "copy");

    let managed_file_path = figure_file_path_from_export_json(&vault_path);
    let original_content = std::fs::read(&managed_file_path).expect("read managed file");
    let replacement_content = vec![0x58_u8; original_content.len()];
    std::fs::write(&managed_file_path, replacement_content).expect("overwrite managed file");

    let output = run_lamian(["--vault", vault_path.to_string_lossy().as_ref(), "verify"]);
    assert!(
        !output.status.success(),
        "verify unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: JsonValue = serde_json::from_slice(&output.stdout).expect("parse verify json");
    assert_eq!(payload["status"].as_str(), Some("issues_found"));
    assert!(issue_kinds(&payload).contains("hash_drift"));
    assert!(!issue_kinds(&payload).contains("size_drift"));
}

#[test]
fn cli_verify_reports_size_drift() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    inject_file_and_get_figure_id(&vault_path, &fixture_path, "copy");

    let managed_file_path = figure_file_path_from_export_json(&vault_path);
    let mut content = std::fs::read(&managed_file_path).expect("read managed file");
    content.push(0);
    std::fs::write(&managed_file_path, content).expect("append managed file");

    let output = run_lamian(["--vault", vault_path.to_string_lossy().as_ref(), "verify"]);
    assert!(
        !output.status.success(),
        "verify unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: JsonValue = serde_json::from_slice(&output.stdout).expect("parse verify json");
    assert_eq!(payload["status"].as_str(), Some("issues_found"));
    assert!(issue_kinds(&payload).contains("size_drift"));
}

fn issue_kinds(payload: &JsonValue) -> std::collections::HashSet<String> {
    payload["result"]["issues"]
        .as_array()
        .expect("issues array")
        .iter()
        .filter_map(|issue| issue["kind"].as_str().map(ToString::to_string))
        .collect()
}

fn figure_file_path_from_export_json(vault_path: &Path) -> PathBuf {
    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "json",
    ]);
    let payload: JsonValue = serde_json::from_slice(&output.stdout).expect("parse export json");
    let file_path_value = payload["figures"][0]["file_path"]
        .as_str()
        .expect("figure file path");
    PathBuf::from(file_path_value)
}

fn inject_file_and_get_figure_id(vault_path: &Path, file_path: &Path, copy_mode: &str) -> String {
    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "inject",
        file_path.to_string_lossy().as_ref(),
        "--source-type",
        "local",
        "--source-key",
        "batch:verify-fixture",
        "--copy-mode",
        copy_mode,
    ]);

    extract_figure_id_from_stdout(&output.stdout)
}

fn extract_figure_id_from_stdout(stdout: &[u8]) -> String {
    let output = String::from_utf8_lossy(stdout);
    let prefix = "Injected figure: ";

    for line in output.lines() {
        if let Some(figure_id) = line.strip_prefix(prefix) {
            return figure_id.trim().to_string();
        }
    }

    panic!("failed to parse figure id from stdout:\n{output}");
}

fn repository_fixture_path(file_name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(file_name);

    assert!(
        path.exists(),
        "missing fixture file: {}",
        path.to_string_lossy()
    );
    path
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

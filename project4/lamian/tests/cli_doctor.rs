use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rusqlite::Connection;
use serde_json::Value as JsonValue;
use tempfile::TempDir;

#[test]
fn cli_doctor_reports_clean_vault() {
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let output =
        run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "doctor"]);

    let payload: JsonValue = serde_json::from_slice(&output.stdout).expect("parse doctor json");
    assert_eq!(payload["command"].as_str(), Some("doctor"));
    assert_eq!(payload["status"].as_str(), Some("ok"));
    assert_eq!(
        payload["result"]["issue_count_before_fix"].as_u64(),
        Some(0)
    );
    assert_eq!(payload["result"]["unresolved_count"].as_u64(), Some(0));
}

#[test]
fn cli_doctor_detects_self_link_and_exits_non_zero() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id = inject_fixture_and_get_figure_id(&vault_path, &fixture_path);
    insert_self_link_issue(&vault_path, &figure_id);

    let output = run_lamian(["--vault", vault_path.to_string_lossy().as_ref(), "doctor"]);
    assert!(
        !output.status.success(),
        "doctor unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("doctor found unresolved issue(s)"),
        "unexpected doctor stderr:\n{stderr}"
    );

    let payload: JsonValue = serde_json::from_slice(&output.stdout).expect("parse doctor json");
    assert_eq!(payload["command"].as_str(), Some("doctor"));
    assert_eq!(payload["status"].as_str(), Some("issues_found"));
    assert_eq!(
        payload["result"]["issue_count_before_fix"].as_u64(),
        Some(1)
    );
    assert_eq!(payload["result"]["unresolved_count"].as_u64(), Some(1));
    assert_eq!(
        payload["result"]["issues_before_fix"][0]["kind"].as_str(),
        Some("self_link")
    );
    assert_eq!(
        payload["result"]["issues_before_fix"][0]["fixable"].as_bool(),
        Some(true)
    );
}

#[test]
fn cli_doctor_fix_removes_self_link_issue() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id = inject_fixture_and_get_figure_id(&vault_path, &fixture_path);
    insert_self_link_issue(&vault_path, &figure_id);

    let fix_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "doctor",
        "--fix",
    ]);
    let fix_payload: JsonValue =
        serde_json::from_slice(&fix_output.stdout).expect("parse doctor --fix json");

    assert_eq!(fix_payload["command"].as_str(), Some("doctor"));
    assert_eq!(fix_payload["status"].as_str(), Some("ok"));
    assert_eq!(fix_payload["result"]["fix_requested"].as_bool(), Some(true));
    assert_eq!(
        fix_payload["result"]["issue_count_before_fix"].as_u64(),
        Some(1)
    );
    assert_eq!(fix_payload["result"]["fixed_count"].as_u64(), Some(1));
    assert_eq!(fix_payload["result"]["unresolved_count"].as_u64(), Some(0));

    let verify_output =
        run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "doctor"]);
    let verify_payload: JsonValue =
        serde_json::from_slice(&verify_output.stdout).expect("parse verify doctor json");
    assert_eq!(verify_payload["status"].as_str(), Some("ok"));
    assert_eq!(
        verify_payload["result"]["issue_count_before_fix"].as_u64(),
        Some(0)
    );
}

#[test]
fn cli_doctor_detects_missing_figure_file_path() {
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let staged_file_path = temp_dir.path().join("transient.png");

    std::fs::write(&staged_file_path, b"sample image").expect("write transient file");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let _figure_id = inject_file_and_get_figure_id(&vault_path, &staged_file_path);

    std::fs::remove_file(&staged_file_path).expect("remove transient file");

    let output = run_lamian(["--vault", vault_path.to_string_lossy().as_ref(), "doctor"]);
    assert!(
        !output.status.success(),
        "doctor unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("doctor found unresolved issue(s)"),
        "unexpected doctor stderr:\n{stderr}"
    );

    let payload: JsonValue = serde_json::from_slice(&output.stdout).expect("parse doctor json");
    assert_eq!(payload["command"].as_str(), Some("doctor"));
    assert_eq!(payload["status"].as_str(), Some("issues_found"));
    assert_eq!(
        payload["result"]["issue_count_before_fix"].as_u64(),
        Some(1)
    );
    assert_eq!(payload["result"]["unresolved_count"].as_u64(), Some(1));
    assert_eq!(
        payload["result"]["issues_before_fix"][0]["kind"].as_str(),
        Some("figure_file_path_invalid")
    );
    assert_eq!(
        payload["result"]["issues_before_fix"][0]["fixable"].as_bool(),
        Some(false)
    );
}

fn insert_self_link_issue(vault_path: &Path, figure_id: &str) {
    let database_path = vault_database_path(vault_path);
    let connection = Connection::open(database_path).expect("open sqlite db");
    connection
        .execute(
            "INSERT INTO links (from_figure_id, to_figure_id, relation_type) VALUES (?1, ?1, 'related')",
            [figure_id],
        )
        .expect("insert self-link issue");
}

fn vault_database_path(vault_path: &Path) -> PathBuf {
    vault_path.join(".lamian").join("lamian.db")
}

fn inject_fixture_and_get_figure_id(vault_path: &Path, fixture_path: &Path) -> String {
    inject_file_and_get_figure_id(vault_path, fixture_path)
}

fn inject_file_and_get_figure_id(vault_path: &Path, file_path: &Path) -> String {
    let inject_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "inject",
        file_path.to_string_lossy().as_ref(),
        "--source-type",
        "local",
        "--source-key",
        "batch:doctor-fixture",
        "--copy-mode",
        "reference",
    ]);

    extract_figure_id_from_stdout(&inject_output.stdout)
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

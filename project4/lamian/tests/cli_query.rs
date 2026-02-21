use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value as JsonValue;
use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";
const URL_SOURCE_KEY: &str = "https://example.org/secondary-source";

#[test]
fn cli_query_save_list_run_and_delete() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let first_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);
    let second_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "url", URL_SOURCE_KEY);

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        first_figure_id.as_str(),
        "observatory:jwst",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        second_figure_id.as_str(),
        "galaxy:elliptical",
    ]);

    let save_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "query",
        "save",
        "jwst-only",
        "--tag",
        "observatory:jwst",
        "--sort",
        "updated-at",
        "--order",
        "desc",
        "--limit",
        "5",
    ]);
    let save_json: JsonValue =
        serde_json::from_slice(&save_output.stdout).expect("parse save json");
    assert_eq!(save_json["command"].as_str(), Some("query.save"));
    assert_eq!(save_json["status"].as_str(), Some("ok"));
    assert_eq!(
        save_json["result"]["query_name"].as_str(),
        Some("jwst-only")
    );

    let list_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "query",
        "list",
    ]);
    let list_json: JsonValue =
        serde_json::from_slice(&list_output.stdout).expect("parse list json");
    assert_eq!(list_json["command"].as_str(), Some("query.list"));
    assert_eq!(list_json["count"].as_u64(), Some(1));
    assert_eq!(
        list_json["queries"][0]["query_name"].as_str(),
        Some("jwst-only")
    );

    let run_ids_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "query",
        "run",
        "jwst-only",
        "--detail",
        "ids",
    ]);
    let run_ids_json: JsonValue =
        serde_json::from_slice(&run_ids_output.stdout).expect("parse run ids json");
    assert_eq!(run_ids_json["command"].as_str(), Some("query.run"));
    assert_eq!(run_ids_json["result"]["total_matches"].as_u64(), Some(1));

    let figure_ids = run_ids_json["result"]["figure_ids"]
        .as_array()
        .expect("figure ids array");
    assert_eq!(figure_ids.len(), 1);
    assert_eq!(figure_ids[0].as_str(), Some(first_figure_id.as_str()));
    assert_ne!(figure_ids[0].as_str(), Some(second_figure_id.as_str()));

    let run_full_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "query",
        "run",
        "jwst-only",
        "--detail",
        "full",
    ]);
    let run_full_json: JsonValue =
        serde_json::from_slice(&run_full_output.stdout).expect("parse run full json");
    let first_row = &run_full_json["result"]["figures"][0];
    assert_eq!(
        first_row["figure_id"].as_str(),
        Some(first_figure_id.as_str())
    );
    assert_eq!(first_row["tags"][0].as_str(), Some("observatory:jwst"));
    assert_eq!(first_row["source_keys"][0].as_str(), Some(DOI_SOURCE_KEY));

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "query",
        "delete",
        "jwst-only",
    ]);

    let list_after_delete_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "query",
        "list",
    ]);
    let list_after_delete_json: JsonValue =
        serde_json::from_slice(&list_after_delete_output.stdout).expect("parse list after delete");
    assert_eq!(list_after_delete_json["count"].as_u64(), Some(0));
}

#[test]
fn cli_query_save_rejects_empty_filters() {
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "query",
        "save",
        "empty-filters",
    ]);
    assert!(
        !output.status.success(),
        "query save unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing required query field: filters"),
        "unexpected stderr for empty filters:\n{stderr}"
    );
}

#[test]
fn cli_query_save_rejects_duplicate_name() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        figure_id.as_str(),
        "observatory:jwst",
    ]);

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "query",
        "save",
        "duplicate-name",
        "--tag",
        "observatory:jwst",
    ]);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "query",
        "save",
        "duplicate-name",
        "--tag",
        "observatory:jwst",
    ]);
    assert!(
        !output.status.success(),
        "duplicate query save unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("saved query already exists"),
        "unexpected stderr for duplicate name:\n{stderr}"
    );
}

fn inject_fixture_and_get_figure_id(
    vault_path: &Path,
    fixture_path: &Path,
    source_type: &str,
    source_key: &str,
) -> String {
    let inject_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "inject",
        fixture_path.to_string_lossy().as_ref(),
        "--source-type",
        source_type,
        "--source-key",
        source_key,
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

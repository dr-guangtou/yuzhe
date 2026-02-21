use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value as JsonValue;
use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";
const URL_SOURCE_KEY: &str = "https://example.org/secondary-source";

#[test]
fn cli_collection_static_lifecycle() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    let create_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "create",
        "my-static",
    ]);
    let create_json: JsonValue =
        serde_json::from_slice(&create_output.stdout).expect("parse collection create json");
    assert_eq!(create_json["command"].as_str(), Some("collection.create"));
    assert_eq!(
        create_json["result"]["collection_mode"].as_str(),
        Some("static")
    );
    assert_eq!(create_json["result"]["query_id"].as_i64(), None);

    let add_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "add",
        "my-static",
        figure_id.as_str(),
    ]);
    let add_json: JsonValue =
        serde_json::from_slice(&add_output.stdout).expect("parse collection add json");
    assert_eq!(add_json["command"].as_str(), Some("collection.add"));
    assert_eq!(add_json["result"]["created_relation"].as_bool(), Some(true));

    let add_duplicate_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "add",
        "my-static",
        figure_id.as_str(),
    ]);
    let add_duplicate_json: JsonValue =
        serde_json::from_slice(&add_duplicate_output.stdout).expect("parse duplicate add json");
    assert_eq!(
        add_duplicate_json["result"]["created_relation"].as_bool(),
        Some(false)
    );

    let list_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "list",
        "--collection",
        "my-static",
    ]);
    let list_json: JsonValue =
        serde_json::from_slice(&list_output.stdout).expect("parse collection list json");
    assert_eq!(list_json["command"].as_str(), Some("collection.list"));
    assert_eq!(list_json["count"].as_u64(), Some(1));
    assert_eq!(
        list_json["collections"][0]["collection_mode"].as_str(),
        Some("static")
    );
    assert_eq!(
        list_json["collections"][0]["figure_ids"][0].as_str(),
        Some(figure_id.as_str())
    );

    let remove_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "remove",
        "my-static",
        figure_id.as_str(),
    ]);
    let remove_json: JsonValue =
        serde_json::from_slice(&remove_output.stdout).expect("parse collection remove json");
    assert_eq!(remove_json["command"].as_str(), Some("collection.remove"));
    assert_eq!(remove_json["result"]["removed_count"].as_u64(), Some(1));

    let delete_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "delete",
        "my-static",
    ]);
    let delete_json: JsonValue =
        serde_json::from_slice(&delete_output.stdout).expect("parse collection delete json");
    assert_eq!(delete_json["command"].as_str(), Some("collection.delete"));
    assert_eq!(
        delete_json["result"]["collection_name"].as_str(),
        Some("my-static")
    );

    let list_after_delete = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "list",
    ]);
    let list_after_delete_json: JsonValue =
        serde_json::from_slice(&list_after_delete.stdout).expect("parse list after delete json");
    assert_eq!(list_after_delete_json["count"].as_u64(), Some(0));
}

#[test]
fn cli_collection_dynamic_list_uses_saved_query() {
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

    let query_save_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "query",
        "save",
        "jwst-only",
        "--tag",
        "observatory:jwst",
    ]);
    let query_save_json: JsonValue =
        serde_json::from_slice(&query_save_output.stdout).expect("parse query save json");
    let query_id = query_save_json["result"]["query_id"]
        .as_i64()
        .expect("query id");

    let create_dynamic_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "create",
        "jwst-dynamic",
        "--query-id",
        &query_id.to_string(),
    ]);
    let create_dynamic_json: JsonValue = serde_json::from_slice(&create_dynamic_output.stdout)
        .expect("parse dynamic collection create json");
    assert_eq!(
        create_dynamic_json["result"]["collection_mode"].as_str(),
        Some("dynamic")
    );
    assert_eq!(
        create_dynamic_json["result"]["query_id"].as_i64(),
        Some(query_id)
    );

    let list_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "list",
        "--collection",
        "jwst-dynamic",
    ]);
    let list_json: JsonValue =
        serde_json::from_slice(&list_output.stdout).expect("parse dynamic collection list json");
    assert_eq!(
        list_json["collections"][0]["collection_mode"].as_str(),
        Some("dynamic")
    );
    assert_eq!(
        list_json["collections"][0]["query_id"].as_i64(),
        Some(query_id)
    );
    let figure_ids = list_json["collections"][0]["figure_ids"]
        .as_array()
        .expect("dynamic collection figure ids");
    assert_eq!(figure_ids.len(), 1);
    assert_eq!(figure_ids[0].as_str(), Some(first_figure_id.as_str()));
    assert_ne!(figure_ids[0].as_str(), Some(second_figure_id.as_str()));

    let add_output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "add",
        "jwst-dynamic",
        first_figure_id.as_str(),
    ]);
    assert!(
        !add_output.status.success(),
        "collection add on dynamic unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&add_output.stdout),
        String::from_utf8_lossy(&add_output.stderr)
    );
    let add_stderr = String::from_utf8_lossy(&add_output.stderr);
    assert!(
        add_stderr.contains("collection mode conflict"),
        "unexpected collection add stderr:\n{add_stderr}"
    );
}

#[test]
fn cli_collection_create_rejects_missing_saved_query() {
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "create",
        "invalid-dynamic",
        "--query-id",
        "999",
    ]);
    assert!(
        !output.status.success(),
        "collection create unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("saved query does not exist"),
        "unexpected collection create stderr:\n{stderr}"
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

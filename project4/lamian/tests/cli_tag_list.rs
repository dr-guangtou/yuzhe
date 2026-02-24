use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value as JsonValue;
use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";
const URL_SOURCE_KEY: &str = "https://example.org/elliptical-galaxy";

#[test]
fn cli_tag_list_prints_sorted_tags_with_figure_counts() {
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
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        first_figure_id.as_str(),
        "shared",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        second_figure_id.as_str(),
        "shared",
    ]);

    let list_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "list",
    ]);
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        list_stdout.contains("Tags: 3"),
        "unexpected tag list header:\n{list_stdout}"
    );
    assert!(
        list_stdout.contains("galaxy:elliptical | figures=1"),
        "missing galaxy tag row:\n{list_stdout}"
    );
    assert!(
        list_stdout.contains("observatory:jwst | figures=1"),
        "missing observatory tag row:\n{list_stdout}"
    );
    assert!(
        list_stdout.contains("shared | figures=2"),
        "missing shared tag row:\n{list_stdout}"
    );

    let galaxy_index = list_stdout
        .find("galaxy:elliptical | figures=1")
        .expect("galaxy row");
    let observatory_index = list_stdout
        .find("observatory:jwst | figures=1")
        .expect("observatory row");
    let shared_index = list_stdout.find("shared | figures=2").expect("shared row");
    assert!(
        galaxy_index < observatory_index,
        "unexpected tag sort:\n{list_stdout}"
    );
    assert!(
        observatory_index < shared_index,
        "unexpected tag sort:\n{list_stdout}"
    );
}

#[test]
fn cli_tag_list_json_output_returns_rows() {
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

    let list_output = run_lamian_and_assert_success([
        "--json",
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "list",
    ]);
    let list_json: JsonValue = serde_json::from_slice(&list_output.stdout).expect("parse json");
    assert_eq!(list_json["command"].as_str(), Some("tag.list"));
    assert_eq!(list_json["status"].as_str(), Some("ok"));
    assert_eq!(list_json["count"].as_u64(), Some(1));
    assert_eq!(
        list_json["tags"][0]["tag_name"].as_str(),
        Some("observatory:jwst")
    );
    assert_eq!(list_json["tags"][0]["figure_count"].as_u64(), Some(1));
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

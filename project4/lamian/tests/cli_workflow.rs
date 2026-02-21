use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value as JsonValue;
use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";
const URL_SOURCE_KEY: &str = "https://example.org/workflow-source";

#[test]
fn cli_full_sequential_workflow_init_to_export() {
    let fixture_png = repository_fixture_path("2602.17205_1.png");
    let fixture_jpg = repository_fixture_path("500px-Elliptical_galaxy_IC_2006.jpg");

    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let note_file_path = temp_dir.path().join("workflow-note.md");
    std::fs::write(&note_file_path, "full workflow note").expect("write workflow note");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let first_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_png, "doi", DOI_SOURCE_KEY);
    let second_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_jpg, "url", URL_SOURCE_KEY);

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        first_figure_id.as_str(),
        "--name",
        "Workflow Figure 1",
        "--caption",
        "workflow caption",
        "--note-file",
        note_file_path.to_string_lossy().as_ref(),
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        first_figure_id.as_str(),
        "workflow:jwst",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "link",
        "add",
        first_figure_id.as_str(),
        second_figure_id.as_str(),
        "--relation",
        "derived_from",
    ]);

    let search_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "search",
        "--tag",
        "workflow:jwst",
    ]);
    let search_stdout = String::from_utf8_lossy(&search_output.stdout);
    assert!(
        search_stdout.contains("Search results: 1"),
        "unexpected search stdout:\n{search_stdout}"
    );
    assert!(
        search_stdout.contains(first_figure_id.as_str()),
        "search output does not include tagged figure id.\nstdout:\n{search_stdout}"
    );
    assert!(
        !search_stdout.contains(second_figure_id.as_str()),
        "search output unexpectedly includes untagged figure id.\nstdout:\n{search_stdout}"
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
    assert_eq!(export_json["schema_version"].as_i64(), Some(5));
    assert_eq!(export_json["figures"].as_array().map(Vec::len), Some(2));

    let figures = export_json["figures"].as_array().expect("figures array");
    let first_figure = figures
        .iter()
        .find(|figure| figure["figure_id"].as_str() == Some(first_figure_id.as_str()))
        .expect("first figure in export");
    assert_eq!(
        first_figure["display_name"].as_str(),
        Some("Workflow Figure 1")
    );
    assert_eq!(first_figure["caption"].as_str(), Some("workflow caption"));
    assert_eq!(first_figure["tags"].as_array().map(Vec::len), Some(1));
    assert_eq!(first_figure["tags"][0].as_str(), Some("workflow:jwst"));
    assert_eq!(
        first_figure["note"]["note_markdown"].as_str(),
        Some("full workflow note")
    );
    assert_eq!(
        first_figure["outbound_links"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        first_figure["outbound_links"][0]["to_figure_id"].as_str(),
        Some(second_figure_id.as_str())
    );
    assert_eq!(
        first_figure["outbound_links"][0]["relation_type"].as_str(),
        Some("derived_from")
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

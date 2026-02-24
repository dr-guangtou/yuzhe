use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value as JsonValue;
use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";
const URL_SOURCE_KEY: &str = "https://example.org/elliptical-galaxy";

#[test]
fn cli_phase1_commands_support_global_json_output() {
    let first_fixture_path = repository_fixture_path("2602.17205_1.png");
    let second_fixture_path = repository_fixture_path("500px-Elliptical_galaxy_IC_2006.jpg");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let inject_first_json = run_lamian_and_parse_json([
        "--json",
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "inject",
        first_fixture_path.to_string_lossy().as_ref(),
        "--source-type",
        "doi",
        "--source-key",
        DOI_SOURCE_KEY,
        "--copy-mode",
        "reference",
    ]);
    assert_eq!(inject_first_json["command"].as_str(), Some("inject"));
    assert_eq!(inject_first_json["status"].as_str(), Some("ok"));
    let first_figure_id = inject_first_json["result"]["figure_id"]
        .as_str()
        .expect("first figure id")
        .to_string();

    let inject_second_json = run_lamian_and_parse_json([
        "--json",
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "inject",
        second_fixture_path.to_string_lossy().as_ref(),
        "--source-type",
        "url",
        "--source-key",
        URL_SOURCE_KEY,
        "--copy-mode",
        "reference",
    ]);
    assert_eq!(inject_second_json["command"].as_str(), Some("inject"));
    assert_eq!(inject_second_json["status"].as_str(), Some("ok"));
    let second_figure_id = inject_second_json["result"]["figure_id"]
        .as_str()
        .expect("second figure id")
        .to_string();

    let update_json = run_lamian_and_parse_json([
        "--json",
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        first_figure_id.as_str(),
        "--name",
        "JWST Panel 1",
        "--caption",
        "NIRCam composite",
    ]);
    assert_eq!(update_json["command"].as_str(), Some("update"));
    assert_eq!(update_json["status"].as_str(), Some("ok"));
    assert_eq!(
        update_json["result"]["figure_id"].as_str(),
        Some(first_figure_id.as_str())
    );

    let tag_add_json = run_lamian_and_parse_json([
        "--json",
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        first_figure_id.as_str(),
        "observatory:jwst",
    ]);
    assert_eq!(tag_add_json["command"].as_str(), Some("tag.add"));
    assert_eq!(tag_add_json["status"].as_str(), Some("ok"));
    assert_eq!(
        tag_add_json["result"]["normalized_tag"].as_str(),
        Some("observatory:jwst")
    );

    let tag_rename_json = run_lamian_and_parse_json([
        "--json",
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "rename",
        "observatory:jwst",
        "observatory:webb",
    ]);
    assert_eq!(tag_rename_json["command"].as_str(), Some("tag.rename"));
    assert_eq!(tag_rename_json["status"].as_str(), Some("ok"));
    assert_eq!(tag_rename_json["result"]["renamed_count"].as_u64(), Some(1));

    let tag_remove_json = run_lamian_and_parse_json([
        "--json",
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "remove",
        first_figure_id.as_str(),
        "observatory:webb",
    ]);
    assert_eq!(tag_remove_json["command"].as_str(), Some("tag.remove"));
    assert_eq!(tag_remove_json["status"].as_str(), Some("ok"));
    assert_eq!(
        tag_remove_json["result"]["removed_relation"].as_bool(),
        Some(true)
    );

    let link_add_json = run_lamian_and_parse_json([
        "--json",
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "link",
        "add",
        first_figure_id.as_str(),
        second_figure_id.as_str(),
        "--relation",
        "related",
    ]);
    assert_eq!(link_add_json["command"].as_str(), Some("link.add"));
    assert_eq!(link_add_json["status"].as_str(), Some("ok"));
    assert_eq!(
        link_add_json["result"]["created_link"].as_bool(),
        Some(true)
    );

    let link_remove_json = run_lamian_and_parse_json([
        "--json",
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "link",
        "remove",
        first_figure_id.as_str(),
        second_figure_id.as_str(),
    ]);
    assert_eq!(link_remove_json["command"].as_str(), Some("link.remove"));
    assert_eq!(link_remove_json["status"].as_str(), Some("ok"));
    assert_eq!(
        link_remove_json["result"]["removed_count"].as_u64(),
        Some(1)
    );

    let search_json = run_lamian_and_parse_json([
        "--json",
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "search",
        "--text",
        "nircam composite",
    ]);
    assert_eq!(search_json["command"].as_str(), Some("search"));
    assert_eq!(search_json["status"].as_str(), Some("ok"));
    assert_eq!(search_json["count"].as_u64(), Some(1));
    assert_eq!(
        search_json["result"]["figures"][0]["figure_id"].as_str(),
        Some(first_figure_id.as_str())
    );

    let export_target_path = temp_dir.path().join("snapshot.yaml");
    let export_json = run_lamian_and_parse_json([
        "--json",
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "yaml",
        "--target",
        export_target_path.to_string_lossy().as_ref(),
    ]);
    assert_eq!(export_json["command"].as_str(), Some("export"));
    assert_eq!(export_json["status"].as_str(), Some("ok"));
    assert_eq!(export_json["result"]["figure_count"].as_u64(), Some(2));
    assert_eq!(
        export_json["result"]["target_path"].as_str(),
        Some(export_target_path.to_string_lossy().as_ref())
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

fn run_lamian_and_parse_json(
    arguments: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
) -> JsonValue {
    let output = run_lamian_and_assert_success(arguments);
    serde_json::from_slice(&output.stdout).expect("parse lamian json output")
}

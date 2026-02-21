use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";

#[test]
fn cli_export_json_writes_target_file_with_expected_fields() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let note_file_path = temp_dir.path().join("note.md");
    std::fs::write(&note_file_path, "figure note line").expect("write note file");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        figure_id.as_str(),
        "--name",
        "JWST Panel 1",
        "--caption",
        "NIRCam composite",
        "--note-file",
        note_file_path.to_string_lossy().as_ref(),
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        figure_id.as_str(),
        "observatory:jwst",
    ]);

    let export_path = temp_dir.path().join("exports").join("snapshot.json");
    let export_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "json",
        "--target",
        export_path.to_string_lossy().as_ref(),
    ]);
    let export_stdout = String::from_utf8_lossy(&export_output.stdout);
    assert!(
        export_stdout.contains("Exported metadata: 1 figures"),
        "unexpected export stdout:\n{export_stdout}"
    );

    let export_content = std::fs::read_to_string(&export_path).expect("read export json");
    let parsed: JsonValue = serde_json::from_str(&export_content).expect("parse export json");

    assert_eq!(parsed["schema_version"].as_i64(), Some(3));
    assert_eq!(parsed["figures"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        parsed["figures"][0]["figure_id"].as_str(),
        Some(figure_id.as_str())
    );
    assert_eq!(
        parsed["figures"][0]["display_name"].as_str(),
        Some("JWST Panel 1")
    );
    assert_eq!(
        parsed["figures"][0]["caption"].as_str(),
        Some("NIRCam composite")
    );
    assert_eq!(
        parsed["figures"][0]["tags"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        parsed["figures"][0]["tags"][0].as_str(),
        Some("observatory:jwst")
    );
    assert_eq!(
        parsed["figures"][0]["note"]["note_markdown"].as_str(),
        Some("figure note line")
    );
}

#[test]
fn cli_export_yaml_writes_target_file() {
    let fixture_path = repository_fixture_path("500px-Elliptical_galaxy_IC_2006.jpg");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    let export_path = temp_dir.path().join("snapshot.yaml");
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "yaml",
        "--target",
        export_path.to_string_lossy().as_ref(),
    ]);

    let export_content = std::fs::read_to_string(&export_path).expect("read export yaml");
    let parsed: YamlValue = serde_yaml::from_str(&export_content).expect("parse export yaml");
    assert_eq!(
        parsed["figures"].as_sequence().map(Vec::len),
        Some(1),
        "unexpected yaml export content:\n{export_content}"
    );
    assert_eq!(
        parsed["figures"][0]["figure_id"].as_str(),
        Some(figure_id.as_str())
    );
}

#[test]
fn cli_export_json_without_target_prints_payload_only() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "json",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Exported metadata"),
        "unexpected status line in stdout export:\n{stdout}"
    );

    let parsed: JsonValue = serde_json::from_str(&stdout).expect("parse stdout json");
    assert_eq!(parsed["figures"].as_array().map(Vec::len), Some(1));
}

#[test]
fn cli_export_rejects_directory_target_path() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "json",
        "--target",
        temp_dir.path().to_string_lossy().as_ref(),
    ]);

    assert!(
        !output.status.success(),
        "export unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid export value for target"),
        "unexpected stderr for directory target:\n{stderr}"
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

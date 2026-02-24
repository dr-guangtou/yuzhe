use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";
const URL_SOURCE_KEY: &str = "https://example.org/related";

#[test]
fn cli_show_prints_full_figure_metadata() {
    let first_fixture_path = repository_fixture_path("2602.17205_1.png");
    let second_fixture_path = repository_fixture_path("500px-Elliptical_galaxy_IC_2006.jpg");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let note_path = temp_dir.path().join("note.md");
    std::fs::write(&note_path, "note line").expect("write note");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let first_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &first_fixture_path, "doi", DOI_SOURCE_KEY);
    let second_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &second_fixture_path, "url", URL_SOURCE_KEY);

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        first_figure_id.as_str(),
        "--name",
        "JWST Panel 1",
        "--caption",
        "NIRCam composite",
        "--note-file",
        note_path.to_string_lossy().as_ref(),
    ]);
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
        "link",
        "add",
        first_figure_id.as_str(),
        second_figure_id.as_str(),
        "--relation",
        "related",
    ]);

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "show",
        first_figure_id.as_str(),
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains(&format!("Figure: {first_figure_id}")),
        "show output missing figure id:\n{stdout}"
    );
    assert!(
        stdout.contains("Display name: JWST Panel 1"),
        "show output missing display name:\n{stdout}"
    );
    assert!(
        stdout.contains("Caption: NIRCam composite"),
        "show output missing caption:\n{stdout}"
    );
    assert!(
        stdout.contains("Sources (1):"),
        "show output missing sources count:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("- doi | {DOI_SOURCE_KEY}")),
        "show output missing source row:\n{stdout}"
    );
    assert!(
        stdout.contains("Tags (1):"),
        "show output missing tags count:\n{stdout}"
    );
    assert!(
        stdout.contains("- observatory:jwst"),
        "show output missing tag:\n{stdout}"
    );
    assert!(
        stdout.contains("Outbound links (1):"),
        "show output missing outbound links count:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("- {second_figure_id} | relation=related")),
        "show output missing link row:\n{stdout}"
    );
    assert!(
        stdout.contains("Note markdown: note line"),
        "show output missing note content:\n{stdout}"
    );
}

#[test]
fn cli_info_alias_matches_show_command() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "info",
        figure_id.as_str(),
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("Figure: {figure_id}")),
        "info alias output missing figure id:\n{stdout}"
    );
}

#[test]
fn cli_show_fails_for_unknown_figure_id() {
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "show",
        "fig_missing",
    ]);
    assert!(
        !output.status.success(),
        "show unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown figure id: fig_missing"),
        "unexpected show stderr:\n{stderr}"
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

    assert!(path.exists(), "missing fixture file: {}", path.display());
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

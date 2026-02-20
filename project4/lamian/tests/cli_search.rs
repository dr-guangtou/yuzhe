use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";
const URL_SOURCE_KEY: &str = "https://en.wikipedia.org/wiki/Elliptical_galaxy";

#[test]
fn cli_search_filters_by_tag() {
    let (_temp_dir, vault_path, first_figure_id, second_figure_id) = seed_two_figures();

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "search",
        "--tag",
        "OBSERVATORY:JWST",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Search results: 1"),
        "unexpected tag search count:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("{first_figure_id} |")),
        "tag search did not include expected figure:\n{stdout}"
    );
    assert!(
        !stdout.contains(&format!("{second_figure_id} |")),
        "tag search unexpectedly included second figure:\n{stdout}"
    );
}

#[test]
fn cli_search_filters_by_source_key_case_insensitive() {
    let (_temp_dir, vault_path, first_figure_id, second_figure_id) = seed_two_figures();

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "search",
        "--source-key",
        "HTTPS://EN.WIKIPEDIA.ORG/WIKI/ELLIPTICAL_GALAXY",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Search results: 1"),
        "unexpected source-key search count:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("{second_figure_id} |")),
        "source-key search did not include expected figure:\n{stdout}"
    );
    assert!(
        !stdout.contains(&format!("{first_figure_id} |")),
        "source-key search unexpectedly included first figure:\n{stdout}"
    );
}

#[test]
fn cli_search_filters_by_text() {
    let (_temp_dir, vault_path, first_figure_id, second_figure_id) = seed_two_figures();

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "search",
        "--text",
        "ELLIPTICAL_GALAXY",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Search results: 1"),
        "unexpected text search count:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("{second_figure_id} |")),
        "text search did not include expected figure:\n{stdout}"
    );
    assert!(
        !stdout.contains(&format!("{first_figure_id} |")),
        "text search unexpectedly included first figure:\n{stdout}"
    );
}

#[test]
fn cli_search_prints_empty_result_message() {
    let (_temp_dir, vault_path, _first_figure_id, _second_figure_id) = seed_two_figures();

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "search",
        "--text",
        "no_match_phrase_2026_02_20",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Search results: 0"),
        "unexpected empty search count:\n{stdout}"
    );
    assert!(
        stdout.contains("No figures matched."),
        "missing empty-result message:\n{stdout}"
    );
}

fn seed_two_figures() -> (TempDir, PathBuf, String, String) {
    let first_fixture_path = repository_fixture_path("2602.17205_1.png");
    let second_fixture_path = repository_fixture_path("500px-Elliptical_galaxy_IC_2006.jpg");

    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let first_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &first_fixture_path, "doi", DOI_SOURCE_KEY);
    let second_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &second_fixture_path, "url", URL_SOURCE_KEY);

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

    (temp_dir, vault_path, first_figure_id, second_figure_id)
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

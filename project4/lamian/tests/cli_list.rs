use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";
const URL_SOURCE_KEY: &str = "https://en.wikipedia.org/wiki/Elliptical_galaxy";

#[test]
fn cli_list_prints_default_figure_id_order() {
    let (_temp_dir, vault_path, first_figure_id, second_figure_id) = seed_two_figures();

    let output =
        run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("List results: 2"),
        "unexpected list count:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("{first_figure_id} |")),
        "missing first figure row:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("{second_figure_id} |")),
        "missing second figure row:\n{stdout}"
    );
}

#[test]
fn cli_list_alias_ls_and_limit_work() {
    let (_temp_dir, vault_path, _first_figure_id, _second_figure_id) = seed_two_figures();

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "ls",
        "--limit",
        "1",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("List results: 1"),
        "unexpected limited list count:\n{stdout}"
    );
}

#[test]
fn cli_list_sort_by_display_name_desc() {
    let (_temp_dir, vault_path, first_figure_id, second_figure_id) = seed_two_figures();

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        first_figure_id.as_str(),
        "--name",
        "A Item",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        second_figure_id.as_str(),
        "--name",
        "Z Item",
    ]);

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "list",
        "--sort",
        "display-name",
        "--order",
        "desc",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let rows = extract_list_rows(&stdout);
    assert_eq!(rows.len(), 2, "unexpected row count:\n{stdout}");
    assert!(
        rows[0].contains(&second_figure_id),
        "expected descending name order first row to be second figure:\n{stdout}"
    );
    assert!(
        rows[1].contains(&first_figure_id),
        "expected descending name order second row to be first figure:\n{stdout}"
    );
}

#[test]
fn cli_list_rejects_zero_limit() {
    let (_temp_dir, vault_path, _first_figure_id, _second_figure_id) = seed_two_figures();

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "list",
        "--limit",
        "0",
    ]);
    assert!(
        !output.status.success(),
        "list unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid list value for limit"),
        "unexpected stderr for zero limit:\n{stderr}"
    );
}

fn extract_list_rows(stdout: &str) -> Vec<&str> {
    stdout
        .lines()
        .filter(|line| {
            line.contains(" | ")
                && !line.starts_with("List results:")
                && !line.starts_with("No figures found.")
        })
        .collect()
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

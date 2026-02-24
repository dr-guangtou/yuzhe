use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rusqlite::Connection;
use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";
const URL_SOURCE_KEY: &str = "https://en.wikipedia.org/wiki/Elliptical_galaxy";

#[test]
fn cli_delete_removes_figure_dependencies_and_managed_file() {
    let first_fixture_path = repository_fixture_path("500px-Elliptical_galaxy_IC_2006.jpg");
    let second_fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let note_file_path = temp_dir.path().join("note.md");
    std::fs::write(&note_file_path, "delete me note").expect("write note");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let first_figure_id = inject_fixture_and_get_figure_id(
        &vault_path,
        &first_fixture_path,
        "url",
        URL_SOURCE_KEY,
        "copy",
    );
    let second_figure_id = inject_fixture_and_get_figure_id(
        &vault_path,
        &second_fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "reference",
    );

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        first_figure_id.as_str(),
        "topic:unique",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        first_figure_id.as_str(),
        "topic:shared",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        second_figure_id.as_str(),
        "topic:shared",
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
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        first_figure_id.as_str(),
        "--note-file",
        note_file_path.to_string_lossy().as_ref(),
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "create",
        "cleanup-test",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "collection",
        "add",
        "cleanup-test",
        first_figure_id.as_str(),
    ]);

    let connection = open_connection(&vault_path);
    let managed_file_path = figure_path_for_id(&connection, first_figure_id.as_str());
    assert!(
        managed_file_path.exists(),
        "expected managed file to exist before delete"
    );

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "delete",
        first_figure_id.as_str(),
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("Deleted figure: {first_figure_id}")),
        "unexpected delete stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("Removed managed file: yes"),
        "expected managed file removal flag:\n{stdout}"
    );
    assert!(
        stdout.contains("Removed orphan tags: 1"),
        "expected orphan tag cleanup count:\n{stdout}"
    );
    assert!(
        !managed_file_path.exists(),
        "expected managed file to be removed by delete command"
    );

    let connection = open_connection(&vault_path);
    assert_eq!(
        count_rows(
            &connection,
            "SELECT COUNT(*) FROM figures WHERE figure_id = ?1",
            first_figure_id.as_str()
        ),
        0
    );
    assert_eq!(
        count_rows(
            &connection,
            "SELECT COUNT(*) FROM sources WHERE figure_id = ?1",
            first_figure_id.as_str()
        ),
        0
    );
    assert_eq!(
        count_rows(
            &connection,
            "SELECT COUNT(*) FROM notes WHERE figure_id = ?1",
            first_figure_id.as_str()
        ),
        0
    );
    assert_eq!(
        count_rows(
            &connection,
            "SELECT COUNT(*) FROM figure_tags WHERE figure_id = ?1",
            first_figure_id.as_str()
        ),
        0
    );
    assert_eq!(
        count_rows(
            &connection,
            "SELECT COUNT(*) FROM links WHERE from_figure_id = ?1 OR to_figure_id = ?1",
            first_figure_id.as_str()
        ),
        0
    );
    assert_eq!(
        count_rows(
            &connection,
            "SELECT COUNT(*) FROM collection_items WHERE figure_id = ?1",
            first_figure_id.as_str()
        ),
        0
    );

    assert_eq!(
        count_rows(
            &connection,
            "SELECT COUNT(*) FROM tags WHERE tag_name = ?1",
            "topic:unique"
        ),
        0
    );
    assert_eq!(
        count_rows(
            &connection,
            "SELECT COUNT(*) FROM tags WHERE tag_name = ?1",
            "topic:shared"
        ),
        1
    );
}

#[test]
fn cli_delete_keeps_reference_file_on_disk() {
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let external_file_path = temp_dir.path().join("external_reference.png");
    std::fs::write(&external_file_path, b"reference-bytes").expect("write external file");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id = inject_fixture_and_get_figure_id(
        &vault_path,
        &external_file_path,
        "local",
        "batch:external-reference",
        "reference",
    );

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "delete",
        figure_id.as_str(),
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Removed managed file: no"),
        "reference delete should not remove external file:\n{stdout}"
    );
    assert!(
        external_file_path.exists(),
        "external reference file should remain after delete"
    );
}

#[test]
fn cli_delete_rejects_unknown_figure_id() {
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "delete",
        "fig_missing",
    ]);
    assert!(
        !output.status.success(),
        "delete unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown figure id: fig_missing"),
        "unexpected stderr for unknown figure:\n{stderr}"
    );
}

fn open_connection(vault_path: &Path) -> Connection {
    Connection::open(vault_path.join(".lamian").join("lamian.db")).expect("open sqlite database")
}

fn count_rows(connection: &Connection, sql: &str, parameter: &str) -> i64 {
    connection
        .query_row(sql, [parameter], |row| row.get(0))
        .expect("count query")
}

fn figure_path_for_id(connection: &Connection, figure_id: &str) -> PathBuf {
    let file_path_value: String = connection
        .query_row(
            "SELECT file_path FROM figures WHERE figure_id = ?1",
            [figure_id],
            |row| row.get(0),
        )
        .expect("query figure file path");
    PathBuf::from(file_path_value)
}

fn inject_fixture_and_get_figure_id(
    vault_path: &Path,
    fixture_path: &Path,
    source_type: &str,
    source_key: &str,
    copy_mode: &str,
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
        copy_mode,
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

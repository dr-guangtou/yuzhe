use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rusqlite::Connection;
use tempfile::TempDir;

const DOI_KEY: &str = "10.1126/science.ady9404";

#[test]
fn cli_tag_add_persists_normalized_tag_after_inject() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let inject_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "inject",
        fixture_path.to_string_lossy().as_ref(),
        "--source-type",
        "doi",
        "--source-key",
        DOI_KEY,
        "--copy-mode",
        "reference",
    ]);

    let figure_id = extract_figure_id_from_stdout(&inject_output.stdout);

    let tag_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        figure_id.as_str(),
        "JWST:Machine_Learning",
    ]);
    let tag_stdout = String::from_utf8_lossy(&tag_output.stdout);
    assert!(
        tag_stdout.contains("Added tag: jwst:machine_learning"),
        "unexpected tag add output:\n{tag_stdout}"
    );

    let connection = Connection::open(vault_path.join(".lamian").join("lamian.db"))
        .expect("open sqlite database");

    let (tag_parent, tag_count): (Option<String>, i64) = connection
        .query_row(
            "SELECT tag_parent, COUNT(*) FROM tags WHERE tag_name = ?1",
            ["jwst:machine_learning"],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query tag table");
    assert_eq!(tag_parent, Some("jwst".to_string()));
    assert_eq!(tag_count, 1);

    let assignment_count: i64 = connection
        .query_row(
            r#"
SELECT COUNT(*)
FROM figure_tags
JOIN tags ON tags.tag_id = figure_tags.tag_id
WHERE figure_tags.figure_id = ?1 AND tags.tag_name = ?2
"#,
            [figure_id.as_str(), "jwst:machine_learning"],
            |row| row.get(0),
        )
        .expect("query tag assignment");
    assert_eq!(assignment_count, 1);
}

#[test]
fn cli_tag_add_is_idempotent_for_duplicate_assignment() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let inject_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "inject",
        fixture_path.to_string_lossy().as_ref(),
        "--source-type",
        "doi",
        "--source-key",
        DOI_KEY,
        "--copy-mode",
        "reference",
    ]);
    let figure_id = extract_figure_id_from_stdout(&inject_output.stdout);

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        figure_id.as_str(),
        "jwst",
    ]);
    let duplicate_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        figure_id.as_str(),
        "JWST",
    ]);

    let duplicate_stdout = String::from_utf8_lossy(&duplicate_output.stdout);
    assert!(
        duplicate_stdout.contains("Tag already assigned: jwst"),
        "unexpected duplicate output:\n{duplicate_stdout}"
    );

    let connection = Connection::open(vault_path.join(".lamian").join("lamian.db"))
        .expect("open sqlite database");
    let assignment_count: i64 = connection
        .query_row(
            r#"
SELECT COUNT(*)
FROM figure_tags
JOIN tags ON tags.tag_id = figure_tags.tag_id
WHERE figure_tags.figure_id = ?1 AND tags.tag_name = ?2
"#,
            [figure_id.as_str(), "jwst"],
            |row| row.get(0),
        )
        .expect("query tag assignment");
    assert_eq!(assignment_count, 1);
}

#[test]
fn cli_tag_add_rejects_invalid_tag_value() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let inject_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "inject",
        fixture_path.to_string_lossy().as_ref(),
        "--source-type",
        "doi",
        "--source-key",
        DOI_KEY,
        "--copy-mode",
        "reference",
    ]);
    let figure_id = extract_figure_id_from_stdout(&inject_output.stdout);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        figure_id.as_str(),
        "jwst::ml",
    ]);

    assert!(
        !output.status.success(),
        "tag add unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid tag value"),
        "unexpected stderr for invalid tag value:\n{stderr}"
    );
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

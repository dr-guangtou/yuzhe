use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rusqlite::Connection;
use tempfile::TempDir;

const DOI_KEY: &str = "10.1126/science.ady9404";
const URL_SOURCE_KEY: &str = "https://example.org/secondary-source";

#[test]
fn cli_tag_remove_deletes_assignment_and_orphan_tag() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let figure_id = inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_KEY);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        figure_id.as_str(),
        "jwst",
    ]);

    let remove_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "remove",
        figure_id.as_str(),
        "jwst",
    ]);

    let remove_stdout = String::from_utf8_lossy(&remove_output.stdout);
    assert!(
        remove_stdout.contains("Removed tag: jwst"),
        "unexpected remove output:\n{remove_stdout}"
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
        .expect("query removed assignment");
    assert_eq!(assignment_count, 0);

    let tag_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM tags WHERE tag_name = ?1",
            ["jwst"],
            |row| row.get(0),
        )
        .expect("query orphan tag cleanup");
    assert_eq!(tag_count, 0);
}

#[test]
fn cli_tag_remove_rejects_unassigned_tag_on_existing_tag_row() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let first_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_KEY);
    let second_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "url", URL_SOURCE_KEY);

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        first_figure_id.as_str(),
        "jwst",
    ]);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "remove",
        second_figure_id.as_str(),
        "jwst",
    ]);

    assert!(
        !output.status.success(),
        "tag remove unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("is not assigned"),
        "unexpected stderr for unassigned removal:\n{stderr}"
    );
}

#[test]
fn cli_tag_rename_updates_hierarchy_prefix_and_parents() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let figure_id = inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_KEY);

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        figure_id.as_str(),
        "jwst",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        figure_id.as_str(),
        "jwst:machine_learning",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        figure_id.as_str(),
        "jwst:photometry",
    ]);

    let rename_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "rename",
        "jwst",
        "observatory",
    ]);

    let rename_stdout = String::from_utf8_lossy(&rename_output.stdout);
    assert!(
        rename_stdout.contains("Renamed tag: jwst -> observatory (affected: 3)"),
        "unexpected rename output:\n{rename_stdout}"
    );

    let connection = Connection::open(vault_path.join(".lamian").join("lamian.db"))
        .expect("open sqlite database");
    let renamed_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM tags WHERE tag_name LIKE 'observatory%'",
            [],
            |row| row.get(0),
        )
        .expect("query renamed tags");
    assert_eq!(renamed_count, 3);

    let old_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM tags WHERE tag_name LIKE 'jwst%'",
            [],
            |row| row.get(0),
        )
        .expect("query old tags");
    assert_eq!(old_count, 0);

    let parent_root: Option<String> = connection
        .query_row(
            "SELECT tag_parent FROM tags WHERE tag_name = 'observatory'",
            [],
            |row| row.get(0),
        )
        .expect("query root parent");
    let parent_child: Option<String> = connection
        .query_row(
            "SELECT tag_parent FROM tags WHERE tag_name = 'observatory:machine_learning'",
            [],
            |row| row.get(0),
        )
        .expect("query child parent");
    assert_eq!(parent_root, None);
    assert_eq!(parent_child, Some("observatory".to_string()));
}

#[test]
fn cli_tag_rename_rejects_existing_target_tag() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let figure_id = inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_KEY);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        figure_id.as_str(),
        "jwst",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        figure_id.as_str(),
        "hubble",
    ]);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "tag",
        "rename",
        "jwst",
        "hubble",
    ]);

    assert!(
        !output.status.success(),
        "tag rename unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tag already exists"),
        "unexpected stderr for rename collision:\n{stderr}"
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

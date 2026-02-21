use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rusqlite::Connection;
use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";

#[test]
fn cli_update_persists_name_caption_and_note() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    let note_file_path = temp_dir.path().join("note.md");
    let note_content = "# Observation\nThis panel shows resolved structures.\n";
    std::fs::write(&note_file_path, note_content).expect("write note file");

    let update_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        figure_id.as_str(),
        "--name",
        "JWST Panel 1",
        "--caption",
        "NIRCam composite of target field",
        "--note-file",
        note_file_path.to_string_lossy().as_ref(),
    ]);

    let update_stdout = String::from_utf8_lossy(&update_output.stdout);
    assert!(
        update_stdout.contains(&format!("Updated figure: {figure_id}")),
        "unexpected update stdout:\n{update_stdout}"
    );
    assert!(
        update_stdout.contains("Updated fields: name, caption, note_file"),
        "unexpected updated field list:\n{update_stdout}"
    );

    let connection = Connection::open(vault_path.join(".lamian").join("lamian.db"))
        .expect("open sqlite database");
    let (display_name, caption): (String, Option<String>) = connection
        .query_row(
            "SELECT display_name, caption FROM figures WHERE figure_id = ?1",
            [figure_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query updated figure");
    assert_eq!(display_name, "JWST Panel 1");
    assert_eq!(
        caption,
        Some("NIRCam composite of target field".to_string())
    );

    let persisted_note: String = connection
        .query_row(
            "SELECT note_markdown FROM notes WHERE figure_id = ?1",
            [figure_id.as_str()],
            |row| row.get(0),
        )
        .expect("query persisted note");
    assert_eq!(persisted_note, note_content);
}

#[test]
fn cli_update_rejects_missing_payload() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        figure_id.as_str(),
    ]);

    assert!(
        !output.status.success(),
        "update unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing update payload"),
        "unexpected stderr for missing payload:\n{stderr}"
    );
}

#[test]
fn cli_update_rejects_unknown_figure_id() {
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        "fig_missing",
        "--name",
        "new name",
    ]);

    assert!(
        !output.status.success(),
        "update unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown figure id: fig_missing"),
        "unexpected stderr for unknown figure:\n{stderr}"
    );
}

#[test]
fn cli_update_rejects_missing_note_file_path() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    let missing_note_path = temp_dir.path().join("missing.md");
    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        figure_id.as_str(),
        "--note-file",
        missing_note_path.to_string_lossy().as_ref(),
    ]);

    assert!(
        !output.status.success(),
        "update unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("input file does not exist"),
        "unexpected stderr for missing note file:\n{stderr}"
    );
}

#[test]
fn cli_update_clears_caption_with_flag() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        figure_id.as_str(),
        "--caption",
        "temporary caption",
    ]);

    let clear_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        figure_id.as_str(),
        "--clear-caption",
    ]);

    let clear_stdout = String::from_utf8_lossy(&clear_output.stdout);
    assert!(
        clear_stdout.contains(&format!("Updated figure: {figure_id}")),
        "unexpected clear stdout:\n{clear_stdout}"
    );
    assert!(
        clear_stdout.contains("Updated fields: caption"),
        "unexpected updated field list for clear:\n{clear_stdout}"
    );

    let connection = Connection::open(vault_path.join(".lamian").join("lamian.db"))
        .expect("open sqlite database");
    let caption: Option<String> = connection
        .query_row(
            "SELECT caption FROM figures WHERE figure_id = ?1",
            [figure_id.as_str()],
            |row| row.get(0),
        )
        .expect("query cleared caption");
    assert_eq!(caption, None);
}

#[test]
fn cli_update_rejects_conflicting_caption_set_and_clear() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "update",
        figure_id.as_str(),
        "--caption",
        "value",
        "--clear-caption",
    ]);

    assert!(
        !output.status.success(),
        "update unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot combine --caption with --clear-caption"),
        "unexpected stderr for conflicting caption flags:\n{stderr}"
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

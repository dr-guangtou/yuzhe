use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rusqlite::Connection;
use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";

#[test]
fn cli_source_update_persists_source_metadata_fields() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "source",
        "update",
        figure_id.as_str(),
        "--title",
        "JWST Deep Field",
        "--authors",
        "A. Researcher; B. Scientist",
        "--published-at",
        "2026-02-24",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("Updated source metadata for figure: {figure_id}")),
        "unexpected source update stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("Updated fields: title, authors, published_at"),
        "unexpected updated fields list:\n{stdout}"
    );

    let connection = Connection::open(vault_path.join(".lamian").join("lamian.db"))
        .expect("open sqlite database");
    let (title, authors, published_at): (Option<String>, Option<String>, Option<String>) =
        connection
            .query_row(
                r#"
SELECT source_title, source_authors, source_published_at
FROM sources
WHERE figure_id = ?1
"#,
                [figure_id.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("query source metadata");
    assert_eq!(title, Some("JWST Deep Field".to_string()));
    assert_eq!(authors, Some("A. Researcher; B. Scientist".to_string()));
    assert_eq!(published_at, Some("2026-02-24".to_string()));
}

#[test]
fn cli_source_update_rejects_missing_payload() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "source",
        "update",
        figure_id.as_str(),
    ]);
    assert!(
        !output.status.success(),
        "source update unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing source update payload"),
        "unexpected missing payload error:\n{stderr}"
    );
}

#[test]
fn cli_source_update_rejects_unknown_figure_id() {
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "source",
        "update",
        "fig_missing",
        "--title",
        "new title",
    ]);
    assert!(
        !output.status.success(),
        "source update unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown figure id: fig_missing"),
        "unexpected unknown figure error:\n{stderr}"
    );
}

#[test]
fn cli_source_update_rejects_conflicting_field_flags() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "source",
        "update",
        figure_id.as_str(),
        "--title",
        "value",
        "--clear-title",
    ]);
    assert!(
        !output.status.success(),
        "source update unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot combine set and clear flags for the same field"),
        "unexpected conflicting flag error:\n{stderr}"
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

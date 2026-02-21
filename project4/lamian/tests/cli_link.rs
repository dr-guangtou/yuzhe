use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rusqlite::Connection;
use tempfile::TempDir;

const DOI_KEY: &str = "10.1126/science.ady9404";
const URL_SOURCE_KEY: &str = "https://example.org/secondary-source";

#[test]
fn cli_link_add_persists_relation() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let from_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_KEY);
    let to_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "url", URL_SOURCE_KEY);

    let output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "link",
        "add",
        from_figure_id.as_str(),
        to_figure_id.as_str(),
        "--relation",
        "Supports",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!(
            "Added link: {} -> {} [supports]",
            from_figure_id, to_figure_id
        )),
        "unexpected add output:\n{stdout}"
    );

    let connection = Connection::open(vault_path.join(".lamian").join("lamian.db"))
        .expect("open sqlite database");
    let link_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM links WHERE from_figure_id = ?1 AND to_figure_id = ?2 AND relation_type = ?3",
            [from_figure_id.as_str(), to_figure_id.as_str(), "supports"],
            |row| row.get(0),
        )
        .expect("query link row");
    assert_eq!(link_count, 1);
}

#[test]
fn cli_link_add_is_idempotent_for_duplicate_relation() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let from_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_KEY);
    let to_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "url", URL_SOURCE_KEY);

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "link",
        "add",
        from_figure_id.as_str(),
        to_figure_id.as_str(),
        "--relation",
        "related",
    ]);
    let duplicate_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "link",
        "add",
        from_figure_id.as_str(),
        to_figure_id.as_str(),
        "--relation",
        "RELATED",
    ]);

    let duplicate_stdout = String::from_utf8_lossy(&duplicate_output.stdout);
    assert!(
        duplicate_stdout.contains(&format!(
            "Link already exists: {} -> {} [related]",
            from_figure_id, to_figure_id
        )),
        "unexpected duplicate output:\n{duplicate_stdout}"
    );

    let connection = Connection::open(vault_path.join(".lamian").join("lamian.db"))
        .expect("open sqlite database");
    let link_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM links WHERE from_figure_id = ?1 AND to_figure_id = ?2 AND relation_type = ?3",
            [from_figure_id.as_str(), to_figure_id.as_str(), "related"],
            |row| row.get(0),
        )
        .expect("query duplicate relation");
    assert_eq!(link_count, 1);
}

#[test]
fn cli_link_remove_deletes_all_relations_for_pair() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let from_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_KEY);
    let to_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "url", URL_SOURCE_KEY);

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "link",
        "add",
        from_figure_id.as_str(),
        to_figure_id.as_str(),
        "--relation",
        "related",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "link",
        "add",
        from_figure_id.as_str(),
        to_figure_id.as_str(),
        "--relation",
        "supports",
    ]);

    let remove_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "link",
        "remove",
        from_figure_id.as_str(),
        to_figure_id.as_str(),
    ]);
    let remove_stdout = String::from_utf8_lossy(&remove_output.stdout);
    assert!(
        remove_stdout.contains(&format!(
            "Removed links: {} -> {} (count: 2)",
            from_figure_id, to_figure_id
        )),
        "unexpected remove output:\n{remove_stdout}"
    );

    let connection = Connection::open(vault_path.join(".lamian").join("lamian.db"))
        .expect("open sqlite database");
    let remaining_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM links WHERE from_figure_id = ?1 AND to_figure_id = ?2",
            [from_figure_id.as_str(), to_figure_id.as_str()],
            |row| row.get(0),
        )
        .expect("query remaining links");
    assert_eq!(remaining_count, 0);
}

#[test]
fn cli_link_remove_allows_self_link_cleanup() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id = inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_KEY);

    let connection = Connection::open(vault_path.join(".lamian").join("lamian.db"))
        .expect("open sqlite database");
    connection
        .execute(
            "INSERT INTO links (from_figure_id, to_figure_id, relation_type) VALUES (?1, ?2, ?3)",
            [figure_id.as_str(), figure_id.as_str(), "legacy"],
        )
        .expect("insert legacy self-link");

    let remove_output = run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "link",
        "remove",
        figure_id.as_str(),
        figure_id.as_str(),
    ]);
    let remove_stdout = String::from_utf8_lossy(&remove_output.stdout);
    assert!(
        remove_stdout.contains(&format!(
            "Removed links: {} -> {} (count: 1)",
            figure_id, figure_id
        )),
        "unexpected remove output for self-link cleanup:\n{remove_stdout}"
    );

    let remaining_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM links WHERE from_figure_id = ?1 AND to_figure_id = ?2",
            [figure_id.as_str(), figure_id.as_str()],
            |row| row.get(0),
        )
        .expect("query remaining self-links");
    assert_eq!(remaining_count, 0);
}

#[test]
fn cli_link_add_rejects_unknown_figure_id() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let to_figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "url", URL_SOURCE_KEY);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "link",
        "add",
        "fig_missing_source",
        to_figure_id.as_str(),
        "--relation",
        "related",
    ]);

    assert!(
        !output.status.success(),
        "link add unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown figure id: fig_missing_source"),
        "unexpected stderr for unknown figure:\n{stderr}"
    );
}

#[test]
fn cli_link_add_rejects_self_link() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id = inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_KEY);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "link",
        "add",
        figure_id.as_str(),
        figure_id.as_str(),
        "--relation",
        "related",
    ]);

    assert!(
        !output.status.success(),
        "self link unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("self-link is not allowed"),
        "unexpected stderr for self-link:\n{stderr}"
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

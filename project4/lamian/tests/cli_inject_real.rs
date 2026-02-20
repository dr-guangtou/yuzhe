use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rusqlite::Connection;
use tempfile::TempDir;

const DOI_URL: &str = "https://doi.org/10.1126/science.ady9404";
const DOI_KEY: &str = "10.1126/science.ady9404";
const WIKIPEDIA_ELLIPTICAL_GALAXY_URL: &str = "https://en.wikipedia.org/wiki/Elliptical_galaxy";

#[test]
fn cli_inject_real_repository_fixtures() {
    let first_png = repository_fixture_path("2602.17205_1.png");
    let second_image = repository_fixture_path("500px-Elliptical_galaxy_IC_2006.jpg");

    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "inject",
        first_png.to_string_lossy().as_ref(),
        "--source-type",
        "doi",
        "--source-key",
        DOI_KEY,
        "--copy-mode",
        "reference",
    ]);

    run_lamian_and_assert_success([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "inject",
        second_image.to_string_lossy().as_ref(),
        "--source-type",
        "url",
        "--source-key",
        WIKIPEDIA_ELLIPTICAL_GALAXY_URL,
        "--copy-mode",
        "copy",
    ]);

    let database_path = vault_path.join(".lamian").join("lamian.db");
    let connection = Connection::open(database_path).expect("open sqlite database");

    let figure_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM figures", [], |row| row.get(0))
        .expect("query figure count");
    let source_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM sources", [], |row| row.get(0))
        .expect("query source count");

    assert_eq!(figure_count, 2);
    assert_eq!(source_count, 2);

    let (doi_figure_id, doi_path): (String, String) = connection
        .query_row(
            r#"
SELECT figures.figure_id, figures.file_path
FROM figures
JOIN sources ON sources.figure_id = figures.figure_id
WHERE sources.source_type = 'doi' AND sources.source_key = ?1
"#,
            [DOI_KEY],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query DOI source row");

    let (url_figure_id, url_path): (String, String) = connection
        .query_row(
            r#"
SELECT figures.figure_id, figures.file_path
FROM figures
JOIN sources ON sources.figure_id = figures.figure_id
WHERE sources.source_type = 'url' AND sources.source_key = ?1
"#,
            [WIKIPEDIA_ELLIPTICAL_GALAXY_URL],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query URL source row");

    assert_ne!(doi_figure_id, url_figure_id);

    let first_canonical = first_png.canonicalize().expect("canonical first png");
    assert_eq!(PathBuf::from(doi_path), first_canonical);

    let copied_path = PathBuf::from(url_path);
    assert!(copied_path.starts_with(vault_path.join(".lamian").join("figures")));
    assert!(copied_path.exists());
}

#[test]
fn cli_inject_rejects_wrong_file_format_fixture() {
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let wrong_fixture_path = temp_dir.path().join("wrong_format_fixture.txt");
    std::fs::write(&wrong_fixture_path, b"this is not an image format").expect("write fixture");

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "inject",
        wrong_fixture_path.to_string_lossy().as_ref(),
        "--source-type",
        "manual",
        "--source-key",
        DOI_URL,
        "--copy-mode",
        "reference",
    ]);

    assert!(
        !output.status.success(),
        "inject unexpectedly succeeded for wrong-format fixture.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unsupported media type"),
        "unexpected stderr for wrong-format fixture:\n{stderr}"
    );
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

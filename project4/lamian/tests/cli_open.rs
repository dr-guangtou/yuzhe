use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rusqlite::Connection;
use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";
const OPEN_LAUNCHER_ENVIRONMENT_VARIABLE: &str = "LAMIAN_OPEN_LAUNCHER";

#[test]
fn cli_open_resolves_stored_relative_path_and_reports_success() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    let launcher_path = build_success_launcher(temp_dir.path());

    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);
    let figure_id =
        inject_fixture_and_get_figure_id(&vault_path, &fixture_path, "doi", DOI_SOURCE_KEY, "copy");
    let expected_open_path = resolved_figure_path(&vault_path, figure_id.as_str());

    let output = run_lamian_with_environment_and_assert_success(
        [
            "--vault",
            vault_path.to_string_lossy().as_ref(),
            "open",
            figure_id.as_str(),
        ],
        [(
            OPEN_LAUNCHER_ENVIRONMENT_VARIABLE,
            launcher_path.to_string_lossy().as_ref(),
        )],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains(&format!("Opened figure: {figure_id}")),
        "open output missing figure id:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("Opened path: {}", expected_open_path.display())),
        "open output missing resolved file path:\n{stdout}"
    );
}

#[test]
fn cli_open_fails_for_unknown_figure_id() {
    let temp_dir = TempDir::new().expect("temp directory");
    let vault_path = temp_dir.path().join("vault");
    run_lamian_and_assert_success(["--vault", vault_path.to_string_lossy().as_ref(), "init"]);

    let output = run_lamian([
        "--vault",
        vault_path.to_string_lossy().as_ref(),
        "open",
        "fig_missing",
    ]);

    assert!(
        !output.status.success(),
        "open unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown figure id: fig_missing"),
        "unexpected open stderr:\n{stderr}"
    );
}

fn resolved_figure_path(vault_path: &Path, figure_id: &str) -> PathBuf {
    let connection = open_connection(vault_path);
    let stored_file_path: String = connection
        .query_row(
            "SELECT file_path FROM figures WHERE figure_id = ?1",
            [figure_id],
            |row| row.get(0),
        )
        .expect("query stored figure path");
    let stored_path = PathBuf::from(stored_file_path);
    if stored_path.is_absolute() {
        stored_path
    } else {
        vault_path.join(stored_path)
    }
}

fn open_connection(vault_path: &Path) -> Connection {
    Connection::open(vault_path.join(".lamian").join("lamian.db")).expect("open sqlite database")
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

    assert!(path.exists(), "missing fixture file: {}", path.display());
    path
}

#[cfg(unix)]
fn build_success_launcher(base_directory: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let launcher_path = base_directory.join("open_success.sh");
    std::fs::write(&launcher_path, "#!/bin/sh\nexit 0\n").expect("write success launcher");
    let permissions = std::fs::Permissions::from_mode(0o755);
    std::fs::set_permissions(&launcher_path, permissions).expect("chmod success launcher");
    launcher_path
}

#[cfg(windows)]
fn build_success_launcher(base_directory: &Path) -> PathBuf {
    let launcher_path = base_directory.join("open_success.bat");
    std::fs::write(&launcher_path, "@echo off\r\nexit /b 0\r\n").expect("write success launcher");
    launcher_path
}

fn run_lamian(arguments: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lamian"))
        .args(arguments)
        .output()
        .expect("execute lamian CLI")
}

fn run_lamian_with_environment(
    arguments: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
    environment: impl IntoIterator<Item = (impl AsRef<std::ffi::OsStr>, impl AsRef<std::ffi::OsStr>)>,
) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lamian"));
    command.args(arguments);
    for (key, value) in environment {
        command.env(key, value);
    }
    command
        .output()
        .expect("execute lamian CLI with environment")
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

fn run_lamian_with_environment_and_assert_success(
    arguments: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
    environment: impl IntoIterator<Item = (impl AsRef<std::ffi::OsStr>, impl AsRef<std::ffi::OsStr>)>,
) -> Output {
    let output = run_lamian_with_environment(arguments, environment);
    assert!(
        output.status.success(),
        "lamian command failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

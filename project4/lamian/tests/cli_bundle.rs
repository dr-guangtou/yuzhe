use std::fs::File;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use flate2::read::GzDecoder;
use flate2::{Compression, GzBuilder};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use tar::{Archive, Builder, EntryType, Header};
use tempfile::TempDir;

const DOI_SOURCE_KEY: &str = "10.1126/science.ady9404";

#[derive(Debug, Deserialize)]
struct BundleManifestDocument {
    managed_files: Vec<BundleManifestManagedFile>,
}

#[derive(Debug, Deserialize)]
struct BundleManifestManagedFile {
    bundle_path: String,
}

#[test]
fn cli_bundle_export_and_import_roundtrip_preserves_figure_and_managed_file() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    let figure_id = inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("bundle").join("snapshot.tar.gz");
    let export_output = run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);
    let export_json: JsonValue =
        serde_json::from_slice(&export_output.stdout).expect("parse bundle export json");
    assert_eq!(export_json["command"].as_str(), Some("bundle.export"));
    assert_eq!(export_json["status"].as_str(), Some("ok"));
    assert_eq!(export_json["result"]["figure_count"].as_u64(), Some(1));
    assert_eq!(
        export_json["result"]["managed_file_count"].as_u64(),
        Some(1)
    );

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    let import_output = run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        bundle_path.to_string_lossy().as_ref(),
    ]);
    let import_json: JsonValue =
        serde_json::from_slice(&import_output.stdout).expect("parse bundle import json");
    assert_eq!(import_json["command"].as_str(), Some("bundle.import"));
    assert_eq!(import_json["status"].as_str(), Some("ok"));
    assert_eq!(import_json["result"]["total_figures"].as_u64(), Some(1));
    assert_eq!(import_json["result"]["imported_figures"].as_u64(), Some(1));
    assert_eq!(
        import_json["result"]["skipped_existing_figures"].as_u64(),
        Some(0)
    );
    assert_eq!(
        import_json["result"]["managed_files_written"].as_u64(),
        Some(1)
    );

    let export_after_import = run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "json",
    ]);
    let imported_export_json: JsonValue = serde_json::from_slice(&export_after_import.stdout)
        .expect("parse export json after import");
    assert_eq!(
        imported_export_json["figures"].as_array().map(Vec::len),
        Some(1),
        "expected one figure after bundle import"
    );
    assert_eq!(
        imported_export_json["figures"][0]["figure_id"].as_str(),
        Some(figure_id.as_str())
    );
    let imported_path = imported_export_json["figures"][0]["file_path"]
        .as_str()
        .expect("imported file path");
    assert!(
        imported_path.contains(".lamian"),
        "expected managed path under .lamian, got: {imported_path}"
    );
    assert!(
        imported_path.contains("figures"),
        "expected managed path under .lamian/figures, got: {imported_path}"
    );
    let imported_file_path = PathBuf::from(imported_path);
    assert!(
        imported_file_path.exists(),
        "expected imported managed file to exist at {}",
        imported_file_path.display()
    );
    assert_eq!(
        imported_export_json["figures"][0]["file_hash_sha256"].as_str(),
        Some(sha256_file(&imported_file_path).as_str())
    );
}

#[test]
fn cli_bundle_inspect_reports_validated_summary() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    let inspect_output = run_lamian_and_assert_success([
        "bundle",
        "inspect",
        bundle_path.to_string_lossy().as_ref(),
    ]);
    let inspect_json: JsonValue =
        serde_json::from_slice(&inspect_output.stdout).expect("parse bundle inspect json");
    assert_eq!(inspect_json["command"].as_str(), Some("bundle.inspect"));
    assert_eq!(inspect_json["status"].as_str(), Some("ok"));
    assert_eq!(inspect_json["result"]["figure_count"].as_u64(), Some(1));
    assert_eq!(
        inspect_json["result"]["managed_file_count"].as_u64(),
        Some(1)
    );
    assert_eq!(inspect_json["result"]["bundle_version"].as_u64(), Some(1));
    assert!(inspect_json["result"]["metadata_checksum_sha256"]
        .as_str()
        .is_some());
    assert!(inspect_json["result"]["manifest_checksum_sha256"]
        .as_str()
        .is_some());
}

#[test]
fn cli_bundle_import_dry_run_reports_plan_without_mutation() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    let import_output = run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        bundle_path.to_string_lossy().as_ref(),
        "--dry-run",
    ]);
    let import_json: JsonValue =
        serde_json::from_slice(&import_output.stdout).expect("parse dry-run import json");
    assert_eq!(import_json["command"].as_str(), Some("bundle.import"));
    assert_eq!(import_json["status"].as_str(), Some("ok"));
    assert_eq!(import_json["result"]["dry_run"].as_bool(), Some(true));
    assert_eq!(import_json["result"]["imported_figures"].as_u64(), Some(1));
    assert_eq!(
        import_json["result"]["managed_files_written"].as_u64(),
        Some(1)
    );

    let export_output = run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "json",
    ]);
    let export_json: JsonValue =
        serde_json::from_slice(&export_output.stdout).expect("parse target export json");
    assert_eq!(
        export_json["figures"].as_array().map(Vec::len),
        Some(0),
        "dry-run import must not mutate target vault"
    );
}

#[test]
fn cli_bundle_import_skips_existing_figure_conflict() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &target_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let import_output = run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        bundle_path.to_string_lossy().as_ref(),
    ]);
    let import_json: JsonValue =
        serde_json::from_slice(&import_output.stdout).expect("parse bundle import json");
    assert_eq!(import_json["status"].as_str(), Some("ok"));
    assert_eq!(import_json["result"]["total_figures"].as_u64(), Some(1));
    assert_eq!(import_json["result"]["imported_figures"].as_u64(), Some(0));
    assert_eq!(
        import_json["result"]["skipped_existing_figures"].as_u64(),
        Some(1)
    );
    assert_eq!(
        import_json["result"]["managed_files_written"].as_u64(),
        Some(0)
    );
}

#[test]
fn cli_bundle_import_on_conflict_error_fails_without_mutation() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &target_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let output = run_lamian([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        bundle_path.to_string_lossy().as_ref(),
        "--on-conflict",
        "error",
    ]);
    assert!(
        !output.status.success(),
        "bundle import with on-conflict=error unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("on_conflict=error"),
        "expected on-conflict error message, got:\n{stderr}"
    );

    let export_output = run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "json",
    ]);
    let export_json: JsonValue =
        serde_json::from_slice(&export_output.stdout).expect("parse export json");
    assert_eq!(
        export_json["figures"].as_array().map(Vec::len),
        Some(1),
        "on-conflict=error should preserve prior target state"
    );
}

#[test]
fn cli_bundle_import_on_conflict_replace_overwrites_existing_figure_content() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    let source_figure_id = inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "update",
        source_figure_id.as_str(),
        "--caption",
        "bundle caption",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        source_figure_id.as_str(),
        "science:bundle",
    ]);

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    let target_figure_id = inject_fixture_and_get_figure_id(
        &target_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );
    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "update",
        target_figure_id.as_str(),
        "--caption",
        "target stale caption",
    ]);
    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "tag",
        "add",
        target_figure_id.as_str(),
        "science:target-old",
    ]);

    let import_output = run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        bundle_path.to_string_lossy().as_ref(),
        "--on-conflict",
        "replace",
    ]);
    let import_json: JsonValue =
        serde_json::from_slice(&import_output.stdout).expect("parse replace import json");
    assert_eq!(import_json["status"].as_str(), Some("ok"));
    assert_eq!(
        import_json["result"]["on_conflict"].as_str(),
        Some("replace")
    );
    assert_eq!(import_json["result"]["imported_figures"].as_u64(), Some(1));
    assert_eq!(
        import_json["result"]["skipped_existing_figures"].as_u64(),
        Some(0)
    );

    let export_output = run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "json",
    ]);
    let export_json: JsonValue =
        serde_json::from_slice(&export_output.stdout).expect("parse export json");
    assert_eq!(export_json["figures"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        export_json["figures"][0]["caption"].as_str(),
        Some("bundle caption")
    );
    let tags = export_json["figures"][0]["tags"]
        .as_array()
        .expect("figure tags");
    assert!(tags
        .iter()
        .any(|value| value.as_str() == Some("science:bundle")));
    assert!(!tags
        .iter()
        .any(|value| value.as_str() == Some("science:target-old")));
}

#[test]
fn cli_bundle_import_rejects_corrupted_managed_file_checksum() {
    let fixture_path = repository_fixture_path("500px-Elliptical_galaxy_IC_2006.jpg");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    let corrupted_bundle_path = temp_dir.path().join("snapshot-corrupted.tar.gz");
    write_corrupted_bundle(&bundle_path, &corrupted_bundle_path);

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);

    let output = run_lamian([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        corrupted_bundle_path.to_string_lossy().as_ref(),
    ]);
    assert!(
        !output.status.success(),
        "corrupted bundle import unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("bundle checksum mismatch"),
        "expected checksum mismatch error, got:\n{stderr}"
    );
}

#[test]
fn cli_bundle_import_rejects_duplicate_manifest_entry() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    let invalid_bundle_path = temp_dir.path().join("snapshot-duplicate-manifest.tar.gz");
    write_bundle_with_added_entry(
        &bundle_path,
        &invalid_bundle_path,
        "manifest.json",
        b"{\"bundle_version\":1}\n",
        EntryType::Regular,
    );

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);

    let output = run_lamian([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        invalid_bundle_path.to_string_lossy().as_ref(),
    ]);
    assert!(
        !output.status.success(),
        "duplicate manifest bundle import unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("duplicate manifest entry"),
        "expected duplicate manifest error, got:\n{stderr}"
    );
}

#[test]
fn cli_bundle_import_rejects_duplicate_metadata_entry() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    let invalid_bundle_path = temp_dir.path().join("snapshot-duplicate-metadata.tar.gz");
    write_bundle_with_added_entry(
        &bundle_path,
        &invalid_bundle_path,
        "metadata.json",
        b"{\"schema_version\":5,\"figures\":[]}\n",
        EntryType::Regular,
    );

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);

    let output = run_lamian([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        invalid_bundle_path.to_string_lossy().as_ref(),
    ]);
    assert!(
        !output.status.success(),
        "duplicate metadata bundle import unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("duplicate metadata entry"),
        "expected duplicate metadata error, got:\n{stderr}"
    );
}

#[test]
fn cli_bundle_import_rejects_unexpected_archive_entry() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    let invalid_bundle_path = temp_dir.path().join("snapshot-unexpected-entry.tar.gz");
    write_bundle_with_added_entry(
        &bundle_path,
        &invalid_bundle_path,
        "extra.txt",
        b"unexpected entry",
        EntryType::Regular,
    );

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);

    let output = run_lamian([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        invalid_bundle_path.to_string_lossy().as_ref(),
    ]);
    assert!(
        !output.status.success(),
        "unexpected entry bundle import unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unexpected archive entry outside manifest/metadata/files"),
        "expected unexpected-entry error, got:\n{stderr}"
    );
}

#[test]
fn cli_bundle_import_rejects_unsupported_tar_member_type() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    let invalid_bundle_path = temp_dir.path().join("snapshot-unsupported-member.tar.gz");
    write_bundle_with_added_entry(
        &bundle_path,
        &invalid_bundle_path,
        "files/extra-dir",
        b"",
        EntryType::Directory,
    );

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);

    let output = run_lamian([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        invalid_bundle_path.to_string_lossy().as_ref(),
    ]);
    assert!(
        !output.status.success(),
        "unsupported tar member bundle import unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unsupported tar member type"),
        "expected unsupported member type error, got:\n{stderr}"
    );
}

#[test]
fn cli_bundle_import_rejects_non_portable_reference_path() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "reference",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);

    let output = run_lamian([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        bundle_path.to_string_lossy().as_ref(),
    ]);
    assert!(
        !output.status.success(),
        "non-portable bundle import unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("bundle reference file path is not portable"),
        "expected non-portable path error, got:\n{stderr}"
    );
}

#[test]
fn cli_bundle_import_rejects_invalid_tag_value() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    let invalid_bundle_path = temp_dir.path().join("snapshot-invalid-tags.tar.gz");
    write_bundle_with_modified_metadata(&bundle_path, &invalid_bundle_path, |metadata| {
        let figures = metadata["figures"]
            .as_array_mut()
            .expect("metadata figures array");
        let first_figure = figures.first_mut().expect("first figure");
        first_figure["tags"] = serde_json::json!(["bad tag!"]);
    });

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);

    let output = run_lamian([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        invalid_bundle_path.to_string_lossy().as_ref(),
    ]);
    assert!(
        !output.status.success(),
        "invalid tag bundle import unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tag can only include letters"),
        "expected tag validation error, got:\n{stderr}"
    );
}

#[test]
fn cli_bundle_import_reports_dropped_outbound_links_by_default() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    let bundle_with_missing_link = temp_dir.path().join("snapshot-missing-link.tar.gz");
    write_bundle_with_modified_metadata(&bundle_path, &bundle_with_missing_link, |metadata| {
        let figures = metadata["figures"]
            .as_array_mut()
            .expect("metadata figures array");
        let first_figure = figures.first_mut().expect("first figure");
        first_figure["outbound_links"] = serde_json::json!([
            {
                "to_figure_id": "missing-figure-id",
                "relation_type": "related",
                "created_at": "2026-02-24T00:00:00Z"
            }
        ]);
    });

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);

    let import_output = run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        bundle_with_missing_link.to_string_lossy().as_ref(),
    ]);
    let import_json: JsonValue =
        serde_json::from_slice(&import_output.stdout).expect("parse bundle import json");
    assert_eq!(import_json["status"].as_str(), Some("ok"));
    assert_eq!(import_json["result"]["imported_figures"].as_u64(), Some(1));
    assert_eq!(
        import_json["result"]["outbound_links_seen"].as_u64(),
        Some(1)
    );
    assert_eq!(
        import_json["result"]["outbound_links_written"].as_u64(),
        Some(0)
    );
    assert_eq!(
        import_json["result"]["outbound_links_dropped_missing_target"].as_u64(),
        Some(1)
    );

    let export_output = run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "json",
    ]);
    let export_json: JsonValue =
        serde_json::from_slice(&export_output.stdout).expect("parse export json");
    assert_eq!(export_json["figures"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        export_json["figures"][0]["outbound_links"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );
}

#[test]
fn cli_bundle_import_fail_on_link_loss_rolls_back_import() {
    let fixture_path = repository_fixture_path("2602.17205_1.png");
    let temp_dir = TempDir::new().expect("temp directory");
    let source_vault_path = temp_dir.path().join("source_vault");
    let target_vault_path = temp_dir.path().join("target_vault");

    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);
    inject_fixture_and_get_figure_id(
        &source_vault_path,
        &fixture_path,
        "doi",
        DOI_SOURCE_KEY,
        "copy",
    );

    let bundle_path = temp_dir.path().join("snapshot.tar.gz");
    run_lamian_and_assert_success([
        "--vault",
        source_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "export",
        "--target",
        bundle_path.to_string_lossy().as_ref(),
    ]);

    let bundle_with_missing_link = temp_dir.path().join("snapshot-missing-link.tar.gz");
    write_bundle_with_modified_metadata(&bundle_path, &bundle_with_missing_link, |metadata| {
        let figures = metadata["figures"]
            .as_array_mut()
            .expect("metadata figures array");
        let first_figure = figures.first_mut().expect("first figure");
        first_figure["outbound_links"] = serde_json::json!([
            {
                "to_figure_id": "missing-figure-id",
                "relation_type": "related",
                "created_at": "2026-02-24T00:00:00Z"
            }
        ]);
    });

    run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "init",
    ]);

    let output = run_lamian([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "bundle",
        "import",
        bundle_with_missing_link.to_string_lossy().as_ref(),
        "--fail-on-link-loss",
    ]);
    assert!(
        !output.status.success(),
        "strict link-loss bundle import unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--fail-on-link-loss"),
        "expected strict mode link-loss error, got:\n{stderr}"
    );

    let export_output = run_lamian_and_assert_success([
        "--vault",
        target_vault_path.to_string_lossy().as_ref(),
        "export",
        "--format",
        "json",
    ]);
    let export_json: JsonValue =
        serde_json::from_slice(&export_output.stdout).expect("parse export json");
    assert_eq!(export_json["figures"].as_array().map(Vec::len), Some(0));
}

fn write_corrupted_bundle(source_bundle_path: &Path, destination_bundle_path: &Path) {
    let source_file = File::open(source_bundle_path).expect("open source bundle");
    let mut source_archive = Archive::new(GzDecoder::new(source_file));
    let mut entries = Vec::new();

    for entry_result in source_archive
        .entries()
        .expect("read source archive entries")
    {
        let mut entry = entry_result.expect("read source archive entry");
        let path = entry
            .path()
            .expect("read source archive entry path")
            .to_string_lossy()
            .replace('\\', "/");
        let mut content = Vec::new();
        entry
            .read_to_end(&mut content)
            .expect("read source archive entry content");
        entries.push((path, content));
    }

    let manifest_content = entries
        .iter()
        .find_map(|(path, content)| {
            if path == "manifest.json" {
                Some(content.as_slice())
            } else {
                None
            }
        })
        .expect("manifest content");
    let manifest: BundleManifestDocument =
        serde_json::from_slice(manifest_content).expect("parse bundle manifest");
    let managed_file_bundle_path = manifest
        .managed_files
        .first()
        .expect("first managed file in manifest")
        .bundle_path
        .clone();

    for (path, content) in &mut entries {
        if path == &managed_file_bundle_path {
            if content.is_empty() {
                content.push(1);
            } else {
                content[0] ^= 0x7f;
            }
            break;
        }
    }

    let destination_file = File::create(destination_bundle_path).expect("create corrupted bundle");
    let encoder = GzBuilder::new()
        .mtime(0)
        .write(destination_file, Compression::default());
    let mut destination_archive = Builder::new(encoder);

    for (path, content) in entries {
        let mut header = Header::new_gnu();
        header.set_mode(0o644);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(0);
        header.set_size(content.len() as u64);
        header.set_cksum();
        destination_archive
            .append_data(&mut header, path, Cursor::new(content))
            .expect("append archive entry");
    }

    let encoder = destination_archive
        .into_inner()
        .expect("finalize tar writer");
    encoder.finish().expect("finalize gzip writer");
}

fn write_bundle_with_added_entry(
    source_bundle_path: &Path,
    destination_bundle_path: &Path,
    entry_path: &str,
    entry_content: &[u8],
    entry_type: EntryType,
) {
    let source_file = File::open(source_bundle_path).expect("open source bundle");
    let mut source_archive = Archive::new(GzDecoder::new(source_file));
    let mut entries = Vec::new();

    for entry_result in source_archive
        .entries()
        .expect("read source archive entries")
    {
        let mut entry = entry_result.expect("read source archive entry");
        let path = entry
            .path()
            .expect("read source archive entry path")
            .to_string_lossy()
            .replace('\\', "/");
        let mut content = Vec::new();
        entry
            .read_to_end(&mut content)
            .expect("read source archive entry content");
        entries.push((path, content));
    }

    let destination_file = File::create(destination_bundle_path).expect("create bundle");
    let encoder = GzBuilder::new()
        .mtime(0)
        .write(destination_file, Compression::default());
    let mut destination_archive = Builder::new(encoder);

    for (path, content) in entries {
        let mut header = Header::new_gnu();
        header.set_mode(0o644);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(0);
        header.set_size(content.len() as u64);
        header.set_cksum();
        destination_archive
            .append_data(&mut header, path, Cursor::new(content))
            .expect("append archive entry");
    }

    let mut header = Header::new_gnu();
    header.set_entry_type(entry_type);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    if entry_type.is_file() {
        header.set_size(entry_content.len() as u64);
    } else {
        header.set_size(0);
    }
    header.set_cksum();
    destination_archive
        .append_data(
            &mut header,
            entry_path,
            Cursor::new(if entry_type.is_file() {
                entry_content.to_vec()
            } else {
                Vec::new()
            }),
        )
        .expect("append additional archive entry");

    let encoder = destination_archive
        .into_inner()
        .expect("finalize tar writer");
    encoder.finish().expect("finalize gzip writer");
}

fn write_bundle_with_modified_metadata(
    source_bundle_path: &Path,
    destination_bundle_path: &Path,
    modify: impl FnOnce(&mut JsonValue),
) {
    let source_file = File::open(source_bundle_path).expect("open source bundle");
    let mut source_archive = Archive::new(GzDecoder::new(source_file));
    let mut entries = Vec::new();

    for entry_result in source_archive
        .entries()
        .expect("read source archive entries")
    {
        let mut entry = entry_result.expect("read source archive entry");
        let path = entry
            .path()
            .expect("read source archive entry path")
            .to_string_lossy()
            .replace('\\', "/");
        let mut content = Vec::new();
        entry
            .read_to_end(&mut content)
            .expect("read source archive entry content");
        entries.push((path, content));
    }

    let metadata_index = entries
        .iter()
        .position(|(path, _)| path == "metadata.json")
        .expect("metadata entry");
    let mut metadata_value: JsonValue =
        serde_json::from_slice(&entries[metadata_index].1).expect("parse metadata json");
    modify(&mut metadata_value);
    let metadata_bytes = serialize_json_with_trailing_newline(&metadata_value);
    entries[metadata_index].1 = metadata_bytes.clone();

    let manifest_index = entries
        .iter()
        .position(|(path, _)| path == "manifest.json")
        .expect("manifest entry");
    let mut manifest_value: JsonValue =
        serde_json::from_slice(&entries[manifest_index].1).expect("parse manifest json");
    let metadata_checksum = sha256_bytes(&metadata_bytes);
    manifest_value["metadata_checksum_sha256"] = JsonValue::String(metadata_checksum);
    entries[manifest_index].1 = serialize_json_with_trailing_newline(&manifest_value);

    let destination_file = File::create(destination_bundle_path).expect("create bundle");
    let encoder = GzBuilder::new()
        .mtime(0)
        .write(destination_file, Compression::default());
    let mut destination_archive = Builder::new(encoder);

    for (path, content) in entries {
        let mut header = Header::new_gnu();
        header.set_mode(0o644);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(0);
        header.set_size(content.len() as u64);
        header.set_cksum();
        destination_archive
            .append_data(&mut header, path, Cursor::new(content))
            .expect("append archive entry");
    }

    let encoder = destination_archive
        .into_inner()
        .expect("finalize tar writer");
    encoder.finish().expect("finalize gzip writer");
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

fn sha256_file(path: &Path) -> String {
    let content = std::fs::read(path).expect("read file for sha256");
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn serialize_json_with_trailing_newline(value: &JsonValue) -> Vec<u8> {
    let mut content =
        serde_json::to_string_pretty(value).expect("serialize JSON for bundle rewrite");
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.into_bytes()
}

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::LamianError;

const LAMIAN_DIR_NAME: &str = ".lamian";
const DATABASE_FILE_NAME: &str = "lamian.db";
const CONFIG_FILE_NAME: &str = "vault.toml";

struct Migration {
    version: i64,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    sql: r#"
CREATE TABLE IF NOT EXISTS figures (
    figure_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_hash_sha256 TEXT NOT NULL,
    media_type TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sources (
    source_id INTEGER PRIMARY KEY AUTOINCREMENT,
    figure_id TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK (source_type IN ('doi', 'url', 'local', 'manual')),
    source_key TEXT NOT NULL,
    source_title TEXT,
    source_authors TEXT,
    source_published_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (figure_id) REFERENCES figures(figure_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tags (
    tag_id INTEGER PRIMARY KEY AUTOINCREMENT,
    tag_name TEXT NOT NULL UNIQUE,
    tag_parent TEXT
);

CREATE TABLE IF NOT EXISTS figure_tags (
    figure_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (figure_id, tag_id),
    FOREIGN KEY (figure_id) REFERENCES figures(figure_id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(tag_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS links (
    link_id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_figure_id TEXT NOT NULL,
    to_figure_id TEXT NOT NULL,
    relation_type TEXT NOT NULL DEFAULT 'related',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (from_figure_id) REFERENCES figures(figure_id) ON DELETE CASCADE,
    FOREIGN KEY (to_figure_id) REFERENCES figures(figure_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS notes (
    figure_id TEXT PRIMARY KEY,
    note_markdown TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (figure_id) REFERENCES figures(figure_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sources_figure_id ON sources(figure_id);
CREATE INDEX IF NOT EXISTS idx_figure_tags_tag_id ON figure_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_links_from_figure_id ON links(from_figure_id);
CREATE INDEX IF NOT EXISTS idx_links_to_figure_id ON links(to_figure_id);
"#,
}];

#[derive(Debug, Clone)]
pub struct VaultPaths {
    pub vault_root: PathBuf,
    pub lamian_root: PathBuf,
    pub database_path: PathBuf,
    pub config_path: PathBuf,
}

pub fn initialize_vault(vault_root: &Path) -> Result<VaultPaths, LamianError> {
    if vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: vault_root.to_path_buf(),
        });
    }

    let paths = resolve_vault_paths(vault_root);
    fs::create_dir_all(&paths.vault_root)?;
    fs::create_dir_all(&paths.lamian_root)?;

    let mut connection = Connection::open(&paths.database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    apply_migrations(&mut connection)?;

    ensure_config_file(&paths)?;

    Ok(paths)
}

fn resolve_vault_paths(vault_root: &Path) -> VaultPaths {
    let normalized_root = vault_root.to_path_buf();
    let lamian_root = normalized_root.join(LAMIAN_DIR_NAME);
    let database_path = lamian_root.join(DATABASE_FILE_NAME);
    let config_path = lamian_root.join(CONFIG_FILE_NAME);

    VaultPaths {
        vault_root: normalized_root,
        lamian_root,
        database_path,
        config_path,
    }
}

fn apply_migrations(connection: &mut Connection) -> Result<(), LamianError> {
    let transaction = connection.transaction()?;
    transaction.execute_batch(
        r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#,
    )?;

    let applied_versions = load_applied_versions(&transaction)?;

    for migration in MIGRATIONS {
        if applied_versions.contains(&migration.version) {
            continue;
        }
        transaction.execute_batch(migration.sql)?;
        transaction.execute(
            "INSERT INTO schema_migrations (version) VALUES (?1)",
            [migration.version],
        )?;
    }

    transaction.commit()?;
    Ok(())
}

fn load_applied_versions(connection: &Connection) -> Result<HashSet<i64>, LamianError> {
    let mut statement = connection.prepare("SELECT version FROM schema_migrations")?;
    let mut rows = statement.query([])?;
    let mut versions = HashSet::new();

    while let Some(row) = rows.next()? {
        versions.insert(row.get::<_, i64>(0)?);
    }

    Ok(versions)
}

fn ensure_config_file(paths: &VaultPaths) -> Result<(), LamianError> {
    if paths.config_path.exists() {
        return Ok(());
    }

    let content = format!(
        "schema_version = {}\ncreated_with = \"lamian {}\"\n",
        latest_migration_version(),
        env!("CARGO_PKG_VERSION")
    );
    fs::write(&paths.config_path, content)?;
    Ok(())
}

fn latest_migration_version() -> i64 {
    MIGRATIONS.last().map_or(0, |migration| migration.version)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use tempfile::TempDir;

    use super::initialize_vault;

    #[test]
    fn initialize_vault_creates_database_and_config() {
        let temp_dir = TempDir::new().expect("temp directory");
        let vault_path = temp_dir.path().join("vault");

        let paths = initialize_vault(&vault_path).expect("init vault");

        assert!(paths.vault_root.exists());
        assert!(paths.lamian_root.exists());
        assert!(paths.database_path.exists());
        assert!(paths.config_path.exists());

        let connection = Connection::open(&paths.database_path).expect("open sqlite db");
        assert!(table_exists(&connection, "schema_migrations"));
        assert!(table_exists(&connection, "figures"));
        assert!(table_exists(&connection, "sources"));
        assert!(table_exists(&connection, "tags"));
        assert!(table_exists(&connection, "figure_tags"));
        assert!(table_exists(&connection, "links"));
        assert!(table_exists(&connection, "notes"));
    }

    #[test]
    fn initialize_vault_is_idempotent() {
        let temp_dir = TempDir::new().expect("temp directory");
        let vault_path = temp_dir.path().join("vault");

        let first = initialize_vault(&vault_path).expect("first init");
        let second = initialize_vault(&vault_path).expect("second init");

        assert_eq!(first.database_path, second.database_path);

        let connection = Connection::open(&first.database_path).expect("open sqlite db");
        let migration_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("migration count");

        assert_eq!(migration_count, 1);
    }

    fn table_exists(connection: &Connection, table_name: &str) -> bool {
        let query = "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)";
        let exists: i64 = connection
            .query_row(query, [table_name], |row| row.get(0))
            .expect("table exists query");
        exists == 1
    }
}

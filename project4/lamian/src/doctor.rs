use std::path::PathBuf;

use rusqlite::{Connection, Transaction};
use serde::Serialize;

use crate::db;
use crate::error::LamianError;

#[derive(Debug, Clone)]
pub struct DoctorRequest {
    pub vault_root: PathBuf,
    pub fix: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorResult {
    pub fix_requested: bool,
    pub issue_count_before_fix: usize,
    pub fixed_count: usize,
    pub issue_count_after_fix: usize,
    pub unresolved_count: usize,
    pub issues_before_fix: Vec<DoctorIssue>,
    pub issues_after_fix: Vec<DoctorIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorIssue {
    pub kind: DoctorIssueKind,
    pub subject: String,
    pub detail: String,
    pub fixable: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorIssueKind {
    SelfLink,
    DanglingTagParent,
    OrphanTag,
    FigureWithoutSource,
}

#[derive(Debug, Clone)]
enum IssueRecord {
    SelfLink {
        link_id: i64,
        from_figure_id: String,
    },
    DanglingTagParent {
        tag_id: i64,
        tag_name: String,
        tag_parent: String,
    },
    OrphanTag {
        tag_id: i64,
        tag_name: String,
    },
    FigureWithoutSource {
        figure_id: String,
    },
}

pub fn doctor_vault(request: DoctorRequest) -> Result<DoctorResult, LamianError> {
    if request.vault_root.as_os_str().is_empty() {
        return Err(LamianError::InvalidVaultPath {
            path: request.vault_root,
        });
    }

    let vault_paths = db::resolve_vault_paths(&request.vault_root);
    if !vault_paths.database_path.exists() {
        return Err(LamianError::VaultNotInitialized {
            vault_root: request.vault_root,
        });
    }

    let mut connection = Connection::open(vault_paths.database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    let issues_before_fix_records = collect_issues(&connection)?;
    let mut fixed_count = 0_usize;

    if request.fix {
        let transaction = connection.transaction()?;
        for issue in &issues_before_fix_records {
            fixed_count += apply_fix(&transaction, issue)?;
        }
        transaction.commit()?;
    }

    let issues_after_fix_records = collect_issues(&connection)?;
    let issues_before_fix = issues_before_fix_records
        .iter()
        .map(issue_record_to_output)
        .collect::<Vec<_>>();
    let issues_after_fix = issues_after_fix_records
        .iter()
        .map(issue_record_to_output)
        .collect::<Vec<_>>();

    Ok(DoctorResult {
        fix_requested: request.fix,
        issue_count_before_fix: issues_before_fix.len(),
        fixed_count,
        issue_count_after_fix: issues_after_fix.len(),
        unresolved_count: issues_after_fix.len(),
        issues_before_fix,
        issues_after_fix,
    })
}

fn collect_issues(connection: &Connection) -> Result<Vec<IssueRecord>, LamianError> {
    let mut issues = Vec::new();
    collect_self_link_issues(connection, &mut issues)?;
    collect_dangling_tag_parent_issues(connection, &mut issues)?;
    collect_orphan_tag_issues(connection, &mut issues)?;
    collect_figure_without_source_issues(connection, &mut issues)?;
    Ok(issues)
}

fn collect_self_link_issues(
    connection: &Connection,
    issues: &mut Vec<IssueRecord>,
) -> Result<(), LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT link_id, from_figure_id
FROM links
WHERE from_figure_id = to_figure_id
ORDER BY link_id ASC
"#,
    )?;
    let mut rows = statement.query([])?;

    while let Some(row) = rows.next()? {
        let link_id: i64 = row.get(0)?;
        let from_figure_id: String = row.get(1)?;
        issues.push(IssueRecord::SelfLink {
            link_id,
            from_figure_id,
        });
    }

    Ok(())
}

fn collect_dangling_tag_parent_issues(
    connection: &Connection,
    issues: &mut Vec<IssueRecord>,
) -> Result<(), LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT t.tag_id, t.tag_name, t.tag_parent
FROM tags t
WHERE
    t.tag_parent IS NOT NULL
    AND NOT EXISTS (
        SELECT 1
        FROM tags parent_tag
        WHERE parent_tag.tag_name = t.tag_parent
    )
ORDER BY t.tag_id ASC
"#,
    )?;
    let mut rows = statement.query([])?;

    while let Some(row) = rows.next()? {
        let tag_id: i64 = row.get(0)?;
        let tag_name: String = row.get(1)?;
        let tag_parent: String = row.get(2)?;
        issues.push(IssueRecord::DanglingTagParent {
            tag_id,
            tag_name,
            tag_parent,
        });
    }

    Ok(())
}

fn collect_orphan_tag_issues(
    connection: &Connection,
    issues: &mut Vec<IssueRecord>,
) -> Result<(), LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT t.tag_id, t.tag_name
FROM tags t
WHERE
    NOT EXISTS (
        SELECT 1
        FROM figure_tags ft
        WHERE ft.tag_id = t.tag_id
    )
    AND NOT EXISTS (
        SELECT 1
        FROM tags child
        WHERE child.tag_parent = t.tag_name
    )
ORDER BY t.tag_id ASC
"#,
    )?;
    let mut rows = statement.query([])?;

    while let Some(row) = rows.next()? {
        let tag_id: i64 = row.get(0)?;
        let tag_name: String = row.get(1)?;
        issues.push(IssueRecord::OrphanTag { tag_id, tag_name });
    }

    Ok(())
}

fn collect_figure_without_source_issues(
    connection: &Connection,
    issues: &mut Vec<IssueRecord>,
) -> Result<(), LamianError> {
    let mut statement = connection.prepare(
        r#"
SELECT f.figure_id
FROM figures f
WHERE NOT EXISTS (
    SELECT 1
    FROM sources s
    WHERE s.figure_id = f.figure_id
)
ORDER BY f.figure_id ASC
"#,
    )?;
    let mut rows = statement.query([])?;

    while let Some(row) = rows.next()? {
        let figure_id: String = row.get(0)?;
        issues.push(IssueRecord::FigureWithoutSource { figure_id });
    }

    Ok(())
}

fn apply_fix(transaction: &Transaction, issue: &IssueRecord) -> Result<usize, LamianError> {
    let affected_rows = match issue {
        IssueRecord::SelfLink { link_id, .. } => {
            transaction.execute("DELETE FROM links WHERE link_id = ?1", [link_id])?
        }
        IssueRecord::DanglingTagParent { tag_id, .. } => transaction.execute(
            "UPDATE tags SET tag_parent = NULL WHERE tag_id = ?1",
            [tag_id],
        )?,
        IssueRecord::OrphanTag { tag_id, .. } => {
            transaction.execute("DELETE FROM tags WHERE tag_id = ?1", [tag_id])?
        }
        IssueRecord::FigureWithoutSource { .. } => 0,
    };

    if affected_rows > 0 { Ok(1) } else { Ok(0) }
}

fn issue_record_to_output(issue: &IssueRecord) -> DoctorIssue {
    match issue {
        IssueRecord::SelfLink {
            link_id,
            from_figure_id,
        } => DoctorIssue {
            kind: DoctorIssueKind::SelfLink,
            subject: format!("link:{link_id}"),
            detail: format!("self-link on figure `{from_figure_id}`"),
            fixable: true,
        },
        IssueRecord::DanglingTagParent {
            tag_id,
            tag_name,
            tag_parent,
        } => DoctorIssue {
            kind: DoctorIssueKind::DanglingTagParent,
            subject: format!("tag:{tag_id}:{tag_name}"),
            detail: format!("parent tag `{tag_parent}` does not exist"),
            fixable: true,
        },
        IssueRecord::OrphanTag { tag_id, tag_name } => DoctorIssue {
            kind: DoctorIssueKind::OrphanTag,
            subject: format!("tag:{tag_id}:{tag_name}"),
            detail: "tag has no figure assignments and no child tags".to_string(),
            fixable: true,
        },
        IssueRecord::FigureWithoutSource { figure_id } => DoctorIssue {
            kind: DoctorIssueKind::FigureWithoutSource,
            subject: format!("figure:{figure_id}"),
            detail: "figure has no source records".to_string(),
            fixable: false,
        },
    }
}

# LaMian Implementation Spec (Standalone Snapshot)

## Product Goal

Build a local-only visual knowledge base for research figures with reliable metadata, strong provenance, and an automation-friendly CLI.

## Implemented Scope (Phase 1)

1. Vault initialization (`init`)
2. Figure ingest (`inject`) with strict source provenance
3. Metadata update (`update`)
4. Hierarchical tags
5. Figure links via stable IDs
6. Search and filtering
7. Metadata export (`yaml` or `json`)

## Planned Scope (Phase 1.5, Pre-GUI)

### Wave A

1. Saved queries (`query save|run|list|delete`)
2. Batch import (`import`) with strict provenance templates
3. Vault diagnostics (`doctor`) with DB-only safe `--fix`

### Wave B

1. Collections (`collection`) with static and dynamic modes
2. Portable bundles (`bundle export|import`) via `tar.gz`

## Ingest Architecture Rule

- There is one shared ingest core service.
- CLI `inject`, CLI `import`, and GUI drag-and-drop call the same ingest core service.
- GUI drag-and-drop is an input method, not a separate ingest implementation.
- If source provenance fields are missing at drop time, GUI prompts user metadata before final ingest commit.
- Ingest core accepts one or many file paths.

## Non-Functional Scope

1. Local-first and offline-capable
2. SQLite transactions and migration safety
3. Clear, actionable CLI errors
4. Deterministic output ordering for automation
5. Cross-platform-capable architecture

## Canonical Data Store

- SQLite is the source of truth.
- Sidecar/export/bundle files are portability artifacts, not canonical state.

## CLI Surface (Target After Phase 1.5)

```text
lamian init --vault <path>
lamian inject <file_path> --vault <path> --source-type <type> --source-key <value> [--copy-mode copy|reference]
lamian update <figure_id> [--name ...] [--caption ...] [--note-file ...]
lamian tag add|remove|rename ...
lamian link add|remove ...
lamian search [--tag ...] [--source-key ...] [--text ...]
lamian export [--format yaml|json] [--target <path>]

lamian query save <name> [--tag ...] [--source-key ...] [--text ...] [--sort ...] [--order ...] [--limit ...]
lamian query run <name_or_id> [--detail ids|full]
lamian query list
lamian query delete <name_or_id>

lamian import <input_path> --source-type <type> --source-key-template <template> [--copy-mode copy|reference] [--recursive] [--dry-run]
lamian doctor [--fix]

lamian collection create <name> [--query-id <id>]
lamian collection add <collection> <figure_id>
lamian collection remove <collection> <figure_id>
lamian collection list [--collection <id_or_name>]
lamian collection delete <collection>

lamian bundle export --target <path.tar.gz>
lamian bundle import <path.tar.gz>
```

## Core Tables

- Existing:
  - `figures`
  - `sources`
  - `tags`
  - `figure_tags`
  - `links`
  - `notes`
  - `schema_migrations`
- Planned in Phase 1.5:
  - `saved_queries`
  - `collections`
  - `collection_items`

## Output Contract

- Existing Phase 1 commands keep current human-readable output.
- New Phase 1.5 commands use JSON-only output.
- `query run` supports `--detail ids|full`.

## Acceptance Baseline

1. Existing Phase 1 behavior remains backward compatible.
2. New migrations apply safely from existing vaults.
3. Wave A and Wave B command behavior is covered by tests.
4. Batch and bundle commands provide deterministic summaries and conflict reporting.
5. GUI drag-and-drop path stays equivalent to core ingest behavior.

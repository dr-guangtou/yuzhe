# LaMian Implementation Spec (Standalone Snapshot)

## Product Goal

Build a local-only visual knowledge base for research figures with reliable metadata, strong provenance, and an automation-friendly CLI.

## MVP Functional Scope

1. Vault initialization (`init`)
2. Figure ingest (`inject`) with strict source provenance
3. Metadata update/edit
4. Hierarchical tags
5. Figure links via stable IDs
6. Search and filtering
7. Metadata export (`yaml` or `json`)

## Ingest Architecture Rule

- There is one shared ingest core service.
- CLI `inject` and GUI drag-and-drop both call the same ingest core service.
- GUI drag-and-drop is an input method, not a separate ingest implementation.
- If source provenance fields are missing at drop time, GUI prompts user metadata before final ingest commit.
- Ingest core must accept one or many file paths to support drag-and-drop batches.

## Non-Functional Scope

1. Local-first and offline-capable
2. SQLite transactions and migration safety
3. Clear, actionable CLI errors
4. Cross-platform-capable architecture

## Canonical Data Store

- SQLite is the source of truth.
- Sidecar files are export artifacts, not canonical state.

## CLI Surface (v1 target)

```text
lamian init --vault <path>
lamian inject <file_path> --vault <path> --source-type <type> --source-key <value> [--copy-mode copy|reference]
lamian update <figure_id> [--name ...] [--caption ...] [--note-file ...]
lamian tag add|remove|rename ...
lamian link add|remove ...
lamian search [--tag ...] [--source-key ...] [--text ...]
lamian export [--format yaml|json] [--target <path>]
```

## Core Tables

- `figures` (includes `display_name` and optional `caption`)
- `sources`
- `tags`
- `figure_tags`
- `links`
- `notes`
- `schema_migrations`

## Acceptance Baseline

1. `init` is idempotent.
2. Schema migrations apply safely.
3. CLI and DB behavior is covered by tests.
4. Provenance validation blocks incomplete ingest operations.
5. GUI drag-and-drop path produces equivalent persisted records as CLI ingest.

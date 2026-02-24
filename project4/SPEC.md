# LaMian Technical Specification

## 1. Product Definition

LaMian is a local-only visual knowledge base for research figures and screenshots. It helps users collect, organize, search, audit, and package figures through metadata, tags, notes, links, saved queries, and curated collections.

## 2. Target User and Platform

- Primary user: individual researcher
- Primary platform: macOS
- Architecture direction: cross-platform capable (macOS, Windows, Linux) via Rust core

## 3. Product Principles

- Functionality before appearance
- CLI-first core, GUI as an operational layer over the same core services
- Local-first and offline-friendly
- Strict provenance and metadata quality
- Deterministic outputs for automation and reproducibility
- Minimal and elegant scope for MVP

## 4. Scope

## 4.1 Phase 1 (Completed)

- `init`, `inject`, `update`, `tag`, `link`, `search`, `export`
- migrations v1/v2
- command-specific integration tests

## 4.2 Phase 1.5 (Completed) + Phase 1.x Hardening (Current)

### Wave A

- FR-101 Saved queries (`query save|run|list|delete`)
- FR-102 Batch import (`import`) using strict provenance templates
- FR-103 Vault diagnostics (`doctor`) with optional DB-only safe fixes

### Wave B

- FR-104 Collections (`collection`) with hybrid static/dynamic mode
- FR-105 Portable bundles (`bundle export|import`) using `tar.gz`
- FR-106 Vault integrity verification (`verify`)
- FR-107 Bundle preflight (`bundle inspect`, `bundle import --dry-run`)
- FR-108 Bundle conflict controls (`bundle import --on-conflict skip|error|replace`)

## 4.3 Phase 2 (Next)

- GUI baseline over the same core services.
- Stack for Phase 2.0 baseline: `egui/eframe` (Rust-native, no web runtime dependency).
- Delivery order:
  - Phase 2.0-S1: read-only vault browser + figure detail pane.
  - Later Phase 2.x: mutation flows (`update`/`tag`/`link`/`delete`) and drag-and-drop ingest.

## 5. Functional Requirements

- FR-001 Vault initialization
- FR-002 Figure ingest (`inject`) with strict provenance
- FR-003 Metadata update (`update`)
- FR-004 Hierarchical tags
- FR-005 Directed links
- FR-006 Search/filter
- FR-007 Metadata export (`yaml`/`json`)
- FR-008 Source metadata update (`source update`)
- FR-101 Saved query management
- FR-102 Batch import with per-item reporting
- FR-103 Doctor checks and DB-safe fixes
- FR-104 Hybrid collections
- FR-105 Bundle portability
- FR-106 Vault integrity verification
- FR-107 Bundle preflight
- FR-108 Bundle conflict controls
- FR-201 GUI vault browser (read-only) reusing core list/search services
- FR-202 GUI figure detail panel (read-only) reusing core show service

## 6. Non-Functional Requirements

- NFR-001 Data durability via SQLite transactions
- NFR-002 Schema migration safety and compatibility testing
- NFR-003 Local-only baseline (no required cloud)
- NFR-004 Clear, actionable errors
- NFR-005 Deterministic ordering in search/export/query/bundle outputs
- NFR-006 Performance characteristics validated by measurement, not estimates

## 7. Technical Architecture

## 7.1 Language and Runtime

- Core language: Rust
- Packaging target: local desktop app + CLI binary

## 7.2 Logical Modules

- `domain_core`
  - figure, source, tag, link, note, query, collection models
  - validation and invariants
- `persistence`
  - SQLite schema, migrations, repositories
- `cli_app`
  - command parsing and JSON/human output contracts
- `bundle`
  - tar manifest, checksum verification, import/export orchestration
- `gui_app`
  - desktop state management and rendering (`egui/eframe`)
  - vault connection UX and error presentation
- `shared_core`
  - library-exposed service boundary reused by CLI and GUI binaries

## 7.3 Data Store Strategy

- Canonical store: SQLite
- Portability: export and bundle artifacts
- Source of truth: database wins in conflicts

## 8. Data Model

## 8.1 Existing Tables

- `figures` (`caption` included since migration v2)
- `sources`
- `tags`
- `figure_tags`
- `links`
- `notes`
- `schema_migrations`

## 8.2 Additional Tables Added in Phase 1.5

- `saved_queries`
  - `query_id`, `query_name`, `filters_json`, `sort_field`, `sort_order`, `limit_count`, timestamps
- `collections`
  - `collection_id`, `collection_name`, `collection_mode` (`static|dynamic`), optional `query_id`, timestamps
- `collection_items`
  - `collection_id`, `figure_id`, `added_at`

## 8.3 Schema Hardening Updates

- Migration v5 enforces link business-key uniqueness (`from_figure_id`, `to_figure_id`, `relation_type`) after deduplicating legacy duplicates.

## 9. CLI Interface (Target After Phase 1.5)

```text
lamian init --vault <path>
lamian inject <file_path> --vault <path> --source-type <type> --source-key <value> [--copy-mode copy|reference]
lamian update <figure_id> [--name ...] [--caption ...] [--note-file ...]
lamian source update <figure_id> [--title ...] [--authors ...] [--published-at ...] [--clear-title] [--clear-authors] [--clear-published-at]
lamian tag add|remove|rename ...
lamian link add|remove ...
lamian search [--tag ...] [--tag-prefix ...] [--source-key ...] [--text ...]
lamian list|ls [--sort figure-id|display-name|created-at|updated-at] [--order asc|desc] [--limit <n>]
lamian show|info <figure_id>
lamian delete <figure_id>
lamian export [--format yaml|json] [--target <path>]

lamian query save <name> [--tag ...] [--source-key ...] [--text ...] [--sort ...] [--order ...] [--limit ...]
lamian query run <name_or_id> [--detail ids|full] [--reference-mode auto|id|name]
lamian query list
lamian query delete <name_or_id> [--reference-mode auto|id|name]

lamian import <input_path> --source-type <type> --source-key-template <template> [--copy-mode copy|reference] [--recursive] [--dry-run]
lamian doctor [--fix]
lamian verify

lamian collection create <name> [--query-id <id>]
lamian collection add <collection> <figure_id> [--reference-mode auto|id|name]
lamian collection remove <collection> <figure_id> [--reference-mode auto|id|name]
lamian collection list [--collection <id_or_name>] [--reference-mode auto|id|name]
lamian collection delete <collection> [--reference-mode auto|id|name]

lamian bundle export --target <path.tar.gz>
lamian bundle inspect <path.tar.gz>
lamian bundle import <path.tar.gz> [--fail-on-link-loss] [--dry-run] [--on-conflict skip|error|replace]
```

## 10. Output Contracts

- Existing commands retain current human-readable output.
- New Phase 1.5 commands output JSON only.
- Batch operations include:
  - summary counts (`processed`, `succeeded`, `failed`, `skipped`)
  - per-item result/error records
- `query run` supports `--detail ids|full`.
- `verify` reports integrity issues and exits non-zero when unresolved issues exist.
- `bundle import --dry-run` reports deterministic projections and does not mutate DB/files.

## 11. Error Handling Contract

- Validation errors:
  - missing provenance fields
  - malformed tags
  - invalid query/filter payload
  - invalid bundle target or manifest
- IO errors:
  - unreadable files
  - missing source files
  - inaccessible vault path
- Data integrity errors:
  - rollback on failed writes
  - conflict policy for bundle import is explicit (`skip|error|replace`) with default `skip`

## 12. Security and Compliance Baseline

- Local-only processing by default
- User is responsible for source rights/license compliance
- Bundle import validates checksums before persistence

## 13. Testing and Acceptance Criteria

## 13.1 Unit Tests

- query filter serialization/validation
- import template rendering
- doctor issue detection and DB-safe fix routines
- collection mode invariants
- bundle manifest/checksum validation

## 13.2 Integration Tests

- existing Phase 1 command suite
- full sequential workflow
- query/import/doctor workflows
- collection static and dynamic workflows
- bundle export/import roundtrip and conflict handling
- migration compatibility from v2 to v3/v4

## 13.3 Acceptance Gates

- Phase 1.5 Wave A:
  - `query`, `import`, `doctor` implemented with JSON contracts and tests
- Phase 1.5 Wave B:
  - `collection`, `bundle` implemented with tests
- GUI gate:
  - all Phase 1.5 tests green
  - docs synchronized across `project4/` and `project4/lamian/docs/`
  - Phase 2.0-S1 GUI launch succeeds on macOS and preserves deterministic row ordering from core services

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
  - Phase 2.0-S2: metadata mutation flow (`update` + `source update`) with explicit edit/save/cancel state handling.
  - Later Phase 2.x: tag/link/delete mutation flows and drag-and-drop ingest.

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
- FR-203 GUI metadata mutation flow with separate figure/source edit sessions and shared-core validation semantics
- FR-204 GUI tag/link/delete mutation flow with deterministic post-mutation list/detail behavior and shared-core validation semantics
- FR-205 GUI drag-and-drop ingest flow reusing shared ingest core with explicit provenance prompt states and deterministic batch application semantics
- FR-206 GUI workflow parity polish for open-file, navigation, and search/filter ergonomics with deterministic interaction outcomes

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

## 7.3 Phase 2.0-S2 GUI Mutation State Model (Implemented)

- Scope is limited to metadata mutation through existing core services:
  - figure fields via `update`
  - source fields via `source update`
- UI splits mutation into two independent edit sessions to avoid cross-service partial transaction ambiguity:
  - Figure metadata editor (`name`, `caption`, `clear_caption`)
  - Source metadata editor (`title`, `authors`, `published_at`, corresponding clear flags)
- Each editor uses the same state lifecycle:
  - `viewing`: read-only panel
  - `editing_clean`: draft opened but unchanged
  - `editing_dirty`: draft changed
  - `saving`: submit in flight, save/cancel disabled
  - `save_failed`: backend returned error; draft preserved for retry/cancel
- Save/cancel behavior:
  - `Edit` copies current detail into draft snapshot.
  - `Save` submits only changed fields to the matching shared service.
  - `Cancel` discards draft and restores `viewing` state without backend calls.
- Validation mapping rule:
  - GUI only enforces interaction-level guards (for example, disable save when no changes).
  - Domain validation remains in shared services; backend error messages are surfaced verbatim in GUI error panel.
  - Clear/set conflicts are prevented in UI controls and still tolerated as backend-protected invariants.
- Post-save refresh:
  - On success, reload selected figure detail via `show`.
  - If figure display name changed, refresh list rows via existing list/search path to keep deterministic ordering behavior.
- Regression and determinism checks:
  - GUI unit tests cover draft lifecycle transitions (`editing_clean`, `editing_dirty`, `saving`, `save_failed`).
  - Save-failure paths preserve drafts and allow retry/cancel recovery for both figure and source editors.
  - Post-save behavior is validated for deterministic list ordering and selected-detail continuity.

## 7.4 Phase 2.1 GUI Mutation Expansion State Model (Implemented Closure)

- Scope adds three GUI mutation flows on top of existing shared services:
  - tag mutation on selected figure via `tag add/remove`
  - link mutation on selected figure via `link add/remove`
  - figure deletion via `delete`
- State and interaction model:
  - tag/link flows: `viewing` -> `editing_clean` -> `editing_dirty` -> `saving` -> (`viewing` on success or `save_failed` on backend error)
  - delete flow: `viewing` -> `confirming_delete` -> `deleting` -> (`viewing` with next selection or `delete_failed`)
- Confirmation and safety rules:
  - delete requires explicit confirmation interaction before calling backend service.
  - while `saving`/`deleting`, mutation controls are disabled to prevent duplicate submits.
- Validation mapping rule:
  - GUI enforces interaction guards only (for example, disable no-op actions or invalid local transitions).
  - Domain validation and invariants remain in shared services; backend error text is surfaced directly in GUI.
- Deterministic post-mutation refresh policy:
  - Tag/link success reloads selected figure detail via `show`; list row ordering remains service-driven.
  - Delete success reloads list/search rows and applies deterministic next selection policy:
    - select the next row at the deleted row index when available;
    - otherwise select the previous row;
    - if no rows remain, clear selection/detail state.
- Regression coverage implemented for Phase 2.1:
  - state transitions for tag/link/delete flows
  - save/delete failure recovery with retry/cancel behavior
  - deterministic list/detail behavior after tag/link/delete success paths

## 7.5 Phase 2.2 GUI Drag-and-Drop Ingest State Model (Design Locked)

- Scope adds drag-and-drop as a GUI input method for existing shared ingest core service:
  - single-file drop and multi-file drop are both accepted.
  - GUI does not implement a separate ingest path; it calls the same ingest service family used by CLI `inject`/`import`.
- Drop session state model:
  - `idle` -> `drop_received` -> `metadata_required` -> `ready_to_commit` -> `committing` -> (`committed` or `commit_failed`)
  - `metadata_required` is entered when required provenance fields are missing for at least one dropped item.
  - `ready_to_commit` requires provenance completeness validation for all items in the pending drop batch.
- Provenance prompt contract:
  - required fields remain `source_type` and `source_key` (matching shared ingest validation semantics).
  - GUI provides per-batch metadata defaults with optional per-item override before commit.
  - commit action is blocked until all items satisfy shared ingest provenance requirements.
- Deterministic behavior contract:
  - multi-file drops are committed in deterministic lexicographic path order after path normalization.
  - post-commit row ordering remains fully service-driven; GUI renders rows exactly as returned by shared list/search services.
  - partial-success outcomes are reported deterministically with stable per-item result ordering.
- Safety and compatibility contract:
  - failed items do not mutate successful-item result ordering or payload semantics.
  - Rust crate edition remains 2021 and drag-and-drop flow preserves existing deterministic CLI/domain invariants.
- Acceptance target before implementation:
  - UX/state flow is design-locked in both incubator and standalone SPEC mirrors before coding starts.

## 7.6 Phase 2.3 GUI Workflow Parity Polish State Model (P4-524/L-224, Design Locked)

- Scope closes GUI workflow parity gaps on top of existing shared services:
  - open selected figure file from GUI using the same resolved-path semantics as CLI `open`.
  - list/detail navigation polish for keyboard and pointer interaction without introducing a new domain path.
  - search/filter ergonomics polish (apply/clear/refine) while preserving service-driven result ordering.
- State and interaction contract:
  - open-file action: `ready` -> `opening_file` -> (`open_succeeded` or `open_failed`) -> `ready`.
  - navigation/search interaction: `browsing` -> (`filtering` or `navigating`) -> `browsing`.
  - transient states never reorder rows locally; rendered ordering always comes from shared list/search responses.
- Validation and ownership boundary:
  - GUI enforces only interaction-level guards (for example, disable open when no selection exists).
  - path resolution, open execution semantics, and domain validation remain in shared services.
  - backend error text is surfaced directly to avoid semantic drift between CLI and GUI behavior.
- Deterministic behavior contract:
  - selection transitions are deterministic for repeated key/pointer sequences against the same row set.
  - filter apply/clear cycles produce deterministic list/detail refresh order based on shared service output.
  - open success/failure does not mutate list ordering or selected detail payload unexpectedly.
- Rust/toolchain compatibility contract:
  - implementation and tests remain Rust 2021-compatible (no Rust 2024-only syntax/features).
  - parity polish does not relax deterministic ordering guarantees already locked in earlier Phase 2 slices.
- Acceptance target before implementation:
  - this Phase 2.3 design lock is documented in both `project4/SPEC.md` and `project4/lamian/docs/SPEC.md` before `P4-525` coding begins.

## 7.7 Data Store Strategy

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
- GUI mutation regression unit tests for S2 editor state transitions and save recovery/determinism behavior
- GUI mutation regression unit tests for Phase 2.1 tag/link/delete state transitions and deterministic post-mutation behavior

## 13.3 Acceptance Gates

- Phase 1.5 Wave A:
  - `query`, `import`, `doctor` implemented with JSON contracts and tests
- Phase 1.5 Wave B:
  - `collection`, `bundle` implemented with tests
- GUI gate:
  - all Phase 1.5 tests green
  - docs synchronized across `project4/` and `project4/lamian/docs/`
  - Phase 2.0-S1 GUI launch succeeds on macOS and preserves deterministic row ordering from core services
  - Phase 2.0-S2 figure/source metadata mutation flows are implemented via shared services and covered by regression tests for save success/failure recovery and deterministic post-save list/detail behavior
  - Phase 2.1 tag/link/delete mutation flows are implemented through shared `tag`/`link`/`delete` services and covered by regression tests for lifecycle failure recovery and deterministic post-mutation selection/refresh behavior
  - Phase 2.2 drag-and-drop ingest UX/state flow is design-locked to shared ingest-core reuse with explicit provenance prompt states and deterministic multi-file commit semantics before implementation
  - Phase 2.3 workflow parity-polish UX/state flow is design-locked for open-file/navigation/search-filter ergonomics with explicit Rust 2021 and deterministic ordering constraints before implementation

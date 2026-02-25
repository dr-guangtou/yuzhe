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

## Phase 2.0 GUI Baseline (Locked)

1. GUI stack: `egui/eframe`.
2. Architecture boundary: extract shared library exports first, then wire both CLI and GUI binaries to the same service layer.
3. First slice (Phase 2.0-S1): read-only vault browser and figure detail panel using existing `list`, `search`, and `show` service semantics.
4. Second slice (Phase 2.0-S2): metadata mutation flow for figure/source fields using `update` and `source update`.
5. Later slices: additional mutation flows and drag-and-drop ingest after S2 stabilization.

## Phase 2.0-S2 Mutation UX and State Flow (P4-505/P4-508, Implemented)

1. Mutation scope is split into two editors to keep service boundaries explicit:
   - figure metadata editor (`name`, `caption`, `clear_caption`)
   - source metadata editor (`title`, `authors`, `published_at`, per-field clear flags)
2. Each editor follows the same state sequence:
   - `viewing` -> `editing_clean` -> `editing_dirty` -> `saving` -> (`viewing` on success or `save_failed` on error)
3. Save/cancel behavior:
   - `Edit` snapshots currently loaded values into a local draft.
   - `Save` submits only changed fields to the corresponding shared service.
   - `Cancel` discards draft and returns to `viewing` without backend mutation.
4. Validation mapping:
   - GUI guards interaction invariants only (for example, no-op save disabled).
   - Domain validation remains centralized in shared services.
   - Backend error text is surfaced directly in GUI feedback to avoid semantic drift.
5. Post-save refresh:
   - always reload selected figure via `show`.
   - if display name changed, reload list/search rows so ordering remains service-driven and deterministic.
6. Regression and determinism coverage:
   - GUI unit tests verify lifecycle transitions for both editors (`editing_clean`, `editing_dirty`, `saving`, `save_failed`).
   - Save-failure paths preserve drafts and expose backend errors while allowing retry/cancel recovery.
   - Post-save behavior is asserted to keep list ordering deterministic and selected detail synchronized.

## Phase 2.1 Mutation Expansion UX and State Flow (P4-511..P4-516, Implemented Closure)

1. Scope extends GUI mutation flows to existing shared services:
   - tag add/remove on selected figure
   - link add/remove on selected figure
   - figure delete
2. State sequence by flow:
   - tag/link: `viewing` -> `editing_clean` -> `editing_dirty` -> `saving` -> (`viewing` on success or `save_failed` on error)
   - delete: `viewing` -> `confirming_delete` -> `deleting` -> (`viewing` on success or `delete_failed` on error)
3. Delete safety contract:
   - delete requires explicit confirmation before backend mutation call.
   - controls are disabled during `deleting` to prevent duplicate submissions.
4. Validation mapping:
   - GUI enforces interaction-level guards only.
   - domain validation remains centralized in shared services.
   - backend error messages are surfaced directly in GUI feedback.
5. Deterministic refresh contract:
   - tag/link success reloads selected detail via `show`; list ordering remains service-driven.
   - delete success reloads list/search rows and applies deterministic selection policy:
     - select next row at deleted index when available;
     - else select previous row;
     - clear selection/detail when list is empty.
6. Regression coverage implemented for Phase 2.1:
   - state transitions for tag/link/delete mutation flows
   - save/delete failure recovery and retry/cancel behavior
   - deterministic list/detail behavior after mutation success paths

## Phase 2.2 Drag-and-Drop Ingest UX and State Flow (P4-517/L-218, Design Locked)

1. Scope adds drag-and-drop as a GUI input method over existing shared ingest core services:
   - single-file and multi-file drops are accepted.
   - GUI does not introduce a separate ingest implementation path.
2. Drop session state sequence:
   - `idle` -> `drop_received` -> `metadata_required` -> `ready_to_commit` -> `committing` -> (`committed` or `commit_failed`)
   - `metadata_required` is entered when any required provenance field is missing in the pending batch.
   - `ready_to_commit` requires provenance completeness for all pending items.
3. Provenance prompt contract:
   - required fields remain `source_type` and `source_key` under shared ingest validation semantics.
   - GUI supports batch defaults with optional per-item override before commit.
   - commit remains blocked until all items satisfy provenance requirements.
4. Deterministic behavior contract:
   - dropped items commit in deterministic lexicographic normalized-path order.
   - post-commit list/search ordering stays service-driven; GUI renders rows in returned order only.
   - per-item success/failure reporting remains stable and deterministic for multi-file batches.
5. Safety and compatibility contract:
   - failed items do not alter successful-item result ordering or payload semantics.
   - Rust crate edition remains 2021 and this flow preserves existing deterministic CLI/domain guarantees.
6. Acceptance target before implementation:
   - design lock is documented in both incubator and standalone SPEC mirrors before coding starts.

## Phase 2.3 Workflow Parity Polish UX and State Flow (P4-524/L-224, Design Locked)

1. Scope closes GUI workflow parity gaps on top of existing shared services:
   - open selected figure file from GUI using the same resolved-path semantics as CLI `open`.
   - list/detail navigation polish for keyboard and pointer interaction without introducing a new domain path.
   - search/filter ergonomics polish (apply/clear/refine) while preserving service-driven result ordering.
2. State and interaction contract:
   - open-file action: `ready` -> `opening_file` -> (`open_succeeded` or `open_failed`) -> `ready`.
   - navigation/search interaction: `browsing` -> (`filtering` or `navigating`) -> `browsing`.
   - transient states never reorder rows locally; rendered ordering always comes from shared list/search responses.
3. Validation and ownership boundary:
   - GUI enforces only interaction-level guards (for example, disable open when no selection exists).
   - path resolution, open execution semantics, and domain validation remain in shared services.
   - backend error text is surfaced directly to avoid semantic drift between CLI and GUI behavior.
4. Deterministic behavior contract:
   - selection transitions are deterministic for repeated key/pointer sequences against the same row set.
   - filter apply/clear cycles produce deterministic list/detail refresh order based on shared service output.
   - open success/failure does not mutate list ordering or selected detail payload unexpectedly.
5. Rust/toolchain compatibility contract:
   - implementation and tests remain Rust 2021-compatible (no Rust 2024-only syntax/features).
   - parity polish does not relax deterministic ordering guarantees already locked in earlier Phase 2 slices.
6. Acceptance target before implementation:
   - this Phase 2.3 design lock is documented in both `project4/SPEC.md` and `project4/lamian/docs/SPEC.md` before `L-225` coding begins.

## Phase 1.8 Decisions (Finalized)

### Wave 1-3 (Implemented)

1. Tag rename (`tag rename`) computes a full rename plan before any mutation, then updates by `tag_id` to avoid descendant corruption when prefixes overlap.
2. `link remove` allows self-link cleanup (`from_figure_id == to_figure_id`) while `link add` continues to reject self-links.
3. Crate edition policy is pinned to Rust 2021 for broader toolchain compatibility.
4. Tag normalization and validation are centralized in a shared helper reused by `tag`, `search`, and `query`, preserving existing error semantics while removing duplication.
5. Bundle import stages managed files under `.lamian/bundle_import_staging`, records promotion metadata in `.lamian/bundle_import_journal.json`, and promotes files after DB commit with startup recovery.
6. Bundle import rejects non-portable reference file paths (absolute, UNC/drive, or parent traversal) to keep bundles portable and fails fast on the first violation.
7. Bundle import reuses CLI domain validation for sources, tags, and link relations, normalizing values and rejecting invalid payloads.
8. Bundle import reports outbound link-loss counters (`outbound_links_seen`, `outbound_links_written`, `outbound_links_dropped_missing_target`); default mode skips missing targets with reporting, and `bundle import --fail-on-link-loss` converts any missing target into a hard failure.
9. Bundle export/import managed-file IO is streaming: export hashes/writes files from disk without loading all managed bytes into memory, and import verifies/stages managed entries by scanning archive streams.
10. Bundle archive structure is strict: exactly one `manifest.json`, exactly one `metadata.json`, managed payloads must be regular files under `files/`, and unexpected/non-regular tar members are rejected during import preflight.
11. Query and collection reference resolution supports explicit disambiguation with `--reference-mode auto|id|name`; default `auto` keeps legacy behavior (try numeric id first, then name).
12. Vault integrity verification is a read-only command: `verify` checks figure file existence plus filesystem-vs-DB hash and size drift without mutating DB or files.
13. Bundle preflight is explicit: `bundle inspect` validates archive structure/checksums and reports summary metadata, and `bundle import --dry-run` produces deterministic import projections without DB or filesystem mutation.
14. Bundle import conflict handling is explicit via `--on-conflict skip|error|replace` (default `skip`): `skip` preserves legacy behavior, `error` fails fast on first conflict, and `replace` rewrites existing same-`figure_id` records in place.

### Later Waves (Implemented)

1. Saved queries support filterless definitions (sort/order/limit-only templates).
2. Phase 1 command families support global `--json` output parity for `inject`, `update`, `tag`, `link`, `search`, and `export`.

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
lamian [--json] inject <file_path> --vault <path> --source-type <type> --source-key <value> [--copy-mode copy|reference]
lamian [--json] update <figure_id> [--name ...] [--caption ...] [--clear-caption] [--note-file ...]
lamian source update <figure_id> [--title ...] [--authors ...] [--published-at ...] [--clear-title] [--clear-authors] [--clear-published-at]
lamian [--json] tag add|remove|rename|list ...
lamian [--json] link add|remove ...
lamian [--json] search [--tag ...] [--tag-prefix ...] [--source-key ...] [--text ...]
lamian list|ls [--sort figure-id|display-name|created-at|updated-at] [--order asc|desc] [--limit <n>]
lamian show|info <figure_id>
lamian delete <figure_id>
lamian [--json] export [--format yaml|json] [--target <path>]

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
lamian collection update <collection> [--reference-mode auto|id|name] [--name <new_name>] [--query-id <id>] [--clear-query-id]

lamian bundle export --target <path.tar.gz>
lamian bundle inspect <path.tar.gz>
lamian bundle import <path.tar.gz> [--fail-on-link-loss] [--dry-run] [--on-conflict skip|error|replace]
```

## Core Tables

- `figures`
- `sources`
- `tags`
- `figure_tags`
- `links`
- `notes`
- `schema_migrations`
- `saved_queries`
- `collections`
- `collection_items`

## Output Contract

- Existing Phase 1 commands keep current human-readable output.
- New Phase 1.5 commands use JSON-only output.
- `query run` supports `--detail ids|full`.
- `verify` is JSON-only with `{ "command": "verify", "status": "ok"|"issues_found", "result": { ... } }`; any non-zero issue count returns `issues_found` and exits non-zero.
- `bundle inspect` is JSON-only and runs the same archive preflight validations as `bundle import` before reporting summary fields.
- `bundle import --dry-run` is JSON-only and returns projected import counters with `result.dry_run = true`; no DB rows or managed files are written.
- `bundle import` reports the active conflict policy as `result.on_conflict` and applies deterministic conflict semantics based on `--on-conflict`.
- Phase 1 commands keep human-readable defaults and also accept global `--json` for machine-friendly envelopes on `inject`, `update`, `tag`, `link`, `search`, and `export`.

## Acceptance Baseline

1. Existing Phase 1 behavior remains backward compatible.
2. New migrations apply safely from existing vaults.
3. Wave A and Wave B command behavior is covered by tests.
4. Batch and bundle commands provide deterministic summaries and conflict reporting.
5. GUI drag-and-drop path stays equivalent to core ingest behavior.
6. Phase 2.0-S1 GUI preserves service ordering guarantees by rendering rows in exactly the order returned from shared core list/search services.
7. Phase 2.0-S2 mutation flows are implemented through shared `update`/`source update` services and validated by GUI regression tests for state transitions, failure recovery, and deterministic post-save list/detail behavior.
8. Phase 2.1 tag/link/delete mutation flows are implemented via shared `tag`/`link`/`delete` services and validated by regression tests for failure recovery and deterministic post-mutation list/detail behavior.
9. Phase 2.2 drag-and-drop ingest UX/state flow is design-locked to shared ingest-core reuse with explicit provenance prompts and deterministic multi-file commit ordering before implementation.
10. Phase 2.3 workflow parity-polish UX/state flow is design-locked for open-file/navigation/search-filter ergonomics with explicit Rust 2021 and deterministic ordering constraints before implementation.

# LaMian TODO (Standalone)

## Phase 1: CLI Core

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-101 | Initialize Rust CLI crate | Done | `init` command available |
| L-102 | Add SQLite migration framework | Done | migrations v1/v2 implemented |
| L-103 | Implement core commands (`inject`, `update`, `tag`, `link`, `search`, `export`) | Done | command cores implemented with typed validation |
| L-104 | Add command-focused integration tests | Done | tests cover inject/update/tag/link/search/export |
| L-105 | Add one full sequential workflow integration test | Done | `cli_workflow` covers `init -> inject -> update -> tag -> link -> search -> export` |

## Phase 1.5: Automation and Curation (Pre-GUI)

### Wave A

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-151 | Add migration for `saved_queries` table | Done | migration v3 added with `saved_queries` schema and index |
| L-152 | Implement `query save|run|list|delete` | Done | JSON-only output implemented with `run --detail ids|full` |
| L-153 | Implement `import` batch ingest | Done | strict provenance template + continue-on-error summary + duplicate skip/report implemented |
| L-154 | Implement `doctor` checks and DB-only `--fix` | Done | deterministic checks implemented; `--fix` mutates DB only |
| L-155 | Add Wave A integration tests | Done | query/import/doctor integration coverage added |

### Wave B

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-161 | Add migration for collections tables | Done | migration v4 added with constraints and indexes for `collections` + `collection_items` |
| L-162 | Implement hybrid `collection` command family | Done | `collection create/add/remove/list/delete` implemented with static/dynamic behavior |
| L-163 | Implement `bundle export|import` with `tar.gz` | Done | deterministic `manifest.json` + checksum verification, metadata + managed files, skip-existing conflict policy |
| L-164 | Add Wave B integration tests | Done | bundle roundtrip/conflict/corruption coverage added (with existing collection tests) |

## Phase 2: GUI Baseline

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-201 | Select Rust GUI approach | Done | selected `egui/eframe` for Rust-native Phase 2.0 baseline |
| L-202 | Build vault browser and figure detail editor | Done | delivered read-only browse/detail plus S2 figure/source metadata editors with deterministic refresh behavior |
| L-203 | Extract shared library boundary for CLI and GUI | Done | added `src/lib.rs` exports and migrated CLI main to library imports |
| L-204 | Add `lamian_gui` desktop binary | Done | added `src/bin/lamian_gui.rs` and `src/gui.rs` with read-only vault browse/detail flow |
| L-205 | Run full verification gate for Phase 2.0-S1 | Done | passed `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test` |
| L-206 | Deliver read-only Phase 2.0-S1 GUI slice | Done | supports vault open, deterministic list/search rows, and `show`-equivalent figure detail rendering |
| L-207 | Design Phase 2.0-S2 GUI mutation UX/state flow | Done | locked editor state lifecycle, shared-service validation mapping, and save/cancel semantics in spec docs |
| L-208 | Wire GUI figure metadata editing to shared `update` service | Done | implemented editor draft state, save/cancel actions, and backend error surfacing via `update_figure` |
| L-209 | Wire GUI source metadata editing to shared `source update` service | Done | implemented source editor draft state, clear-flag controls, and backend error surfacing via `update_source_metadata` |
| L-210 | Add GUI mutation regression and deterministic behavior checks | Done | added `src/gui.rs` regression tests for editor lifecycle transitions, save failure recovery, and deterministic list/detail behavior after figure/source saves |
| L-211 | Run full verification gate for Phase 2.0-S2 | Done | passed `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test` after L-210 |

## Phase 2.1 GUI Mutation Expansion (Implemented Closure)

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-212 | Design Phase 2.1 tag/link/delete mutation UX and state flow | Done | locked state transitions, delete confirmation policy, validation mapping, and deterministic post-delete selection behavior in `docs/SPEC.md` |
| L-213 | Wire GUI tag add/remove actions to shared tag services | Done | added GUI tag editor add/remove actions in `src/gui.rs` via shared tag services with save-failure recovery and deterministic detail refresh |
| L-214 | Wire GUI link add/remove actions to shared link services | Done | added GUI link editor add/remove actions in `src/gui.rs` via shared link services with save-failure recovery and deterministic detail refresh |
| L-215 | Wire GUI figure delete flow to shared delete service | Done | added explicit delete-confirmation GUI flow via shared delete service with deterministic next/previous/clear post-delete selection |
| L-216 | Add GUI regression coverage for tag/link/delete flows | Done | added GUI regression tests for mutation state transitions, failure recovery, and deterministic list/detail behavior across tag/link/delete |
| L-217 | Run full verification gate for Phase 2.1 mutation expansion | Done | passed `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test` after L-213..L-216 |
| L-218 | Design Phase 2.2 drag-and-drop ingest UX and provenance prompts | Done | locked drop-session state flow, provenance prompt policy, deterministic multi-file commit ordering, and shared ingest-core reuse in `docs/SPEC.md` |

## Phase 2.2 GUI Drag-and-Drop Ingest (Next)

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-219 | Implement drop-session state machine in GUI | Done | added drag-and-drop session state model in `src/gui.rs`, captured dropped file paths from `egui` input, and covered deterministic transition behavior with GUI unit tests |
| L-220 | Wire dropped files to shared ingest core services | Done | wired drop-session commit path to shared `inject_figure` services with deterministic one-or-many batch processing, duplicate handling, and partial-failure reporting |
| L-221 | Implement provenance prompt defaults and per-item overrides | Done | added batch-level provenance defaults with per-item override resolution in `src/gui.rs`; commit remains blocked until each item has complete required metadata |
| L-222 | Add GUI regression coverage for deterministic multi-file ingest behavior | Done | added deterministic drop-commit ordering, stable per-item status mapping, and commit-failure recovery retry coverage in `src/gui.rs` tests |
| L-223 | Run full verification gate for Phase 2.2 ingest implementation | Done | passed `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test` after L-219..L-222 |

## Phase 1.x Hardening (Post-Phase 1.5 Review)

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-171 | Harden bundle import path portability | Done | reject non-portable reference paths (absolute/UNC/drive/parent traversal) with CLI coverage |
| L-172 | Reuse domain validation in bundle import | Done | apply inject/tag/link validation rules with normalization and CLI coverage |
| L-173 | Add link-loss visibility in bundle import | Done | added outbound link-loss counters and optional strict failure mode via `bundle import --fail-on-link-loss` |
| L-174 | Stream bundle export/import IO | Done | managed-file export/import now streams archive/file reads and avoids full managed payload buffering |
| L-175 | Tighten archive structural validation | Done | reject duplicate manifest/metadata entries, unexpected archive records, and non-regular tar members during import preflight |
| L-176 | Resolve numeric ID/name ambiguity | Done | added `--reference-mode auto|id|name` for `query` and `collection` reference operations with CLI coverage |
| L-177 | Add vault integrity verification command | Done | implemented read-only `verify` command for missing files, hash drift, and size drift with CLI integration tests |
| L-178 | Add bundle preflight planning | Done | implemented `bundle inspect` validation summary and non-mutating `bundle import --dry-run` projection output with CLI tests |
| L-179 | Add explicit bundle conflict policies | Done | implemented `bundle import --on-conflict skip|error|replace` with deterministic skip/error/replace behaviors and CLI tests |
| L-180 | Refresh user docs for implemented scope | Done | aligned README/USAGE/spec command coverage with implemented verify + bundle preflight/conflict controls |
| L-181 | Fix tag rename descendant corruption path | Done | rename computes full plan before mutation and applies updates by `tag_id`; panic-prone assumption removed |
| L-182 | Allow self-link cleanup via `link remove` | Done | removed self-link guard from removal path; add path still blocks self-links |
| L-183 | Define Rust toolchain compatibility policy | Done | crate edition downgraded to 2021 and incompatible let-chains rewritten for Rust 2021 |
| L-184 | Centralize vault DB connection opening | Done | introduced shared `db::open_vault_connection` and migrated command services to use one canonical open path |
| L-185 | Reduce N+1 query patterns in export/query-full | Done | query full detail and export now batch-load grouped related records instead of per-figure round trips |
| L-186 | Add caption clearing semantics | Done | implemented `update --clear-caption` and explicit conflict error when combined with `--caption` |
| L-187 | Enforce link uniqueness at schema level | Done | added migration v5 to dedupe legacy duplicate links and enforce unique business key index |
| L-188 | Deduplicate tag validation logic | Done | extracted shared `tag_validation` module and reused it in tag/search/query with preserved error semantics |
| L-189 | Improve bundle import crash consistency | Done | bundle import now stages managed files and uses journaled recovery before promotion to final paths |
| L-190 | Extend doctor for file-path integrity | Done | missing/non-regular file checks added with CLI coverage; full gate passed |

## Phase 1.x CLI Expansion Backlog (Post-Review)

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-191 | Add `show`/`info` command for single figure detail | Done | implemented `show` with `info` alias, full metadata output, and `cli_show` integration tests |
| L-192 | Add `list`/`ls` command for figures | Done | implemented `list` with `ls` alias and `--sort/--order/--limit` plus `cli_list` integration tests |
| L-193 | Add figure delete command | Done | implemented `delete` with transactional dependency cleanup, orphan-tag pruning, and managed-file cleanup policy |
| L-194 | Add `open` command for figure file | Done | implemented `open <figure_id>` with resolved path launch in OS viewer and CLI integration tests |
| L-195 | Add hierarchical tag-prefix search mode | Done | implemented `search --tag-prefix` with hierarchical prefix filtering and CLI integration tests |
| L-196 | Add source metadata update command | Done | implemented `source update` for source metadata field set/clear operations with CLI integration tests |
| L-197 | Allow filterless saved queries | Done | `query save` accepts sort/order/limit-only definitions with filterless save/run integration coverage |
| L-198 | Add JSON output mode to Phase 1 commands | Done | global `--json` provides machine-friendly envelopes for inject/update/tag/link/search/export with CLI integration coverage |
| L-199 | Add `tag list` command | Done | implemented deterministic `tag list` output (human + JSON) with per-tag figure counts and integration coverage |
| L-200 | Add collection update command | Done | implemented `collection update` for rename/query-binding/mode changes with payload validation and integration coverage |

## Phase 1.8 Wave 2 Slice Plan (P4-418 / L-188)

| Step | Status | Notes |
| --- | --- | --- |
| [x] W2-S1 | Done | Added shared helper module at `project4/lamian/src/tag_validation.rs` with preserved normalization + validation semantics |
| [x] W2-S2 | Done | Refactored `project4/lamian/src/tag.rs`, `project4/lamian/src/search.rs`, and `project4/lamian/src/query.rs` to reuse shared helper |
| [x] W2-S3 | Done | Ran `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test` in `project4/lamian` (all pass) |
| [x] W2-S4 | Done | Updated project + standalone TODO trackers and review notes for Wave 2 completion |

## Phase 1.8 Wave 3 Slice Plan (P4-419 / L-189)

| Step | Status | Notes |
| --- | --- | --- |
| [x] W3-S1 | Done | Added staged managed-file write flow for bundle import to avoid final-path writes before DB commit |
| [x] W3-S2 | Done | Added bundle import journal + startup recovery path for committed/staged states |
| [x] W3-S3 | Done | Added bundle journal recovery unit tests covering committed and staged scenarios |
| [x] W3-S4 | Done | Ran `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test` in `project4/lamian` (all pass) |

## Phase 1.8 Wave 4 Slice Plan (P4-420 / L-190)

| Step | Status | Notes |
| --- | --- | --- |
| [x] W4-S1 | Done | Reviewed `doctor` implementation and aligned with existing issue-collection pattern |
| [x] W4-S2 | Done | Added file-path integrity checks for missing/non-regular figure files (read-only) |
| [x] W4-S3 | Done | Full gate passed: `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, `cargo test` |
| [x] W4-S4 | Done | Updated TODO trackers and review notes for Wave 4 completion |

## Verification Gate

- `cargo fmt --all`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- migration compatibility checks for new schema versions

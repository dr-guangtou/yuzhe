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
| L-201 | Select Rust GUI approach | Pending | `egui`, `Iced`, or `Tauri` decision |
| L-202 | Build vault browser and figure detail editor | Pending | prioritize functionality over styling |

## Phase 1.x Hardening (Post-Phase 1.5 Review)

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-171 | Harden bundle import path portability | Pending | detect and report non-portable reference-mode file paths imported from bundles |
| L-172 | Reuse domain validation in bundle import | Pending | apply inject/tag/link validation rules to imported metadata records |
| L-173 | Add link-loss visibility in bundle import | Pending | report dropped outbound links and optionally fail when link targets are missing |
| L-174 | Stream bundle export/import IO | Pending | avoid full-file buffering for large managed files and archive entries |
| L-175 | Tighten archive structural validation | Pending | reject duplicate/ambiguous manifest or metadata entries and unexpected archive records |
| L-176 | Resolve numeric ID/name ambiguity | Pending | add explicit reference mode for `query` and `collection` operations (`id` vs `name`) |
| L-177 | Add vault integrity verification command | Pending | implement `verify` command for filesystem-vs-DB checks (missing files, hash drift, size drift) |
| L-178 | Add bundle preflight planning | Pending | implement `bundle inspect` and `bundle import --dry-run` summaries before mutation |
| L-179 | Add explicit bundle conflict policies | Pending | implement `bundle import --on-conflict skip|error|replace` |
| L-180 | Refresh user docs for implemented scope | Pending | align `README.md`, `USAGE.md`, and command coverage text with current state |
| L-181 | Fix tag rename descendant corruption path | Done | rename computes full plan before mutation and applies updates by `tag_id`; panic-prone assumption removed |
| L-182 | Allow self-link cleanup via `link remove` | Done | removed self-link guard from removal path; add path still blocks self-links |
| L-183 | Define Rust toolchain compatibility policy | Done | crate edition downgraded to 2021 and incompatible let-chains rewritten for Rust 2021 |
| L-184 | Centralize vault DB connection opening | Done | introduced shared `db::open_vault_connection` and migrated command services to use one canonical open path |
| L-185 | Reduce N+1 query patterns in export/query-full | Done | query full detail and export now batch-load grouped related records instead of per-figure round trips |
| L-186 | Add caption clearing semantics | Done | implemented `update --clear-caption` and explicit conflict error when combined with `--caption` |
| L-187 | Enforce link uniqueness at schema level | Done | added migration v5 to dedupe legacy duplicate links and enforce unique business key index |
| L-188 | Deduplicate tag validation logic | Done | extracted shared `tag_validation` module and reused it in tag/search/query with preserved error semantics |
| L-189 | Improve bundle import crash consistency | Pending | reduce orphan-file risk around DB commit and file writes via staged writes/journaling (from WEAKNESS-10) |
| L-190 | Extend doctor for file-path integrity | Pending | add checks for missing/broken figure file paths (partially overlaps L-177; from WEAKNESS-8) |

## Phase 1.x CLI Expansion Backlog (Post-Review)

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-191 | Add `show`/`info` command for single figure detail | Pending | return full metadata for one figure ID (from MISS-1) |
| L-192 | Add `list`/`ls` command for figures | Pending | browse all figures with optional limit/sort (from MISS-2) |
| L-193 | Add figure delete command | Pending | safe remove flow with dependency cleanup policy (from MISS-3) |
| L-194 | Add `open` command for figure file | Pending | open resolved file path in OS viewer (from MISS-4) |
| L-195 | Add hierarchical tag-prefix search mode | Pending | support prefix matching for tag trees (from MISS-5) |
| L-196 | Add source metadata update command | Pending | update `source_title`/`source_authors`/`source_published_at` after ingest (from MISS-6) |
| L-197 | Allow filterless saved queries | Pending | support sort+limit-only query definitions (from MISS-7) |
| L-198 | Add JSON output mode to Phase 1 commands | Pending | optional `--json` for inject/update/tag/link/search/export (from MISS-8) |
| L-199 | Add `tag list` command | Pending | enumerate vault tags without full export (from MISS-9) |
| L-200 | Add collection update command | Pending | rename collection and/or retarget dynamic query binding (from MISS-10) |

## Phase 1.8 Wave 2 Slice Plan (P4-418 / L-188)

| Step | Status | Notes |
| --- | --- | --- |
| [x] W2-S1 | Done | Added shared helper module at `project4/lamian/src/tag_validation.rs` with preserved normalization + validation semantics |
| [x] W2-S2 | Done | Refactored `project4/lamian/src/tag.rs`, `project4/lamian/src/search.rs`, and `project4/lamian/src/query.rs` to reuse shared helper |
| [x] W2-S3 | Done | Ran `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test` in `project4/lamian` (all pass) |
| [x] W2-S4 | Done | Updated project + standalone TODO trackers and review notes for Wave 2 completion |

## Verification Gate

- `cargo fmt --all`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- migration compatibility checks for new schema versions

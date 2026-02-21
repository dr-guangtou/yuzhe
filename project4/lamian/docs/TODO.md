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

## Verification Gate

- `cargo fmt --all`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- migration compatibility checks for new schema versions

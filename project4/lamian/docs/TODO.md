# LaMian TODO (Standalone)

## Phase 1: CLI Core

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-101 | Initialize Rust CLI crate | Done | `init` command available |
| L-102 | Add SQLite migration framework | Done | v1 schema and migration table implemented |
| L-103 | Implement `inject` with strict provenance validation | Done | shared ingest core added with typed validation, copy/reference handling, and transactional persistence |
| L-104 | Implement `update`, `tag`, `link`, `search`, `export` logic | In Progress | `update` (name/caption/note-file), `tag` actions, `link add/remove`, and `search` (tag/source/text filters) implemented with typed validation; `export` pending |
| L-105 | Add integration tests for CLI workflow | In Progress | real-fixture CLI tests now cover `inject`, `update`, tag operations, link operations, and search filter/empty-result paths; full workflow still pending |

## Phase 2: Domain and Validation

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-201 | Add domain models for figure/source/tag/link | Pending | keep schema and model parity |
| L-202 | Add tag normalization and hierarchy checks | In Progress | lowercase + delimiter validation and hierarchy-aware rename implemented for `tag`; broader workflows pending |
| L-203 | Add link parser for `[[figure_id]]` references | Pending | normalize and persist parsed links |

## Phase 3: GUI Baseline

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-301 | Select Rust GUI approach | Pending | `egui`, `Iced`, or `Tauri` decision |
| L-302 | Build vault browser and figure detail editor | Pending | prioritize functionality over styling |

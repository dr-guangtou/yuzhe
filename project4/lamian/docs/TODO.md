# LaMian TODO (Standalone)

## Phase 1: CLI Core

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-101 | Initialize Rust CLI crate | Done | `init` command available |
| L-102 | Add SQLite migration framework | Done | v1 schema and migration table implemented |
| L-103 | Implement `inject` with strict provenance validation | Pending | requires file mode and source validation |
| L-104 | Implement `update`, `tag`, `link`, `search`, `export` logic | In Progress | command definitions exist; handlers pending |
| L-105 | Add integration tests for CLI workflow | Pending | init -> inject -> tag -> search path |

## Phase 2: Domain and Validation

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-201 | Add domain models for figure/source/tag/link | Pending | keep schema and model parity |
| L-202 | Add tag normalization and hierarchy checks | Pending | enforce lowercase and delimiter rules |
| L-203 | Add link parser for `[[figure_id]]` references | Pending | normalize and persist parsed links |

## Phase 3: GUI Baseline

| ID | Task | Status | Notes |
| --- | --- | --- | --- |
| L-301 | Select Rust GUI approach | Pending | `egui`, `Iced`, or `Tauri` decision |
| L-302 | Build vault browser and figure detail editor | Pending | prioritize functionality over styling |


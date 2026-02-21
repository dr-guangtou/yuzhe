# LaMian TODO

## Planning Checklist

| ID | Task | Owner | Status | Notes |
| --- | --- | --- | --- | --- |
| P4-001 | Create documentation skeleton in `project4/` | agent | [x] Done | README/PLAN/SPEC/TODO created |
| P4-002 | Write decision-complete technical spec | agent | [x] Done | `SPEC.md` updated for Phase 1.5 scope and interfaces |
| P4-003 | Establish risk register and architecture decisions | agent | [x] Done | `RISK_REGISTER.md` and `DECISIONS.md` synchronized |
| P4-004 | Initialize project-specific agent guidance | agent | [x] Done | `AGENTS.md` and `CLAUDE.md` present |
| P4-005 | Verify cross-file consistency | agent | [ ] In Progress | Phase 1.5 sync pass in progress across project and standalone docs |

## Implementation Backlog (Phase 1 Completed)

| ID | Task | Owner | Status | Notes |
| --- | --- | --- | --- | --- |
| P4-101 | Initialize Rust workspace in `project4/lamian/` | agent | [x] Done | crate and toolchain baseline complete |
| P4-102 | Add SQLite migration framework | agent | [x] Done | migrations v1/v2 implemented |
| P4-103 | Implement core CLI commands (`init`, `inject`, `update`, `tag`, `link`, `search`, `export`) | agent | [x] Done | full Phase 1 command surface implemented |
| P4-104 | Add command-focused integration coverage | agent | [x] Done | tests for inject/update/tag/link/search/export behaviors |
| P4-105 | Add full sequential CLI integration flow | agent | [ ] Pending | `init -> inject -> update -> tag -> link -> search -> export` still needs one explicit test suite |

## Implementation Backlog (Phase 1.5 Wave A)

| ID | Task | Owner | Status | Notes |
| --- | --- | --- | --- | --- |
| P4-201 | Add migration v3 for `saved_queries` | agent | [x] Done | schema/index added with migration-aware test coverage |
| P4-202 | Implement `query save|run|list|delete` | agent | [x] Done | JSON-only output implemented with `run --detail ids|full` and integration tests |
| P4-203 | Implement `import` batch ingest | agent | [x] Done | strict provenance template + continue-on-error summary + duplicate skip/report implemented |
| P4-204 | Implement `doctor` checks + DB-only `--fix` | agent | [x] Done | deterministic checks implemented; `--fix` limited to DB-only safe fixes |
| P4-205 | Add Wave A integration tests | agent | [x] Done | query/import/doctor integration coverage added |

## Implementation Backlog (Phase 1.5 Wave B)

| ID | Task | Owner | Status | Notes |
| --- | --- | --- | --- | --- |
| P4-301 | Add migration v4 for collections | agent | [x] Done | migration v4 added with `collections` + `collection_items` schema, constraints, and indexes |
| P4-302 | Implement `collection` hybrid mode | agent | [x] Done | `collection create/add/remove/list/delete` implemented with static/dynamic behavior and JSON output |
| P4-303 | Implement `bundle export|import` (`tar.gz`) | agent | [ ] Pending | metadata + managed files; skip existing conflicts on import |
| P4-304 | Add Wave B integration tests | agent | [ ] In Progress | collection integration coverage added; bundle roundtrip/conflict/corruption pending |

## Wave B Slice Plan (Current Session)

| Step | Status | Notes |
| --- | --- | --- |
| [x] WB-S1 | Done | Added migration v4 for `collections` and `collection_items` in `project4/lamian/src/db.rs` with constraints/indexes |
| [x] WB-S2 | Done | Extended migration tests to assert new tables exist and schema version bump is reflected |
| [x] WB-S3 | Done | Ran `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test` (all pass) |
| [x] WB-S4 | Done | Updated TODO/docs status for completed Wave B migration slice |

## Wave B Slice Plan (Collection Command Slice)

| Step | Status | Notes |
| --- | --- | --- |
| [x] WB2-S1 | Done | Implemented `collection` core service (`create/add/remove/list/delete`) in `project4/lamian/src/collection.rs` |
| [x] WB2-S2 | Done | Wired `Command::Collection` in `src/cli.rs` and `src/commands.rs` with JSON output contracts |
| [x] WB2-S3 | Done | Added `project4/lamian/tests/cli_collection.rs` for static lifecycle and dynamic query-bound behavior |
| [x] WB2-S4 | Done | Ran full gate (`cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`) |

## Verification Plan

- Run for implementation slices:
  - `cargo fmt --all`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Add migration compatibility checks from existing Phase 1 vaults.
- Keep project and standalone docs synchronized in each wave.

## Review Notes

- 2026-02-19: Documentation bootstrap completed.
- 2026-02-20: Phase 1 core commands completed and merged.
- 2026-02-20: Search, update, and export finalized with integration tests.
- 2026-02-20: Phase 1.5 plan approved: two-wave delivery for `query/import/doctor` then `collection/bundle`.
- 2026-02-20: Implemented Phase 1.5 query foundation (`saved_queries` migration + `query save/run/list/delete` + CLI integration tests).
- 2026-02-21: Implemented Phase 1.5 import slice (`import` core + CLI JSON output + `cli_import` integration coverage).
- 2026-02-21: Implemented Phase 1.5 doctor slice (`doctor` checks + DB-only `--fix` + `cli_doctor` integration coverage); Wave A completed.
- 2026-02-21: Started Wave B with migration slice (`collections` + `collection_items` via v4) and passed full gate.
- 2026-02-21: Implemented Wave B collection command slice (`collection create/add/remove/list/delete`) with integration tests.

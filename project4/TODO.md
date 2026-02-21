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
| P4-105 | Add full sequential CLI integration flow | agent | [x] Done | `cli_workflow` added for `init -> inject -> update -> tag -> link -> search -> export` |

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
| P4-303 | Implement `bundle export|import` (`tar.gz`) | agent | [x] Done | deterministic manifest/checksum + managed-file transfer + skip-existing conflict policy |
| P4-304 | Add Wave B integration tests | agent | [x] Done | collection and bundle (roundtrip/conflict/corruption) integration coverage merged |

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

## Verification Plan

- Run for implementation slices:
  - `cargo fmt --all`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Add migration compatibility checks from existing Phase 1 vaults.
- Keep project and standalone docs synchronized in each wave.

## Hardening Backlog (Post-Phase 1.5 Review)

| ID | Task | Owner | Status | Notes |
| --- | --- | --- | --- | --- |
| P4-401 | Bundle import portability validation | agent | [ ] Pending | detect/report non-portable reference paths imported from bundle metadata |
| P4-402 | Bundle import domain validation parity | agent | [ ] Pending | apply inject/tag/link normalization and validation paths during bundle import |
| P4-403 | Bundle link-loss reporting policy | agent | [ ] Pending | include dropped-link counters and optional strict-fail mode |
| P4-404 | Streaming bundle IO | agent | [ ] Pending | avoid full in-memory buffering for large bundle entries |
| P4-405 | Bundle archive hardening | agent | [ ] Pending | reject duplicate manifest/metadata entries and unsupported tar members |
| P4-406 | Query/collection numeric reference disambiguation | agent | [ ] Pending | add explicit `id` vs `name` reference mode in CLI |
| P4-407 | Vault integrity verification command | agent | [ ] Pending | add core `verify` command for file existence/hash/size drift checks |
| P4-408 | Bundle preflight commands | agent | [ ] Pending | add `bundle inspect` and `bundle import --dry-run` |
| P4-409 | Bundle conflict policy controls | agent | [ ] Pending | add `bundle import --on-conflict skip|error|replace` |
| P4-410 | Documentation alignment pass | agent | [ ] Pending | sync README/USAGE/spec language to implemented Phase 1.5 scope |
| P4-411 | Fix tag rename descendant corruption | agent | [x] Done | rename now uses precomputed plan + `tag_id` updates; regression covered for prefix-expansion case |
| P4-412 | Allow self-link cleanup in `link remove` | agent | [x] Done | removal path now permits self-link cleanup while add path still rejects self-links |
| P4-413 | Toolchain compatibility policy (edition/MSRV) | agent | [x] Done | downgraded crate edition to Rust 2021 and updated 2021-incompatible syntax |
| P4-414 | Shared DB connection helper rollout | agent | [x] Done | added `db::open_vault_connection` and migrated service modules to shared open + FK pragma path |
| P4-415 | Query/export performance batching | agent | [x] Done | replaced `query --detail full` and export per-figure N+1 loads with batched grouped fetches |
| P4-416 | Caption clear semantics | agent | [x] Done | added `update --clear-caption` with conflict validation against `--caption` and test coverage |
| P4-417 | Link uniqueness migration | agent | [x] Done | migration v5 dedupes legacy duplicate links and enforces UNIQUE business key via index |
| P4-418 | Shared tag validation module | agent | [x] Done | shared `tag_validation` module now powers tag/search/query validation with preserved error semantics |
| P4-419 | Bundle import crash-consistency hardening | agent | [x] Done | implemented staged writes + import journal recovery to reduce orphan-file window around DB commit |
| P4-420 | Doctor file-path integrity checks | agent | [ ] Pending | detect missing files referenced by figure records (from WEAKNESS-8) |

## CLI Expansion Backlog (From Independent Review)

| ID | Task | Owner | Status | Notes |
| --- | --- | --- | --- | --- |
| P4-421 | Add `show`/`info` figure command | agent | [ ] Pending | inspect one figure's full metadata from CLI (from MISS-1) |
| P4-422 | Add figure `list`/`ls` command | agent | [ ] Pending | list all figures with optional sorting/limit (from MISS-2) |
| P4-423 | Add figure delete command | agent | [ ] Pending | complete core CRUD with safe deletion semantics (from MISS-3) |
| P4-424 | Add figure `open` command | agent | [ ] Pending | open figure path in system viewer (from MISS-4) |
| P4-425 | Add tag-prefix search capability | agent | [ ] Pending | leverage hierarchical tag model in search (`--tag-prefix`) (from MISS-5) |
| P4-426 | Add source metadata update command | agent | [ ] Pending | update source metadata fields after ingest (from MISS-6) |
| P4-427 | Allow filterless saved queries | agent | [ ] Pending | support sort+limit-only query templates (from MISS-7) |
| P4-428 | Add JSON option for Phase 1 commands | agent | [ ] Pending | machine-friendly output parity with Phase 1.5 (from MISS-8) |
| P4-429 | Add `tag list` command | agent | [ ] Pending | enumerate known tags without full export (from MISS-9) |
| P4-430 | Add collection update command | agent | [ ] Pending | rename and/or change collection mode/query binding (from MISS-10) |

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
- 2026-02-21: Completed Wave B bundle slice and Phase 1 sequential workflow test (`L-105`).
- 2026-02-21: Added post-milestone hardening backlog from thorough review (bundle import hardening, ambiguity resolution, integrity verification).
- 2026-02-21: Completed Phase 1.8 Wave 1 (`BUG-2`, `BUG-3`, edition downgrade to 2021) with full gate pass and new regression tests.
- 2026-02-21: Completed Phase 1.8 Wave 2 (`P4-418`/`L-188`) by extracting shared tag validation and reusing it across tag/search/query with full gate pass.
- 2026-02-21: Completed Phase 1.8 Wave 3 (`P4-419`/`L-189`) by moving bundle import to staged writes with journaled recovery and full gate pass.

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

## Phase 1.8 Wave 4 Slice Plan (P4-420 / L-190)

| Step | Status | Notes |
| --- | --- | --- |
| [x] W4-S1 | Done | Reviewed `doctor` implementation and aligned with existing issue-collection pattern |
| [x] W4-S2 | Done | Added file-path integrity checks for missing/non-regular figure files (read-only) |
| [x] W4-S3 | Done | Full gate passed: `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, `cargo test` |
| [x] W4-S4 | Done | Updated TODO trackers and review notes for Wave 4 completion |

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
| P4-401 | Bundle import portability validation | agent | [x] Done | reject non-portable reference paths (absolute/UNC/drive/parent traversal) with CLI coverage |
| P4-402 | Bundle import domain validation parity | agent | [x] Done | validated source types/keys, tags, and link relations with CLI parity + normalization |
| P4-403 | Bundle link-loss reporting policy | agent | [x] Done | added outbound link-loss counters and `bundle import --fail-on-link-loss` strict mode with rollback semantics |
| P4-404 | Streaming bundle IO | agent | [x] Done | export/import managed files now stream file/archive reads instead of buffering full managed payloads in memory |
| P4-405 | Bundle archive hardening | agent | [x] Done | reject duplicate manifest/metadata, unexpected entries, and non-regular tar members during import preflight |
| P4-406 | Query/collection numeric reference disambiguation | agent | [x] Done | added `--reference-mode auto|id|name` for query run/delete and collection add/remove/list/delete with CLI coverage |
| P4-407 | Vault integrity verification command | agent | [x] Done | added read-only `verify` command for file existence/hash/size drift checks with CLI integration coverage |
| P4-408 | Bundle preflight commands | agent | [x] Done | added `bundle inspect` and `bundle import --dry-run` with validated summary/projection outputs and CLI coverage |
| P4-409 | Bundle conflict policy controls | agent | [x] Done | added `bundle import --on-conflict skip|error|replace` with deterministic skip/error/replace behavior and CLI coverage |
| P4-410 | Documentation alignment pass | agent | [x] Done | synchronized README/USAGE/spec language to implemented Phase 1.5 + Phase 1.x hardening scope |
| P4-411 | Fix tag rename descendant corruption | agent | [x] Done | rename now uses precomputed plan + `tag_id` updates; regression covered for prefix-expansion case |
| P4-412 | Allow self-link cleanup in `link remove` | agent | [x] Done | removal path now permits self-link cleanup while add path still rejects self-links |
| P4-413 | Toolchain compatibility policy (edition/MSRV) | agent | [x] Done | downgraded crate edition to Rust 2021 and updated 2021-incompatible syntax |
| P4-414 | Shared DB connection helper rollout | agent | [x] Done | added `db::open_vault_connection` and migrated service modules to shared open + FK pragma path |
| P4-415 | Query/export performance batching | agent | [x] Done | replaced `query --detail full` and export per-figure N+1 loads with batched grouped fetches |
| P4-416 | Caption clear semantics | agent | [x] Done | added `update --clear-caption` with conflict validation against `--caption` and test coverage |
| P4-417 | Link uniqueness migration | agent | [x] Done | migration v5 dedupes legacy duplicate links and enforces UNIQUE business key via index |
| P4-418 | Shared tag validation module | agent | [x] Done | shared `tag_validation` module now powers tag/search/query validation with preserved error semantics |
| P4-419 | Bundle import crash-consistency hardening | agent | [x] Done | implemented staged writes + import journal recovery to reduce orphan-file window around DB commit |
| P4-420 | Doctor file-path integrity checks | agent | [x] Done | added missing/non-regular file checks with CLI coverage; full gate passed |

## CLI Expansion Backlog (From Independent Review)

| ID | Task | Owner | Status | Notes |
| --- | --- | --- | --- | --- |
| P4-421 | Add `show`/`info` figure command | agent | [x] Done | implemented `show` with `info` alias for full single-figure metadata output and integration coverage |
| P4-422 | Add figure `list`/`ls` command | agent | [x] Done | implemented `list` with `ls` alias and `--sort/--order/--limit` plus integration coverage |
| P4-423 | Add figure delete command | agent | [x] Done | implemented top-level `delete` with transactional dependency cleanup, orphan-tag pruning, and managed-file cleanup policy |
| P4-424 | Add figure `open` command | agent | [x] Done | implemented `open <figure_id>` with resolved path launch via OS viewer semantics and CLI integration coverage |
| P4-425 | Add tag-prefix search capability | agent | [x] Done | added `search --tag-prefix` hierarchical matching (`prefix:%`) with CLI integration coverage |
| P4-426 | Add source metadata update command | agent | [x] Done | added `source update` command for `source_title`/`source_authors`/`source_published_at` with integration coverage |
| P4-427 | Allow filterless saved queries | agent | [x] Done | `query save` now accepts sort/order/limit-only templates with integration coverage for filterless `save` + `run` |
| P4-428 | Add JSON option for Phase 1 commands | agent | [x] Done | added global `--json` envelopes for `inject`/`update`/`tag`/`link`/`search`/`export` with CLI integration coverage |
| P4-429 | Add `tag list` command | agent | [x] Done | added `tag list` with deterministic tag ordering + figure counts in human/JSON outputs and CLI integration coverage |
| P4-430 | Add collection update command | agent | [x] Done | added `collection update` for rename and query binding/mode changes with validation and CLI integration coverage |

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
- 2026-02-24: Completed Phase 1.8 Wave 4 (`P4-420`/`L-190`) with doctor file-path integrity checks, CLI coverage, and full gate pass.
- 2026-02-24: Completed bundle import portability validation (`P4-401`) with reference-path checks and CLI coverage.
- 2026-02-24: Completed bundle import domain validation parity (`P4-402`) with source/tag/link validation and CLI coverage.
- 2026-02-24: Completed bundle link-loss reporting policy (`P4-403`) with counters in `bundle.import` result and optional strict fail mode (`--fail-on-link-loss`).
- 2026-02-24: Completed streaming bundle IO (`P4-404`) by switching managed-file export/import paths to streaming reads/writes with full gate pass.
- 2026-02-24: Completed bundle archive hardening (`P4-405`) with strict preflight checks for duplicate manifest/metadata entries, unexpected archive entries, and non-regular tar members.
- 2026-02-24: Completed query/collection reference disambiguation (`P4-406`) by adding explicit `--reference-mode auto|id|name` with numeric-name ambiguity coverage.
- 2026-02-24: Completed vault integrity verification (`P4-407`) with read-only missing-file/hash-drift/size-drift checks and `cli_verify` integration coverage.
- 2026-02-24: Completed bundle preflight commands (`P4-408`) by adding `bundle inspect` and non-mutating `bundle import --dry-run` with CLI integration coverage.
- 2026-02-24: Completed bundle conflict policy controls (`P4-409`) by adding `bundle import --on-conflict skip|error|replace` and CLI coverage for error/replace semantics.
- 2026-02-24: Completed documentation alignment pass (`P4-410`) by syncing README/USAGE/spec docs with implemented `verify`, bundle preflight, and bundle conflict policy controls.
- 2026-02-24: Completed `show`/`info` single-figure detail command (`P4-421`) with full metadata output, alias wiring, and `cli_show` integration tests.
- 2026-02-24: Completed `list`/`ls` figure command (`P4-422`) with sortable/limited human-readable output and `cli_list` integration tests.
- 2026-02-24: Completed figure delete command (`P4-423`) with transactional cascade cleanup, orphan-tag pruning, managed-file deletion policy, and `cli_delete` integration tests.
- 2026-02-24: Completed figure `open` command (`P4-424`) by resolving stored figure paths and launching them through OS viewer semantics with `cli_open` integration tests.
- 2026-02-24: Completed hierarchical tag-prefix search (`P4-425`) with `search --tag-prefix` filtering and `cli_search` integration tests.
- 2026-02-24: Completed source metadata update command (`P4-426`) with `source update` field set/clear semantics and `cli_source` integration tests.
- 2026-02-24: Completed filterless saved queries (`P4-427`) by allowing sort/order/limit-only `query save` templates and adding CLI integration coverage for deterministic filterless `query run`.
- 2026-02-24: Completed Phase 1 JSON output mode (`P4-428`) by adding global `--json` output envelopes for `inject`/`update`/`tag`/`link`/`search`/`export` and integration coverage in `cli_json`.
- 2026-02-24: Completed `tag list` (`P4-429`) with deterministic ordering, per-tag figure counts, human-readable + `--json` output paths, and `cli_tag_list` integration tests.
- 2026-02-24: Completed `collection update` (`P4-430`) with rename/query-binding update semantics (`--name`, `--query-id`, `--clear-query-id`) and `cli_collection` integration tests.

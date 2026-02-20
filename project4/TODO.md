# LaMian TODO

## Planning Checklist

| ID | Task | Owner | Status | Notes |
| --- | --- | --- | --- | --- |
| P4-001 | Create documentation skeleton in `project4/` | agent | [x] Done | README/PLAN/SPEC/TODO created |
| P4-002 | Write decision-complete technical spec | agent | [x] Done | `SPEC.md` drafted with FR/NFR and interfaces |
| P4-003 | Establish risk register and architecture decisions | agent | [x] Done | `RISK_REGISTER.md` and `DECISIONS.md` created |
| P4-004 | Initialize project-specific agent guidance | agent | [x] Done | `AGENTS.md` and `CLAUDE.md` created |
| P4-005 | Create detailed planning journal entry | agent | [x] Done | `journal/2026-02-19.md` created |
| P4-006 | Verify cross-file consistency | agent | [x] Done | verified on 2026-02-19 via keyword and file inventory checks |
| P4-007 | User review and sign-off on MVP boundaries | user + agent | [ ] Pending | confirm unresolved decisions |

## Implementation Backlog (Phase 1)

| ID | Task | Owner | Status | Notes |
| --- | --- | --- | --- | --- |
| P4-101 | Initialize Rust workspace in `project4/lamian/` | agent | [x] Done | Rust crate initialized with build/test workflow |
| P4-102 | Add SQLite migration framework | agent | [x] Done | migration table + v1 schema + idempotent init tests |
| P4-103 | Implement domain models and validation | agent | [ ] In Progress | strict provenance and `tag` normalization/validation are implemented in shared core; remaining domain models pending |
| P4-104 | Implement CLI commands (`init`, `inject`, `tag`, `link`, `search`, `export`) | agent | [ ] In Progress | `init`, `inject`, full `tag`, and `link add/remove` implemented; `search/export/update` remain stubs |
| P4-105 | Add integration tests for end-to-end CLI workflow | agent | [ ] In Progress | CLI tests now cover `inject`, `tag` operations, and `link` operations with success/failure cases; broader workflow coverage pending |
| P4-106 | Prepare `lamian/` for standalone repository extraction | agent | [x] Done | local docs, license, toolchain pin, and migration checklist added |
| P4-107 | Define shared ingest-core rule for CLI and GUI drag-and-drop | agent | [x] Done | captured in spec and decision logs |
| P4-108 | Add context-window and handover rules for multi-session development | agent | [x] Done | rules added to Project 4 and LaMian AGENTS/CLAUDE files |
| P4-109 | Close remaining Phase 1 command gaps (`update`, `search`, `export`) | agent | [ ] Pending | required before Phase 2 gate |
| P4-110 | Add full sequential CLI integration test (`init -> inject -> update -> tag -> link -> search -> export`) | agent | [ ] Pending | Phase 1 gate requires full workflow coverage |

## Verification Plan

- Validate requirement traceability from `SPEC.md` to this TODO list.
- Verify all planning files agree on:
  - Rust language choice
  - SQLite canonical storage with sidecar export
  - strict provenance policy
  - `project4/` as planning home and `project4/lamian/` as implementation home

## Review Notes

- 2026-02-19: Documentation bootstrap completed on branch `feature/project4-lamian-docs`.
- 2026-02-19: Cross-file consistency verified across planning and governance docs.
- 2026-02-19: Phase 1 bootstrap started in `project4/lamian/` with working `init` command and schema migrations.
- 2026-02-20: Extraction-readiness pass completed for `project4/lamian/`.
- 2026-02-20: Ingest architecture locked: GUI drag-and-drop must call the same core ingest service as CLI.
- 2026-02-20: Added explicit context-window sizing and handover rules to reduce context rot across sessions.
- 2026-02-20: Implemented shared `inject` core service with typed provenance validation, transactional `figures` + `sources` persistence, and new success/failure tests.
- 2026-02-20: Added CLI integration tests for real Desktop PNG fixtures and wrong-format negative fixture in `project4/lamian/tests/cli_inject_real.rs`.
- 2026-02-20: Implemented `tag add` shared core path with tag normalization/idempotent persistence and added `inject -> tag add` integration tests for success, duplicate, and invalid tags.
- 2026-02-20: Implemented `tag remove` and hierarchy-aware `tag rename` in shared core and added integration tests for remove success/unassigned and rename success/conflict paths.
- 2026-02-20: Implemented shared `link add/remove` core with typed validation and added integration tests for add/remove success, idempotency, unknown IDs, and self-link rejection.
- 2026-02-20: Daily review: plan updated to reflect late Phase 1 status and to prioritize `update/search/export` plus a full end-to-end CLI workflow test before Phase 2.

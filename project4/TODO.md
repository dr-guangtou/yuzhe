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
| P4-103 | Implement domain models and validation | agent | [ ] Pending | enforce provenance in `inject` path |
| P4-104 | Implement CLI commands (`init`, `inject`, `tag`, `link`, `search`, `export`) | agent | [ ] In Progress | `init` implemented, others are typed stubs |
| P4-105 | Add integration tests for end-to-end CLI workflow | agent | [ ] Pending | include failure modes |
| P4-106 | Prepare `lamian/` for standalone repository extraction | agent | [x] Done | local docs, license, toolchain pin, and migration checklist added |

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

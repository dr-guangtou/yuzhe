# LaMian Plan (Standalone Mirror)

## 1. Objective

Build LaMian as a local-only, metadata-rich visual knowledge base for research figures, with a CLI-first core and a desktop GUI built over the same shared services.

## 2. Planning Constraints

- Primary language: Rust (Rust 2021 compatibility required)
- Storage baseline: SQLite canonical metadata store
- GUI policy: interaction-layer only; domain validation and mutation semantics stay in shared core services
- Incubator parity: this mirror must stay aligned with `project4/PLAN.md`

## 3. Phase Status

### Phase 1 and Phase 1.5 (Completed)

- Core CLI commands, schema migrations, hardening waves, and expansion backlog are complete with test coverage.

### Phase 2.0 GUI Foundation (Completed)

- Read-only vault browser + detail slice delivered.
- Figure/source metadata mutation editors delivered with deterministic refresh behavior.
- Regression coverage and full gate verification completed.

### Phase 2.1 GUI Mutation Expansion (Completed)

- Added GUI mutation actions for shared `tag`, `link`, and `delete` services.
- Locked and implemented UX/state behavior for destructive actions (delete confirmation and deterministic post-delete selection).
- Added regression coverage for success/failure and deterministic list/detail refresh behavior and passed full gate verification.

### Phase 2.2 GUI Drag-and-Drop Ingest (Current)

- Add one-or-many file drop flow.
- Reuse shared ingest core and provenance validation semantics.
- Define provenance prompt behavior before ingest commit.

### Phase 2.3 GUI Workflow Parity Polish (Planned)

- Prioritize low-risk, high-value GUI workflow parity improvements.
- Keep automation-heavy workflows CLI-first unless GUI value is explicit.

## 4. Verification Strategy

- Run full gate after each implementation slice:
  - `cargo fmt --all`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Keep `docs/SPEC.md` and `docs/TODO.md` synchronized with delivered behavior.

## 5. Next Execution Steps

1. Implement Phase 2.2 drag-and-drop session state machine in `src/gui.rs`.
2. Wire one-or-many dropped files to shared ingest core with provenance defaults and per-item override prompts.
3. Add regression tests for deterministic multi-file ordering, stable per-item result reporting, and commit-failure recovery.
4. Run full gate (`cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`).

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

### Phase 2.2 GUI Drag-and-Drop Ingest (Implemented Closure)

- Delivered one-or-many file drop flow.
- Reused shared ingest core and provenance validation semantics.
- Closed with defined provenance prompt behavior before ingest commit.

### Phase 2.3 GUI Workflow Parity Polish (Current Planning Baseline)

- Expose high-use operational workflows in GUI where low risk and high leverage:
  - open file, list/detail navigation polish, and search/filter ergonomics
- Keep automation-centric flows (`bundle`, `verify`, bulk import) CLI-first unless GUI value is clear.
- Close parity gaps required for MVP “GUI can drive core workflows” gate.
- Keep Rust 2021 compatibility and deterministic ordering guarantees explicit in each planning slice and acceptance check.

Phase 2.3 planning slices:

- `L-224`: lock parity-polish UX/state-flow scope and acceptance contract in both SPEC mirrors before implementation.
- `L-225`: implement open-file parity and deterministic selection/focus-state stabilization.
- `L-226`: implement list/detail navigation and search/filter ergonomics while preserving deterministic ordering behavior.
- `L-227`: add regression coverage for deterministic parity-polish behaviors.
- `L-228`: pass full Rust gate (`cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`).

Phase 2.3 acceptance criteria:

- Scope and acceptance contract are documented in both SPEC mirrors before implementation starts.
- Open-file action behavior matches shared core semantics with deterministic state transitions.
- Navigation and search/filter polish paths preserve deterministic ordering for list rows, selection transitions, and refresh results.
- Implementation remains Rust 2021-compatible across GUI and tests.
- Full gate passes after implementation slices complete.

## 4. Verification Strategy

- Run full gate after each implementation slice:
  - `cargo fmt --all`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Keep `docs/SPEC.md` and `docs/TODO.md` synchronized with delivered behavior.

## 5. Next Execution Steps

1. Phase 2.2 implementation is closed after passing L-223 full gate (`cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`).
2. Complete Phase 2.3 planning and acceptance lock (`L-224`) across both tracker mirrors before implementation.
3. Keep Rust 2021 compatibility and deterministic ordering guarantees explicit in every Phase 2.3 implementation and test slice (`L-225`..`L-228`).

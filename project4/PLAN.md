# LaMian Plan

## 1. Objective

Build LaMian as a local-only, metadata-rich visual knowledge base for research figures, with a CLI-first core and a desktop GUI built over the same core services.

## 2. Planning Constraints

- Development target folder for code: `project4/lamian/`
- Planning and governance artifacts: `project4/`
- Primary language: Rust
- Storage baseline: SQLite as canonical metadata store, with sidecar export support
- Provenance policy: strict provenance required at ingest

## 3. Scope

### In Scope for MVP

- Vault initialization and management
- Figure ingest/inject pipeline
- Metadata create/read/update
- Hierarchical tags
- Search and filtering
- Figure-to-figure linking by stable IDs
- Metadata export
- Automation and curation commands needed before GUI
- Basic GUI that can drive core operations

### Out of Scope for MVP

- Automatic arXiv figure extraction
- Automatic publisher scraping workflow
- Built-in AI agent features
- Cloud sync and collaboration

## 4. Implementation Phases

## Phase 0: Documentation and Governance (Done)

- Create and validate planning documents in `project4/`
- Lock architecture, interfaces, and acceptance criteria
- Initialize project-specific agent instructions

## Phase 1: CLI Core and Data Model (Done)

- Initialize Rust workspace under `project4/lamian/`
- Implement schema migrations and repository layer
- Implement core CLI commands: `init`, `inject`, `update`, `tag`, `link`, `search`, `export`
- Add integration coverage for success/failure paths per command family

### Phase 1 Closure Notes

- Phase 1 command surface is complete in code.
- Project-level planning docs were stale and are now synchronized as part of Phase 1.5 kickoff.
- Remaining Phase 1 quality gate still required: one full sequential integration scenario (`init -> inject -> update -> tag -> link -> search -> export`) plus migration compatibility fixtures.

## Phase 1.5: Pre-GUI Automation and Curation CLI (Completed)

Purpose:

- lock high-leverage operational workflows before GUI
- prevent GUI-first requirements churn in core contracts
- provide automation-ready CLI primitives for agent usage

### Wave A (First)

- `query save|run|list|delete`
- `import` (batch ingest)
- `doctor` (`check` + `--fix` for DB-only safe fixes)

### Wave B (Second)

- `collection create|add|remove|list|delete` (hybrid static + dynamic)
- `bundle export|import` (`tar.gz`, metadata + managed files)

### Gate to Phase 2 (GUI)

- Wave A and Wave B command acceptance tests pass
- backward compatibility and migration upgrade tests pass
- JSON output contracts for new commands are stable
- docs under both `project4/` and `project4/lamian/docs/` are aligned

### Post-Phase 1.5 Hardening Wave (Before/Alongside Early Phase 2)

- Harden bundle import portability and structural validation.
- Reuse domain normalization rules during bundle import to keep invariants consistent with CLI ingest flows.
- Add visibility and policy controls for dropped links and figure conflicts during bundle import.
- Add stream-based bundle processing for large files.
- Add explicit reference disambiguation for numeric query/collection identifiers.
- Add vault integrity verification CLI (`verify`) and bundle preflight (`bundle inspect`, `bundle import --dry-run`).
- Complete documentation synchronization for implemented Phase 1.5 command coverage.
- Resolve critical correctness bugs from independent review (`BUG-2` tag rename corruption and `BUG-3` self-link cleanup path).
- Add schema and service hardening from independent review (link uniqueness migration, shared DB connection helper, tag validation deduplication, query/export batching).
- Evaluate and prioritize CLI expansion items (`MISS-1` to `MISS-10`) after hardening triage.

## Phase 2: Desktop GUI (Functionality First)

### Phase 2.0 (Completed)

- Added minimal GUI wrapper over shared core services.
- Delivered vault view, figure detail, metadata editor, and search.
- Verified deterministic list/detail behavior and save-failure recovery for S2 metadata mutation.

### Phase 2.1 (Completed): GUI Mutation Expansion

- GUI actions for existing mutation services were completed:
  - tag add/remove on selected figure
  - link add/remove between figures
  - figure delete with explicit confirmation and deterministic post-delete selection behavior
- GUI remained interaction-only and reused shared `tag`, `link`, and `delete` services.
- Regression coverage and full Rust gate verification were completed for mutation success/failure and deterministic list/detail refresh behavior.

### Phase 2.2 (Implemented Closure): GUI Drag-and-Drop Ingest

- Delivered drag-and-drop file intake with the same ingest core used by CLI `inject`/`import`.
- Delivered one-or-many file drop path with strict provenance prompts before commit.
- Closed with deterministic result summaries and error reporting parity with existing ingest contracts.

### Phase 2.3 (Current Planning Baseline): GUI Workflow Parity Polish

- Expose high-use operational workflows in GUI where low risk and high leverage:
  - open file, list/detail navigation polish, and search/filter ergonomics
- Keep automation-centric flows (`bundle`, `verify`, bulk import) CLI-first unless GUI value is clear.
- Close parity gaps required for MVP “GUI can drive core workflows” gate.
- Keep Rust 2021 compatibility and deterministic ordering guarantees explicit in each planning slice and acceptance check.

Phase 2.3 planning slices:

- `P4-524`: lock parity-polish UX/state-flow scope and acceptance contract in both SPEC mirrors before implementation.
- `P4-525`: implement open-file parity and deterministic selection/focus-state stabilization.
- `P4-526`: implement list/detail navigation and search/filter ergonomics while preserving deterministic ordering behavior.
- `P4-527`: add regression coverage for deterministic parity-polish behaviors.
- `P4-528`: pass full Rust gate (`cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`).

Phase 2.3 acceptance criteria:

- Scope and acceptance contract are documented in both SPEC mirrors before implementation starts.
- Open-file action behavior matches shared core semantics with deterministic state transitions.
- Navigation and search/filter polish paths preserve deterministic ordering for list rows, selection transitions, and refresh results.
- Implementation remains Rust 2021-compatible across GUI and tests.
- Full gate passes after implementation slices complete.

### Gate to Phase 3

- GUI can perform all MVP core workflows
- Error handling and data integrity checks pass

## Phase 3: Post-MVP Automation

- Evaluate screenshot mode and source-aware ingestion helpers
- Evaluate arXiv/publication modes after policy and feasibility review
- Add plugin-friendly or agent-friendly extension interfaces

## 5. Branching and Delivery Policy

- Never implement features directly on `main`
- Use dedicated feature branches for each vertical slice
- Merge only after verification checklist is complete

## 6. Verification Strategy

- Planning verification:
  - requirement traceability from `SPEC.md` to `TODO.md`
  - consistency review across all planning docs
- Implementation verification:
  - unit tests for domain and persistence
  - integration tests for CLI workflows
  - migration upgrade compatibility tests
  - GUI smoke tests once Phase 2 starts

## 7. Deliverables for Current Phase (Phase 2.3 Planning Baseline)

- Updated planning docs for the current GUI stage:
  - `project4/PLAN.md`
  - `project4/TODO.md`
- Mirrored standalone planning trackers for incubator parity:
  - `project4/lamian/docs/PLAN.md`
  - `project4/lamian/docs/TODO.md`
- Phase 2.3 planning-baseline tracker updates with mirrored acceptance criteria and slice IDs.
- Phase 2.3 implementation branches and gate evidence after each slice.

## 8. Next Execution Step

1. Phase 2.2 implementation is closed after passing P4-523 full gate (`cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`).
2. Complete Phase 2.3 planning and acceptance lock (`P4-524`) across both tracker mirrors before implementation.
3. Keep Rust 2021 compatibility and deterministic ordering guarantees explicit in every Phase 2.3 implementation and test slice (`P4-525`..`P4-528`).

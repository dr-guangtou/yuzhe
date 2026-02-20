# LaMian Plan

## 1. Objective

Build LaMian as a local-only, metadata-rich visual knowledge base for research figures, with a CLI-first core and a lightweight desktop GUI.

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
- CLI command surface for all core operations
- Basic GUI that can drive core operations

### Out of Scope for MVP

- Automatic arXiv figure extraction
- Automatic publisher scraping workflow
- Built-in AI agent features
- Cloud sync and collaboration

## 4. Implementation Phases

## Phase 0: Documentation and Governance (current)

- Create and validate planning documents in `project4/`
- Lock architecture, interfaces, and acceptance criteria
- Initialize project-specific agent instructions

### Gate to Phase 1

- `PLAN.md`, `SPEC.md`, `TODO.md` are complete and internally consistent
- Risk register and decisions are documented
- Open questions for MVP are resolved

## Phase 1: CLI Core and Data Model

- Initialize Rust workspace under `project4/lamian/`
- Implement schema migrations and repository layer
- Implement CLI commands for init/inject/tag/link/search/export
- Add tests for data integrity and command behavior

### Gate to Phase 2

- CLI acceptance tests pass
- Migration and rollback flow verified
- Data model is stable for GUI integration

## Phase 2: Desktop GUI (Functionality First)

- Add minimal GUI wrapper over core services
- Implement vault view, figure detail, metadata editor, and search
- Expose CLI-equivalent operations through GUI actions

### Gate to Phase 3

- GUI can perform all MVP core workflows
- Error handling and data integrity checks pass

## Phase 3: Post-MVP Automation

- Evaluate screenshot mode and source-aware ingestion helpers
- Evaluate arXiv/publication modes after policy and feasibility review
- Add plugin-friendly or agent-friendly extension interfaces

## 5. Branching and Delivery Policy

- Never implement features directly on `main`
- Use dedicated feature branches (first branch: `feature/project4-lamian-docs`)
- Merge only after verification checklist is complete

## 6. Verification Strategy

- Document verification in planning phase:
  - requirement traceability from `SPEC.md` to `TODO.md`
  - consistency review across all planning docs
- Implementation verification in coding phase:
  - unit tests for domain and persistence
  - integration tests for CLI workflows
  - smoke tests for GUI workflows

## 7. Deliverables for Current Phase

- `README.md`
- `PLAN.md`
- `SPEC.md`
- `TODO.md`
- `DECISIONS.md`
- `RISK_REGISTER.md`
- `AGENTS.md`
- `CLAUDE.md`
- `journal/2026-02-19.md`


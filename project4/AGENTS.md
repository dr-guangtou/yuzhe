# AGENTS.md for Project 4 (LaMian)

## Scope

This file applies to all work under `project4/`.

## Directory Responsibilities

- Planning and management documents stay in `project4/`.
- Implementation code must be developed in `project4/lamian/`.
- Do not edit files in `roadmap/`.

## Branch Safety

- Never implement directly on `main` or `master`.
- Create a feature branch before any implementation or substantial document update.

## Development Priorities

- Functionality before appearance.
- Keep changes minimal and elegant.
- Prefer root-cause fixes over temporary workarounds.

## Architecture Defaults

- Language: Rust
- Core mode: CLI-first, GUI built on the same core services
- Storage: SQLite canonical metadata store with sidecar export support
- Provenance: required on ingest

## Documentation Requirements

- Keep these files current: `PLAN.md`, `SPEC.md`, `TODO.md`, `DECISIONS.md`, `RISK_REGISTER.md`.
- Keep detailed work logs in `project4/journal/` using dated Markdown files.
- If scope/constraints change, update affected docs in the same change set.

## Verification Standard

- Do not mark tasks complete without verification evidence.
- For code tasks, include test execution results and known limitations.
- For planning tasks, include cross-file consistency checks.


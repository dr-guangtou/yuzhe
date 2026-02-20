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

## Context-Window Discipline

- Plan implementation steps in small slices that fit one context window.
- Prefer changes that complete 1-2 TODO items per session rather than broad partial edits.
- Before ending a session, update `project4/TODO.md` and add a journal entry in `project4/journal/`.
- If context is getting tight, create handover artifacts:
  - `project4/journal/handover-YYYY-MM-DD-HHMM.md`
  - `project4/journal/next-session-prompt-YYYY-MM-DD-HHMM.md`

## Handover Minimum Content

- Current branch, latest commit hash, and uncommitted status.
- Completed work, in-progress work, and blockers.
- Files changed and why they matter.
- Verification run/not run and exact commands.
- First 1-3 actions for next session with concrete commands.

## Verification Standard

- Do not mark tasks complete without verification evidence.
- For code tasks, include test execution results and known limitations.
- For planning tasks, include cross-file consistency checks.

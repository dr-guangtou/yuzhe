# AGENTS.md for LaMian

## Scope

This file applies to the standalone LaMian repository root.

## Development Priorities

- Functionality before appearance.
- Keep CLI and data integrity as first-class requirements.
- Keep changes minimal and easy to review.

## Architecture Defaults

- Language: Rust
- Storage: SQLite canonical store with sidecar export support
- Ingest policy: strict provenance required

## Verification Standard

- Run formatting, lint, and tests before completion:
  - `cargo fmt --all`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

## Documentation Requirements

- Keep `docs/SPEC.md`, `docs/TODO.md`, and `docs/DECISIONS.md` aligned with code.
- Record major changes in a dated journal entry when a journal folder is present.

## Context-Window Discipline

- Implement in small, testable slices sized for one context window.
- Prefer completing one vertical behavior (validate + persist + test) before starting the next.
- Before session end, update docs and write handover files when context is tight.
- Use `docs/journal/` in standalone mode; while incubating in `project4`, use `project4/journal/`.

## Handover Minimum Content

- Branch name, latest commit, and working tree status.
- What was completed, what is partial, and what is blocked.
- Exact verification commands and outcomes.
- First concrete commands for the next session.

# CLAUDE.md for LaMian

## Project Context

LaMian is a local-first visual knowledge base for research figures and screenshots.

## Technical Baseline

- Rust CLI core
- SQLite schema with migrations
- Strict source provenance on ingest

## Core Commands

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --help
```

## Documentation

- `docs/SPEC.md`
- `docs/TODO.md`
- `docs/DECISIONS.md`
- `docs/MIGRATION.md`

## Collaboration Preference (User-Learning Mode)

- The user is learning Rust through this project.
- Explain Rust-specific terminology, design choices, and tradeoffs in a patient and gentle style.
- When possible, introduce one core Rust concept at a time with concrete examples from this codebase.

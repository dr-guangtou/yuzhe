# LaMian CLI Core

This repository contains the Rust implementation for LaMian.

## Current Status

- Core commands are implemented: `init`, `inject`, `update`, `source update`, `tag`, `link`, `search`, `list|ls`, `show|info`, `delete`, `export`.
- Global `--json` output mode is available for Phase 1 command families: `inject`, `update`, `tag`, `link`, `search`, and `export`.
- Pre-GUI automation commands are implemented: `query`, `import`, `doctor`, `collection` (including `collection update`), `bundle`, `verify`.
- Current schema includes migrations through v5 (including link uniqueness hardening).
- Bundle hardening controls are implemented: `bundle inspect`, `bundle import --dry-run`, `bundle import --fail-on-link-loss`, and `bundle import --on-conflict skip|error|replace`.

## Quick Start

```bash
source "$HOME/.cargo/env"
cd <lamian-repo-root>
cargo run -- --vault "$PWD/.demo_vault" init
```

## Development Checks

```bash
source "$HOME/.cargo/env"
cd <lamian-repo-root>
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## Documentation

- `USAGE.md`: algorithm + CLI usage guide for implemented commands
- `docs/README.md`: doc index
- `docs/SPEC.md`: implementation specification snapshot
- `docs/TODO.md`: implementation checklist
- `docs/DECISIONS.md`: architecture decisions
- `docs/MIGRATION.md`: extraction and repository split checklist

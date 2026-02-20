# LaMian CLI Core (Bootstrap)

This repository contains the Rust implementation for LaMian.

## Current Status

- `init` command is implemented.
- Database schema initialization and migrations are implemented.
- Other CLI commands are scaffolded and return explicit "not implemented yet" errors.

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

- `docs/README.md`: doc index
- `docs/SPEC.md`: implementation specification snapshot
- `docs/TODO.md`: implementation checklist
- `docs/DECISIONS.md`: architecture decisions
- `docs/MIGRATION.md`: extraction and repository split checklist

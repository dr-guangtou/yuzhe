# LaMian CLI Core (Bootstrap)

This repository contains the Rust implementation for LaMian.

## Current Status

- `init` command is implemented.
- Database schema initialization and migrations are implemented.
- `inject`, `tag` (`add`/`remove`/`rename`), `link` (`add`/`remove`), and `search` are implemented.
- `update` and `export` remain pending.

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

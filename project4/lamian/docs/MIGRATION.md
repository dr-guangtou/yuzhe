# Repository Split Checklist

This checklist prepares `project4/lamian/` for extraction into a standalone repository.

## 1. Files That Should Move

- `Cargo.toml`
- `Cargo.lock`
- `src/`
- `.gitignore`
- `rust-toolchain.toml`
- `README.md`
- `LICENSE`
- `AGENTS.md`
- `CLAUDE.md`
- `docs/`

## 2. Files That Should Not Move

- Parent incubator planning files in `project4/` unless intentionally copied.
- Runtime build artifacts in `target/`.

## 3. Post-Split Repository Tasks

1. Initialize new git repository at extracted root.
2. Set default branch and branch protection.
3. Configure CI for:
   - `cargo fmt --all --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`
4. Publish issue templates and contribution guidelines if needed.
5. Update README badges and links.

## 4. Validation Before Split

1. Ensure no parent-path assumptions remain in docs.
2. Ensure license file and `Cargo.toml` license value agree.
3. Ensure docs in `docs/` are sufficient for independent onboarding.

## 5. Current Status

- Local structure is extraction-ready for a basic standalone Rust CLI repository.
- Phase 1 core commands are complete (`init`, `inject`, `update`, `tag`, `link`, `search`, `export`).
- Remaining implementation work before GUI is Phase 1.5 (`query`, `import`, `doctor`, `collection`, `bundle`) plus CI setup in the standalone host repository.

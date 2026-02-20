# CLAUDE.md for Project 4 (LaMian)

## Project Context

LaMian is Project 4: a local, metadata-rich visual knowledge base for research figures and screenshots.

- Planning home: `project4/`
- Implementation home: `project4/lamian/`

## Current Phase

Planning and specification lock-in.  
Primary references for this phase:

- `project4/PLAN.md`
- `project4/SPEC.md`
- `project4/TODO.md`
- `project4/DECISIONS.md`
- `project4/RISK_REGISTER.md`

## Technical Direction

- Language: Rust
- Product priority: functional CLI core before GUI polish
- Storage baseline: SQLite canonical DB + sidecar export
- Ingest policy: strict provenance (source type + key required)

## Recommended Command Patterns (implementation phase)

```bash
# Rust toolchain checks
rustc --version
cargo --version

# Build/test workflow (once Rust workspace exists)
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --bin lamian -- --help
```

## Workflow Expectations

- Keep roadmap files read-only.
- Keep planning docs synchronized with implementation changes.
- Keep dated journal entries under `project4/journal/`.
- Record architecture changes in `project4/DECISIONS.md`.
- Keep implementation scope context-window aware: prefer one small, verifiable step per session.
- When context gets tight, generate handover files in `project4/journal/` with status and next-session prompt.

## Collaboration Preference (User-Learning Mode)

- The user is new to Rust and wants to learn through development.
- Provide patient, gentle explanations for Rust-specific terms and design choices.
- Prefer clear conceptual explanations before deeper technical detail when introducing new Rust patterns.

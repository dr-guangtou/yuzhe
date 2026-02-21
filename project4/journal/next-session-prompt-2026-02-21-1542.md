Resume Phase 1.8 Wave 2 on branch `feat/lamian-phase-1-8-wave-2-caption-clear`.

Goal:
Complete `P4-418` / `L-188` by extracting a shared tag normalization/validation module and reusing it in `tag`, `search`, and `query` paths.

Current status:
- `P4-416` (`--clear-caption`) and `P4-417` (link uniqueness migration v5 + dedupe) are implemented and verified.
- TODOs already updated: `project4/TODO.md` and `project4/lamian/docs/TODO.md`.

First actions:
1. Add shared tag validation helper (new module) with current error semantics.
2. Refactor `project4/lamian/src/tag.rs`, `project4/lamian/src/search.rs`, `project4/lamian/src/query.rs` to consume it.
3. Run full gate: `cd project4/lamian && cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`.

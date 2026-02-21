Resume Phase 1.8 implementation for LaMian from `main`.

Goal:
Implement Wave 1 first: BUG-2 (tag rename corruption), BUG-3 (self-link cleanup), and edition downgrade to 2021.

Current status:
Plan is decision-complete; backlog already recorded in:
- project4/TODO.md (P4-401..P4-430)
- project4/lamian/docs/TODO.md (L-171..L-200)

First actions:
1. Create branch `feat/lamian-phase-1-8-wave-1`.
2. Patch `project4/lamian/src/tag.rs`, `project4/lamian/src/link.rs`, and `project4/lamian/Cargo.toml`.
3. Update `project4/lamian/docs/SPEC.md` with finalized Phase 1.8 decisions.

Verification:
cd project4/lamian && cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test

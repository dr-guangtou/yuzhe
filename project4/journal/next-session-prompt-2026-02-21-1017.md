Continue Phase 1.5 Wave B on branch `feat/lamian-phase-1-5-wave-b`.

Current status:
- Wave B Slice 1 (migration v4: `collections` + `collection_items`) is complete.
- Wave B Slice 2 (`collection create|add|remove|list|delete` + tests) is complete.
- Full gate is green.

Key files:
- `project4/lamian/src/db.rs`
- `project4/lamian/src/collection.rs`
- `project4/lamian/tests/cli_collection.rs`

Next actions:
1. Implement `bundle export|import` core in a small slice (deterministic manifest/checksum + skip-existing conflict policy).
2. Wire bundle CLI commands and JSON output.
3. Add bundle integration tests (roundtrip/conflict/corruption) and run:
   `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`

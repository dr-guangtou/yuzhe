Goal: continue LaMian core development by implementing `tag` command behavior in `project4/lamian`.

Current status:
- Shared `inject` core is complete and merged to `main`.
- `inject` has strict provenance validation, typed errors, and transaction-backed persistence.
- CLI integration tests exist for real fixtures and wrong-format failure.
- `tag`, `update`, `link`, `search`, and `export` are still stubs.

Key files:
- `project4/lamian/src/commands.rs`
- `project4/lamian/src/db.rs`
- `project4/lamian/src/error.rs`
- `project4/lamian/docs/TODO.md`

First actions:
1. Implement `tag add` with normalized tag rules and idempotent persistence.
2. Add tests for `inject -> tag add` success and duplicate/invalid cases.
3. Re-run verification and update TODO statuses.

Verification command:
- `cd project4/lamian && cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`

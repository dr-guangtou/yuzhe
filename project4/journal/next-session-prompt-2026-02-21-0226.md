Continue Phase 1.5 on branch `feat/lamian-phase-1-5`.

Current status:
- Phase 1.5 docs are synchronized across `project4/` and `project4/lamian/docs/`.
- Wave A query slice is complete (`saved_queries` migration + query command + tests).
- Full gate is green.

Next actions:
1. Implement `import` core in small slice (strict provenance template, continue-on-error summary, duplicate skip/report).
2. Wire `Command::Import` in CLI/commands with JSON output.
3. Add `tests/cli_import.rs` coverage and run `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`.

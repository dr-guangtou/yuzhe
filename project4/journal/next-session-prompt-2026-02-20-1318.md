Goal: implement Step 1 of LaMian core development by finishing the shared `inject` path in `project4/lamian`.

Current status:
- `init` and SQLite migrations are working.
- `inject` command exists in CLI definitions but is still a stub.
- Shared ingest-core rule is already documented (CLI and future GUI drop must use same core).

Key files:
- `project4/TODO.md`
- `project4/SPEC.md`
- `project4/lamian/src/{cli.rs,commands.rs,db.rs,error.rs}`

First actions:
1. Implement `inject` core service with strict provenance validation and typed errors.
2. Persist `figures` + `sources` in one transaction and return a stable `figure_id`.
3. Add tests for success and failure cases.

Verification commands:
- `cd project4/lamian && cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`

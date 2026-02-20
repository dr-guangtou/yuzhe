Goal: continue LaMian core algorithm development by implementing `search` in `project4/lamian`.

Current status:
- `init`, `inject`, full `tag`, and `link add/remove` are implemented and tested.
- Remaining command gaps are `update`, `search`, and `export`.
- Integration tests already cover inject/tag/link success and failure paths.

Key files:
- `project4/lamian/src/commands.rs`
- `project4/lamian/src/error.rs`
- `project4/lamian/src/{inject.rs,tag.rs,link.rs}`
- `project4/lamian/docs/TODO.md`

First actions:
1. Implement shared `search` core service with filters (`--tag`, `--source-key`, `--text`) and typed validation.
2. Wire `Command::Search` to core service and define stable CLI result output.
3. Add integration tests for tag/source/text search and empty-result behavior.

Verification command:
- `cd project4/lamian && cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`

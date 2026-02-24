# Repo TODO

## Active

- Track post-CLI-expansion follow-up items after completing `P4-430`. See `project4/TODO.md` for detailed historical tracking and future backlog additions.

## Review

- 2026-02-24: Phase 1.8 Wave 4 doctor file-path integrity checks completed with full gate pass.
- 2026-02-24: Bundle import portability validation (P4-401) completed with reference-path checks and CLI coverage.
- 2026-02-24: Bundle import domain validation parity (P4-402) completed with source/tag/link checks and CLI coverage.
- 2026-02-24: Bundle link-loss reporting policy (P4-403) completed with dropped-link counters and strict fail mode (`bundle import --fail-on-link-loss`).
- 2026-02-24: Streaming bundle IO (P4-404) completed by replacing managed-file buffering with streaming export/import paths.
- 2026-02-24: Bundle archive hardening (P4-405) completed with strict import preflight checks for duplicate manifest/metadata, unexpected entries, and non-regular tar members.
- 2026-02-24: Query/collection numeric reference disambiguation (P4-406) completed with explicit `--reference-mode auto|id|name` and ambiguity-focused CLI tests.
- 2026-02-24: Vault integrity verification command (P4-407) completed with read-only missing-file/hash-drift/size-drift checks and CLI integration coverage.
- 2026-02-24: Bundle preflight commands (P4-408) completed with `bundle inspect` and non-mutating `bundle import --dry-run` plus CLI integration coverage.
- 2026-02-24: Bundle conflict policy controls (P4-409) completed with `bundle import --on-conflict skip|error|replace` and CLI coverage for error/replace semantics.
- 2026-02-24: Documentation alignment pass (P4-410) completed by syncing README/USAGE/spec command coverage with implemented Phase 1.8 hardening features.
- 2026-02-24: `show`/`info` single-figure detail command (P4-421) completed with full metadata output and integration coverage.
- 2026-02-24: `list`/`ls` figure command (P4-422) completed with sortable/limited output and integration coverage.
- 2026-02-24: figure delete command (P4-423) completed with transactional dependency cleanup, orphan-tag pruning, and managed-file cleanup policy.
- 2026-02-24: figure `open` command (P4-424) completed with resolved file-path launch via OS viewer semantics and integration coverage.
- 2026-02-24: tag-prefix search capability (P4-425) completed with hierarchical `--tag-prefix` filtering and integration coverage.
- 2026-02-24: source metadata update command (P4-426) completed with `source update` field set/clear semantics and integration coverage.
- 2026-02-24: filterless saved queries (P4-427) completed with sort/order/limit-only template support and query save/run integration coverage.
- 2026-02-24: Phase 1 JSON output mode (P4-428) completed with global `--json` envelopes for inject/update/tag/link/search/export and integration coverage.
- 2026-02-24: `tag list` command (P4-429) completed with deterministic ordering, per-tag figure counts, and integration coverage.
- 2026-02-24: `collection update` command (P4-430) completed with rename/query-binding update semantics and integration coverage.

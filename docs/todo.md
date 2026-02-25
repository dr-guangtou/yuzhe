# Repo TODO

## Active

- Phase 2.2 GUI drag-and-drop ingest implementation is closed on `feat/phase-2-2-gui-drop-ingest-planning`; next step is Phase 2.3 parity-polish planning/tracker definition.

## Review

- 2026-02-24: Phase 2.0-S1 GUI foundation completed on `feat/phase-2-0-gui-foundation-egui` with `egui/eframe`, shared library extraction, read-only browse/detail UI, and a full Rust gate pass.
- 2026-02-25: P4-505 completed by locking the Phase 2.0-S2 GUI mutation UX/state-flow spec (edit lifecycle, validation mapping, and save/cancel semantics) for implementation.
- 2026-02-25: P4-506 completed by wiring GUI figure metadata editing to shared `update_figure` with draft state, save/cancel controls, and full gate verification.
- 2026-02-25: P4-507 completed by wiring GUI source metadata editing to shared `update_source_metadata` with draft state, clear-flag controls, and full gate verification.
- 2026-02-25: P4-508 completed by adding GUI regression tests for editor lifecycle transitions, save-failure recovery, and deterministic list/detail behavior after figure/source saves.
- 2026-02-25: P4-509 completed by syncing Phase 2 S2 implementation/acceptance docs between `project4/` and `project4/lamian/docs/` and recording incubator parity rules in AGENTS guidance.
- 2026-02-25: P4-510 completed with a passing full Rust gate in `project4/lamian` (`cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`).
- 2026-02-25: Phase 2 next-stage planning updated by defining Phase 2.1 (tag/link/delete mutation expansion) and Phase 2.2 (drag-and-drop ingest design) in PLAN/TODO trackers across incubator and standalone mirrors.
- 2026-02-25: P4-511 completed by locking the Phase 2.1 GUI mutation UX/state model for tag/link/delete flows, including deterministic post-delete selection behavior and shared-service validation mapping.
- 2026-02-25: P4-512 completed by wiring GUI tag add/remove actions to shared tag services with lifecycle/error handling, regression tests, and a passing full gate.
- 2026-02-25: P4-513 completed by wiring GUI link add/remove actions to shared link services with lifecycle/error handling, regression tests, and a passing full gate.
- 2026-02-25: P4-514/P4-515/P4-516 completed by implementing GUI delete confirmation flow, adding full tag/link/delete regression coverage, and passing the full Rust gate in `project4/lamian`.
- 2026-02-25: P4-518 completed by synchronizing Phase 2.1 tracker/spec wording from planning baseline to implemented closure across incubator and standalone mirrors.
- 2026-02-25: P4-517 completed by locking the Phase 2.2 GUI drag-and-drop ingest UX/state model (provenance prompt policy, deterministic multi-file ordering, shared ingest-core reuse, and Rust 2021 compatibility constraints) across SPEC/TODO mirrors.
- 2026-02-25: P4-519 completed by implementing the Phase 2.2 GUI drop-session state machine scaffold in `project4/lamian/src/gui.rs` with deterministic dropped-path ordering and lifecycle transition tests.
- 2026-02-25: P4-520 completed by wiring drop-session commits in `project4/lamian/src/gui.rs` to shared `inject_figure` ingest services with deterministic one-or-many ordering, duplicate-aware outcomes, and partial-failure reporting.
- 2026-02-25: P4-521 completed by adding batch provenance defaults and per-item override behavior to GUI drop ingest state/UI in `project4/lamian/src/gui.rs`, keeping commit blocked until required metadata is complete for all items; full-gate verification remains pending for P4-523 after P4-522.
- 2026-02-25: P4-522 completed by adding GUI regression coverage for deterministic normalized-path commit-result ordering, stable per-item success/failure reporting, and commit-failure recovery retry behavior in `project4/lamian/src/gui.rs`; full-gate verification remains pending for P4-523.
- 2026-02-25: P4-523 completed by passing the full Rust verification gate in `project4/lamian` (`cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`) after P4-519..P4-522, closing Phase 2.2 implementation.
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

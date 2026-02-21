# Architecture Decisions (LaMian)

## ADR-001: Use Rust as the Primary Language

- Date: 2026-02-19
- Status: Accepted
- Context:
  - Need CLI-first architecture with long-term cross-platform capability.
  - Need strong local performance and reliability.
- Decision:
  - Use Rust for core implementation.
- Consequences:
  - Better cross-platform portability than Swift-only path.
  - More effort for GUI polish on macOS than SwiftUI-first design.

## ADR-002: Canonical Metadata Store is SQLite

- Date: 2026-02-19
- Status: Accepted
- Context:
  - Need robust querying, indexing, transactions, and migration support.
- Decision:
  - Use SQLite as source of truth for metadata.
  - Provide sidecar export for portability and interoperability.
- Consequences:
  - Clear consistency model.
  - Need migration/versioning discipline.

## ADR-003: Strict Provenance Required at Ingest

- Date: 2026-02-19
- Status: Accepted
- Context:
  - Research workflow depends on traceability and source integrity.
- Decision:
  - Require source type and source key (`doi`/`url`/other key) when ingesting.
- Consequences:
  - Higher metadata quality.
  - Slightly slower ingest for poorly documented assets.

## ADR-004: CLI-First Core with GUI on Top

- Date: 2026-02-19
- Status: Accepted
- Context:
  - User requires agent-friendly operations and automation readiness.
- Decision:
  - Build complete CLI core first; GUI invokes same core services.
- Consequences:
  - Strong automation path and testability.
  - GUI delivery follows core stabilization.

## ADR-005: Split Project Roles by Directory

- Date: 2026-02-19
- Status: Accepted
- Context:
  - User requested parent folder for planning and sub-directory for real development.
- Decision:
  - Keep planning docs in `project4/` and implementation in `project4/lamian/`.
- Consequences:
  - Clear separation of planning vs coding artifacts.
  - Requires discipline to avoid file drift.

## ADR-006: Single Ingest Core for CLI and GUI

- Date: 2026-02-20
- Status: Accepted
- Context:
  - GUI drag-and-drop ingestion is planned.
  - Duplicate ingest implementations would create behavior drift and validation inconsistencies.
- Decision:
  - Keep one ingest core service in the domain layer.
  - CLI and GUI are frontends that call the same ingest core.
- Consequences:
  - Better consistency in provenance validation and persistence behavior.
  - GUI drag-and-drop implementation is faster once CLI ingest core is stable.

## ADR-007: Phase 1.5 Delivery Uses Two Waves

- Date: 2026-02-20
- Status: Accepted
- Context:
  - Phase 1 commands are complete, but pre-GUI operational tooling is missing.
  - Implementing all automation features at once raises integration risk.
- Decision:
  - Phase 1.5 is split into:
    - Wave A: `query`, `import`, `doctor`
    - Wave B: `collection`, `bundle`
- Consequences:
  - Earlier usable automation surface.
  - Clear gate between data operations and portability features.

## ADR-008: New Phase 1.5 Commands Use JSON-Only Output

- Date: 2026-02-20
- Status: Accepted
- Context:
  - Agent-first usage needs structured output contracts.
  - Existing command output remains human-readable and stable.
- Decision:
  - New Phase 1.5 commands emit JSON only.
  - Existing command output format is unchanged in Phase 1.5.
- Consequences:
  - Easier machine consumption for new workflows.
  - Mixed output styles across old/new commands until broader CLI harmonization.

## ADR-009: Import Requires Explicit Provenance Templates

- Date: 2026-02-20
- Status: Accepted
- Context:
  - Strict provenance is a core invariant.
  - Batch import must not weaken metadata quality.
- Decision:
  - `import` requires explicit `--source-type` and `--source-key-template`.
  - Duplicate figure IDs are skipped and reported.
  - Batch import continues on per-item failures and returns non-zero if failures exist.
- Consequences:
  - Metadata quality is preserved.
  - Import summaries become mandatory for operational visibility.

## ADR-010: Doctor Supports DB-Only Safe Auto-Fixes

- Date: 2026-02-20
- Status: Accepted
- Context:
  - Health checks are useful, but automatic file mutations are high risk.
- Decision:
  - `doctor --fix` is allowed only for DB-safe fixes in Phase 1.5.
  - No file move/delete/rewrite actions in this phase.
- Consequences:
  - Reduces accidental data loss risk.
  - Some issues remain report-only and require manual remediation.

## ADR-011: Collections Are Hybrid With Dynamic Binding to Saved Query IDs

- Date: 2026-02-20
- Status: Accepted
- Context:
  - Users need both curated static lists and query-driven smart sets.
- Decision:
  - Collections support:
    - static membership
    - dynamic mode referencing `saved_queries.query_id`
- Consequences:
  - Flexible curation workflows before GUI.
  - Requires integrity checks for query/collection binding.

## ADR-012: Bundle Format Is Tar.gz With Managed-File Default Payload

- Date: 2026-02-20
- Status: Accepted
- Context:
  - Need portable vault snapshots with deterministic packaging.
- Decision:
  - `bundle export|import` uses `tar.gz`.
  - Default payload includes metadata + managed files under `.lamian/figures`.
  - Import conflict policy defaults to skip existing figure IDs.
- Consequences:
  - Portable, deterministic handoff format.
  - External reference files remain out of scope by default.

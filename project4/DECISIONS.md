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

# LaMian Decisions (Standalone)

## D-001: Rust for Core Implementation

- Status: accepted
- Why:
  - CLI-first reliability and performance
  - Cross-platform portability for future desktop support

## D-002: SQLite as Canonical Metadata Store

- Status: accepted
- Why:
  - transactional integrity
  - migration support
  - strong query capability for tags/search

## D-003: Strict Provenance on Ingest

- Status: accepted
- Why:
  - research traceability and data quality
  - easier long-term curation and auditing

## D-004: CLI First, GUI Layer Second

- Status: accepted
- Why:
  - supports agent-driven workflows
  - improves testability and automation

## D-005: MIT License for Standalone Repository

- Status: accepted
- Why:
  - simple permissive open-source licensing
  - matches crate metadata in `Cargo.toml`

## D-006: Shared Ingest Core Across CLI and GUI

- Status: accepted
- Why:
  - avoid duplicated ingest logic
  - ensure strict provenance validation is enforced identically in all frontends
  - make drag-and-drop a UI trigger over existing core behavior

## D-007: Store Caption on `figures` Instead of `sources`

- Status: accepted
- Why:
  - caption is figure-level metadata, not provenance metadata
  - avoids ambiguity when a figure has multiple source rows
  - keeps update/search/export behavior aligned to domain intent

## D-008: Phase 1.5 Uses Two-Wave Delivery

- Status: accepted
- Why:
  - reduce integration risk by shipping query/import/doctor before collections/bundles
  - keep pre-GUI scope incremental and verifiable

## D-009: New Phase 1.5 Commands Emit JSON-Only Output

- Status: accepted
- Why:
  - improve automation and agent tooling compatibility
  - maintain stable structured contracts for batch operations

## D-010: Batch Import Keeps Strict Provenance

- Status: accepted
- Why:
  - preserve data quality invariant from `inject`
  - avoid silently introducing low-trust records in large imports

## D-011: Doctor Auto-Fix Is Limited to DB-Safe Changes

- Status: accepted
- Why:
  - reduce risk of destructive file-side effects
  - allow controlled cleanup where invariants are deterministic

## D-012: Collections Are Hybrid and Dynamic Mode Binds Saved Query IDs

- Status: accepted
- Why:
  - support both manual curation and smart-set behavior
  - keep dynamic rules centralized in saved query definitions

## D-013: Bundle Format Is Tar.gz With Managed-File Default Payload

- Status: accepted
- Why:
  - deterministic and portable archive format
  - avoids implicit capture of arbitrary external referenced files

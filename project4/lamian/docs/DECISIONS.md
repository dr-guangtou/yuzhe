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


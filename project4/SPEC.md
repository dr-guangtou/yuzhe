# LaMian Technical Specification

## 1. Product Definition

LaMian is a local-only visual knowledge base for research figures and screenshots. It helps users collect, organize, search, and connect figures through metadata, tags, notes, and links.

## 2. Target User and Platform

- Primary user: individual researcher
- Primary platform: macOS
- Architecture direction: cross-platform capable (macOS, Windows, Linux) via Rust core

## 3. Product Principles

- Functionality before appearance
- CLI-first core, GUI as an operational layer over the same core services
- Local-first and offline-friendly
- Strict provenance and metadata quality
- Minimal and elegant scope for MVP

## 4. MVP Scope

### 4.1 Functional Requirements

- FR-001 Vault initialization
  - Create or open a vault directory
  - Initialize database and metadata configuration

- FR-002 Figure ingest (`inject`)
  - Register an existing local image file into vault metadata
  - Copy or reference mode for source file handling
  - Compute and persist file hash
  - Use a single shared ingest core service for all frontends (CLI and GUI)
  - Support both single-file ingest and multi-file ingest inputs

- FR-003 Provenance enforcement
  - Require source type and source key on ingest
  - Source types: `doi`, `url`, `local`, `manual`

- FR-004 Metadata management
  - Basic metadata: name, media type, size, timestamps
  - User metadata: caption, Markdown note, custom fields

- FR-005 Hierarchical tags
  - Tag format supports hierarchy by colon delimiter (example: `galaxy:elliptical`)
  - One figure can hold multiple tags

- FR-006 Search and filtering
  - Query by tags, source fields, caption/note text, and timestamps

- FR-007 Figure linking
  - Stable `figure_id` for each item
  - Link syntax baseline in notes: `[[figure_id]]`
  - Store normalized links in dedicated table

- FR-008 Export
  - Export metadata to sidecar files (YAML or JSON) for portability

- FR-009 GUI baseline
  - Vault browser (list/grid)
  - Figure detail view and metadata editor
  - Search and tag filtering
  - Trigger core operations available in CLI
  - Provide drag-and-drop entry point ("Drop the Figure Here") that calls the same ingest core as CLI
  - If required provenance fields are missing, GUI must prompt for metadata before final commit

### 4.2 Non-Functional Requirements

- NFR-001 Data durability
  - Atomic writes for metadata updates
  - Migration versioning for schema changes

- NFR-002 Privacy
  - No cloud dependency in MVP
  - No required telemetry

- NFR-003 Performance
  - CLI ingest and metadata updates should complete without perceptible delay for single items under normal local IO conditions
  - Search over medium vaults should remain interactive
  - Exact thresholds need to be measured during implementation benchmarks

- NFR-004 Reliability
  - Errors return actionable messages
  - Failed operations do not corrupt vault metadata

## 5. Out of Scope for MVP

- arXiv mode and publisher automation
- Screenshot mode automation
- Built-in LLM caption generation
- Multi-device sync and collaboration

## 6. Technical Architecture

## 6.1 Language and Runtime

- Core language: Rust
- Packaging target: local desktop app + CLI binary

## 6.2 Logical Modules

- `domain_core`
  - figure, source, tag, link, note models
  - validation and invariants
  - shared ingest service used by CLI and GUI frontends
- `persistence`
  - SQLite schema, migrations, repository traits
- `cli_app`
  - command parsing and output formatting
- `gui_app`
  - desktop UI layer invoking domain services
- `exporter`
  - sidecar serialization

## 6.3 Data Store Strategy

- Canonical store: SQLite
- Portability: sidecar export files per figure
- Source of truth: database wins in conflicts

## 7. Data Model (MVP Draft)

## 7.1 `figures`

- `figure_id` (immutable text ID)
- `display_name`
- `file_path`
- `file_hash_sha256`
- `media_type`
- `file_size_bytes`
- `created_at`
- `updated_at`

## 7.2 `sources`

- `source_id`
- `figure_id`
- `source_type` (`doi`, `url`, `local`, `manual`)
- `source_key` (doi/url/path/ref key)
- `source_title` (optional)
- `source_authors` (optional)
- `source_published_at` (optional)

## 7.3 `tags`

- `tag_id`
- `tag_name` (normalized lowercase)
- `tag_parent` (nullable for hierarchy)

## 7.4 `figure_tags`

- `figure_id`
- `tag_id`

## 7.5 `links`

- `link_id`
- `from_figure_id`
- `to_figure_id`
- `relation_type` (default `related`)

## 7.6 `notes`

- `figure_id`
- `note_markdown`
- `updated_at`

## 8. CLI Interface Draft

- `lamian init --vault <path>`
- `lamian inject <file_path> --vault <path> --source-type <type> --source-key <value> [--copy-mode copy|reference]`
- `lamian update <figure_id> [--name ...] [--caption ...] [--note-file ...]`
- `lamian tag add <figure_id> <tag>`
- `lamian tag remove <figure_id> <tag>`
- `lamian tag rename <old_tag> <new_tag>`
- `lamian link add <from_figure_id> <to_figure_id> [--relation related]`
- `lamian link remove <from_figure_id> <to_figure_id>`
- `lamian search [--tag ...] [--source-key ...] [--text ...]`
- `lamian export [--format yaml|json] [--target <path>]`

## 9. Error Handling Contract (MVP)

- Validation errors:
  - missing required provenance fields
  - malformed tag format
  - unknown figure IDs
- IO errors:
  - unreadable file
  - vault path unavailable
- Data integrity errors:
  - duplicate hash collisions handled as explicit duplicates
  - transaction rollback on failed writes

## 10. Security and Compliance Baseline

- Local-only data processing by default
- User is responsible for rights and license compliance of stored images
- LaMian records provenance to improve traceability

## 11. Testing and Acceptance Criteria

### 11.1 Unit Tests

- model validation
- tag parsing and normalization
- link parser for `[[figure_id]]`

### 11.2 Integration Tests

- init -> inject -> update -> tag -> link -> search -> export workflow
- migration upgrade path

### 11.3 GUI Smoke Tests

- open vault
- inspect figure details
- edit metadata
- apply tag filters
- drag-and-drop one file and verify same ingest result as CLI inject
- drag-and-drop multiple files and verify each file is validated and persisted independently

### 11.4 Acceptance

- All FR items satisfied
- No schema corruption on interrupted operations
- CLI and GUI can operate on the same vault without inconsistencies

## 12. Open Questions

- Preferred GUI framework for Rust in final implementation phase
- Final sidecar format default (`yaml` vs `json`)
- Figure deduplication policy when same hash appears with different filenames

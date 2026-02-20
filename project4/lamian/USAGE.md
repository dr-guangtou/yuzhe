# LaMian Usage Guide

## Purpose

This guide explains:

- how the current LaMian CLI works
- the core algorithms behind `init`, `inject`, `tag`, `link`, and `search`
- how to run the implemented workflows end-to-end

## Current Command Coverage

Implemented:

- `init`
- `inject`
- `tag add`
- `tag remove`
- `tag rename`
- `link add`
- `link remove`
- `search`

Not implemented yet:

- `update`
- `export`

## Core Algorithms

## 1. Vault Initialization (`init`)

`lamian init --vault <path>`:

1. Creates the vault root and `.lamian/` directory.
2. Opens SQLite DB at `.lamian/lamian.db`.
3. Applies migrations transactionally (`schema_migrations` + core tables).
4. Creates `.lamian/vault.toml` if missing.

Properties:

- idempotent
- migration-safe
- leaves existing initialized vault intact

## 2. Figure Ingest (`inject`)

`lamian inject <file_path> --source-type <type> --source-key <value> [--copy-mode copy|reference] --vault <path>`

Pipeline:

1. Validate vault path and DB initialization state.
2. Validate provenance:
   - `source_key` required for all source types
   - DOI must start with `10.` and contain `/`
   - URL must start with `http://` or `https://`
3. Validate input file path (exists + regular file).
4. Detect supported image media type from extension.
5. Compute file SHA-256.
6. Build stable `figure_id` from:
   - file hash
   - source type
   - source key
7. Handle file storage mode:
   - `reference`: keep original path
   - `copy`: copy into `.lamian/figures/`
8. Persist `figures` + `sources` in one transaction.

Properties:

- deterministic figure ID for same hash + provenance tuple
- transactional metadata writes
- typed, actionable errors on validation/IO/DB failures

## 3. Tag Management (`tag`)

## 3.1 `tag add`

`lamian tag add <figure_id> <tag> --vault <path>`

Rules:

- normalize tag to lowercase
- hierarchy delimiter is `:`
- reject empty segments (`jwst::ml`, `:jwst`, `jwst:`)
- allowed characters per segment: `a-z`, `0-9`, `_`, `-`

Persistence:

1. Ensure figure exists.
2. `INSERT OR IGNORE` into `tags`.
3. `INSERT OR IGNORE` into `figure_tags`.

Property:

- idempotent assignment (second add does not duplicate mapping)

## 3.2 `tag remove`

`lamian tag remove <figure_id> <tag> --vault <path>`

Behavior:

- requires figure to exist
- removes mapping from `figure_tags`
- if tag has no remaining figure assignments, removes orphan tag row

Failures:

- tag does not exist
- tag exists but is not assigned to the target figure

## 3.3 `tag rename`

`lamian tag rename <old_tag> <new_tag> --vault <path>`

Behavior:

- normalizes old/new to lowercase
- renames root tag and descendants by prefix rewrite
  - example: `jwst` -> `observatory` also renames:
    - `jwst:machine_learning` -> `observatory:machine_learning`
- updates `tag_parent` values accordingly
- rejects rename when target tag or target descendant already exists

## 4. Link Management (`link`)

Links are directed: `A -> B` is different from `B -> A`.

## 4.1 `link add`

`lamian link add <from_figure_id> <to_figure_id> [--relation <value>] --vault <path>`

Rules:

- both figure IDs must exist
- self-link is rejected (`from == to`)
- relation is normalized to lowercase
- allowed relation chars: `a-z`, `0-9`, `_`, `-`, `:`

Property:

- idempotent for exact triple `(from_figure_id, to_figure_id, relation_type)`

## 4.2 `link remove`

`lamian link remove <from_figure_id> <to_figure_id> --vault <path>`

Behavior:

- removes all relations for that directed pair (not filtered by relation type)
- fails if no links exist for that pair

## 5. Search (`search`)

`lamian search [--tag <tag>] [--source-key <source_key>] [--text <text>] --vault <path>`

Behavior:

- accepts independent optional filters and combines them with logical `AND`
- `--tag` is normalized to lowercase and uses the same hierarchy validation rules as tag commands
- `--source-key` matches case-insensitively against source records
- `--text` performs case-insensitive substring matching against figure display name, source fields, notes, and tag names
- output is deterministic: first line prints count, followed by rows sorted by `figure_id`
- when no rows match, prints `Search results: 0` and `No figures matched.`

## Practical CLI Usage

## 1. Initialize Vault

```bash
cd project4/lamian
cargo run -- --vault "$PWD/.demo_vault" init
```

## 2. Inject a Figure

Reference mode:

```bash
cargo run -- --vault "$PWD/.demo_vault" inject \
  "$PWD/tests/fixtures/2602.17205_1.png" \
  --source-type doi \
  --source-key "10.1126/science.ady9404" \
  --copy-mode reference
```

Copy mode:

```bash
cargo run -- --vault "$PWD/.demo_vault" inject \
  "$PWD/tests/fixtures/500px-Elliptical_galaxy_IC_2006.jpg" \
  --source-type url \
  --source-key "https://en.wikipedia.org/wiki/Elliptical_galaxy" \
  --copy-mode copy
```

## 3. Tag Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" tag add <figure_id> "jwst:machine_learning"
cargo run -- --vault "$PWD/.demo_vault" tag remove <figure_id> "jwst:machine_learning"
cargo run -- --vault "$PWD/.demo_vault" tag rename "jwst" "observatory"
```

## 4. Link Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" link add <from_figure_id> <to_figure_id> --relation related
cargo run -- --vault "$PWD/.demo_vault" link remove <from_figure_id> <to_figure_id>
```

## 5. Search Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" search --tag "observatory:jwst"
cargo run -- --vault "$PWD/.demo_vault" search --source-key "10.1126/science.ady9404"
cargo run -- --vault "$PWD/.demo_vault" search --text "elliptical_galaxy"
```

## Verification

Run the full development gate:

```bash
cd project4/lamian
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

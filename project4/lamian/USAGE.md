# LaMian Usage Guide

## Purpose

This guide explains:

- how the current LaMian CLI works
- the core algorithms behind `init`, `inject`, `update`, `source update`, `tag`, `link`, `search`, `list`, `show`, `delete`, `export`, `query`, `import`, `doctor`, `collection`, `bundle`, and `verify`
- how to run the implemented workflows end-to-end

## Current Command Coverage

Implemented:

- `init`
- `inject`
- `update`
- `source update`
- `tag add`
- `tag remove`
- `tag rename`
- `tag list`
- `link add`
- `link remove`
- `search`
- `list` (alias: `ls`)
- `show` (alias: `info`)
- `delete`
- `export`
- `query save|run|list|delete`
- `import`
- `doctor`
- `collection create|add|remove|list|delete|update`
- `bundle export|inspect|import`
- `verify`

Global output mode:

- `--json` (global flag) is implemented for `inject`, `update`, `tag`, `link`, `search`, and `export`.
- Without `--json`, these commands keep their existing human-readable output.

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

## 3. Metadata Update (`update`)

`lamian update <figure_id> [--name <value>] [--caption <value>] [--clear-caption] [--note-file <path>] --vault <path>`

Behavior:

- requires `figure_id` to exist
- requires at least one update payload flag: `--name`, `--caption`, `--clear-caption`, or `--note-file`
- `--name` updates `figures.display_name`
- `--caption` updates `figures.caption`
- `--clear-caption` sets `figures.caption` to `NULL`
- `--caption` and `--clear-caption` cannot be used together
- `--note-file` reads UTF-8 markdown text and upserts into `notes.note_markdown`
- all selected updates are committed transactionally

Failures:

- unknown figure ID
- missing payload (no update flags)
- missing/non-file note path
- non-UTF-8 note file content

## 4. Tag Management (`tag`)

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

## 3.4 `tag list`

`lamian tag list --vault <path>`

Behavior:

- lists all known tags from the vault without running full metadata export
- deterministic order by `tag_name` ascending
- reports per-tag `figure_count`

## 5. Link Management (`link`)

Links are directed: `A -> B` is different from `B -> A`.

## 5.1 `link add`

`lamian link add <from_figure_id> <to_figure_id> [--relation <value>] --vault <path>`

Rules:

- both figure IDs must exist
- self-link is rejected (`from == to`)
- relation is normalized to lowercase
- allowed relation chars: `a-z`, `0-9`, `_`, `-`, `:`

Property:

- idempotent for exact triple `(from_figure_id, to_figure_id, relation_type)`

## 5.2 `link remove`

`lamian link remove <from_figure_id> <to_figure_id> --vault <path>`

Behavior:

- removes all relations for that directed pair (not filtered by relation type)
- fails if no links exist for that pair

## 6. Search (`search`)

`lamian search [--tag <tag>] [--tag-prefix <tag_prefix>] [--source-key <source_key>] [--text <text>] --vault <path>`

Behavior:

- accepts independent optional filters and combines them with logical `AND`
- `--tag` is normalized to lowercase and uses the same hierarchy validation rules as tag commands
- `--tag-prefix` is normalized like tags and matches descendants only (`prefix:*` semantics, for example `observatory` matches `observatory:jwst` but not `observ`)
- `--source-key` matches case-insensitively against source records
- `--text` performs case-insensitive substring matching against figure display name, figure caption, source fields, notes, and tag names
- output is deterministic: first line prints count, followed by rows sorted by `figure_id`
- when no rows match, prints `Search results: 0` and `No figures matched.`

## 7. Figure List (`list`/`ls`)

`lamian list [--sort figure-id|display-name|created-at|updated-at] [--order asc|desc] [--limit <n>] --vault <path>`
`lamian ls [--sort figure-id|display-name|created-at|updated-at] [--order asc|desc] [--limit <n>] --vault <path>`

Behavior:

- returns all figures in human-readable rows with key timestamps
- default sort is `figure-id` ascending
- supports explicit sort field and order
- supports optional positive `--limit` to truncate output
- prints `No figures found.` when the vault has no figures

## 8. Figure Detail (`show`/`info`)

`lamian show <figure_id> --vault <path>`
`lamian info <figure_id> --vault <path>`

Behavior:

- resolves one figure by `figure_id`
- prints full metadata in deterministic sections:
  - figure fields (`display_name`, caption, path/hash/media/size, timestamps)
  - source records
  - assigned tags
  - outbound links
  - optional note metadata/content
- fails with `unknown figure id` when the target figure does not exist

## 9. Figure Delete (`delete`)

`lamian delete <figure_id> --vault <path>`

Behavior:

- removes one figure by `figure_id` in a transaction
- relies on foreign-key cascade cleanup for dependent rows (`sources`, `notes`, links, collection items, figure-tag relations)
- prunes orphan tags after deletion when they have no remaining figure assignment and no child tag references
- attempts managed-file cleanup only when the figure file is under `.lamian/figures`; reference files outside vault storage are not removed

## 9.1 Source Metadata Update (`source update`)

`lamian source update <figure_id> [--title <value>] [--authors <value>] [--published-at <value>] [--clear-title] [--clear-authors] [--clear-published-at] --vault <path>`

Behavior:

- requires `figure_id` to exist and have at least one source row
- updates source metadata fields (`source_title`, `source_authors`, `source_published_at`) for the figure
- supports explicit clearing per field via `--clear-*`
- rejects conflicting set+clear for the same field in one request
- requires at least one metadata update flag

## 10. Export (`export`)

`lamian export [--format yaml|json] [--target <path>] --vault <path>`

Behavior:

- exports full vault metadata snapshot from SQLite canonical store
- output includes `schema_version` and ordered `figures` with sources/tags/links/note
- deterministic ordering for stable diffs and automation
- `--format json|yaml` controls serializer (`yaml` is default)
- without `--target`, payload is printed to stdout only
- with `--target`, writes file and prints a concise status line
- creates parent directories for `--target` automatically
- rejects `--target` when it points to an existing directory

## 11. Saved Queries (`query`)

`lamian query save|run|list|delete ... --vault <path>`

Behavior:

- `save` persists normalized filters (`--tag`, `--source-key`, `--text`) plus sort/order/limit
- `run` resolves references via `--reference-mode auto|id|name` and supports `--detail ids|full`
- `list` returns all saved queries ordered by query name
- `delete` removes query by reference with `--reference-mode auto|id|name`
- output for all `query` subcommands is JSON-only

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

## 3. Update Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" update <figure_id> --name "JWST Panel 1"
cargo run -- --vault "$PWD/.demo_vault" update <figure_id> --caption "NIRCam composite of target field"
cargo run -- --vault "$PWD/.demo_vault" update <figure_id> --clear-caption
cargo run -- --vault "$PWD/.demo_vault" update <figure_id> --note-file "$PWD/example_note.md"
```

## 4. Tag Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" tag add <figure_id> "jwst:machine_learning"
cargo run -- --vault "$PWD/.demo_vault" tag remove <figure_id> "jwst:machine_learning"
cargo run -- --vault "$PWD/.demo_vault" tag rename "jwst" "observatory"
```

## 5. Link Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" link add <from_figure_id> <to_figure_id> --relation related
cargo run -- --vault "$PWD/.demo_vault" link remove <from_figure_id> <to_figure_id>
```

## 6. Search Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" search --tag "observatory:jwst"
cargo run -- --vault "$PWD/.demo_vault" search --source-key "10.1126/science.ady9404"
cargo run -- --vault "$PWD/.demo_vault" search --text "elliptical_galaxy"
```

## 7. List Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" list
cargo run -- --vault "$PWD/.demo_vault" ls --sort display-name --order desc --limit 20
```

## 8. Show Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" show <figure_id>
cargo run -- --vault "$PWD/.demo_vault" info <figure_id>
```

## 9. Delete Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" delete <figure_id>
```

Source metadata update:

```bash
cargo run -- --vault "$PWD/.demo_vault" source update <figure_id> \
  --title "JWST Deep Field" \
  --authors "A. Researcher; B. Scientist" \
  --published-at "2026-02-24"
```

## 10. Export Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" export --format json
cargo run -- --vault "$PWD/.demo_vault" export --format yaml --target "$PWD/.demo_vault/.lamian/export.yaml"
```

## 11. Query Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" query save "jwst-only" --tag "observatory:jwst" --sort updated-at --order desc --limit 5
cargo run -- --vault "$PWD/.demo_vault" query run "jwst-only" --detail ids
cargo run -- --vault "$PWD/.demo_vault" query run "123" --reference-mode name --detail full
cargo run -- --vault "$PWD/.demo_vault" query list
cargo run -- --vault "$PWD/.demo_vault" query delete "jwst-only" --reference-mode auto
```

## 12. Import Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" import "$PWD/import_batch" --source-type local --source-key-template "batch:{relative_path}" --copy-mode reference
cargo run -- --vault "$PWD/.demo_vault" import "$PWD/import_batch" --source-type local --source-key-template "batch:{relative_path}" --recursive --dry-run
```

Supported import template placeholders:

- `{file_name}`
- `{file_stem}`
- `{extension}`
- `{relative_path}`

## 13. Doctor Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" doctor
cargo run -- --vault "$PWD/.demo_vault" doctor --fix
```

## 14. Collection Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" collection create "my-static"
cargo run -- --vault "$PWD/.demo_vault" collection add "my-static" <figure_id> --reference-mode auto
cargo run -- --vault "$PWD/.demo_vault" collection remove "my-static" <figure_id> --reference-mode auto
cargo run -- --vault "$PWD/.demo_vault" collection list
cargo run -- --vault "$PWD/.demo_vault" collection list --collection "my-static" --reference-mode auto
cargo run -- --vault "$PWD/.demo_vault" collection create "jwst-dynamic" --query-id 1
cargo run -- --vault "$PWD/.demo_vault" collection delete "my-static" --reference-mode auto
cargo run -- --vault "$PWD/.demo_vault" collection update "my-static" --name "my-renamed"
cargo run -- --vault "$PWD/.demo_vault" collection update "my-renamed" --query-id 1
cargo run -- --vault "$PWD/.demo_vault" collection update "my-renamed" --clear-query-id
```

## 15. Bundle Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" bundle export --target "$PWD/snapshot.tar.gz"
cargo run -- bundle inspect "$PWD/snapshot.tar.gz"
cargo run -- --vault "$PWD/.demo_vault" bundle import "$PWD/snapshot.tar.gz"
cargo run -- --vault "$PWD/.demo_vault" bundle import "$PWD/snapshot.tar.gz" --dry-run
cargo run -- --vault "$PWD/.demo_vault" bundle import "$PWD/snapshot.tar.gz" --fail-on-link-loss
cargo run -- --vault "$PWD/.demo_vault" bundle import "$PWD/snapshot.tar.gz" --on-conflict skip
cargo run -- --vault "$PWD/.demo_vault" bundle import "$PWD/snapshot.tar.gz" --on-conflict error
cargo run -- --vault "$PWD/.demo_vault" bundle import "$PWD/snapshot.tar.gz" --on-conflict replace
```

## 16. Verify Operations

```bash
cargo run -- --vault "$PWD/.demo_vault" verify
```

## Verification

Run the full development gate:

```bash
cd project4/lamian
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## Output Contract Summary

- Existing Phase 1 commands keep human-readable output.
- `query`, `import`, `doctor`, `collection`, `bundle`, and `verify` use JSON output.
- `verify` exits non-zero with `status = "issues_found"` when integrity issues are detected.
- `bundle import --dry-run` returns projections only and does not mutate DB/files.

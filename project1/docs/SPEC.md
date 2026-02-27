# Project 1 Technical Specification

## Status

- Last updated: 2026-02-19
- Scope: Baseline architecture and governance documentation after review fixes 1-8

## Project Goal

Generate a daily arXiv digest for configured astronomy/cosmology interests with predictable runtime behavior, low operational overhead, and explicit quality gates.

## Runtime Architecture

The runtime is a three-stage pipeline with explicit optional paths:

1. Fetch + dedup:
- Fetch new papers from arXiv RSS by default (`--source rss`) or Atom API (`--source api`).
- Deduplicate by scanning recent digest files and extracting processed arXiv IDs.
  - Default dedup window: last 2 days (`--dedup-days`).
  - Dedup can be disabled with `--no-dedup`.
- Apply update cutoff using latest digest date unless `--debug` is set.

2. Stage 1 local filter (mandatory):
- Run embedding-based local relevance filtering with `song_db` interest model.
- Reject papers below configured `local_filter.threshold`.
- Stage 1 failure is fatal by design.

3. Stage 2 scoring (always one path):
- Default path: topic-embedding scorer (local, no LLM scoring calls).
- Optional path: LLM scoring when `--use-llm-scoring` is set.
- LLM provider resolution is primary-first with fallback providers next.

4. Stage 3 summary generation (optional):
- Enabled by default, disabled with `--no-summary`.
- When disabled or unavailable, abstracts are used directly.

5. Digest output:
- Render Markdown grouped by tiers.
- Persist under `arxiv_digest/archive/YYYY/arxiv-YYYY-MM-DD.md`.
- Output location can be overridden by CLI:
  - `--output PATH` for an explicit file path.
  - `--output-dir DIR` or `--dir DIR` to keep default naming in a custom directory.

## Key Modules

- `src/main.py`: CLI entrypoint, mode handling, orchestration.
- `src/config.py`: YAML parsing, dataclass validation, provider registry contract.
- `src/arxiv_fetcher.py`: RSS/API fetch, parsing, source filtering.
- `src/scorer.py`: Stage 2 scoring logic and tier assignment.
- `src/get_llm_score.py`: single-paper LLM scoring utility.
- `src/summarizer.py`: summary generation + fallbacks.
- `src/formatter.py`: digest formatting + atomic writes.
- `src/llm_client.py`: provider clients, timeout propagation, fallback client.
- `src/state.py`: state file helpers and run metadata.
- `main.py`: compatibility wrapper to `src/main.py`.

## Configuration Contract

Primary configuration file: `config.yaml`

- `category`: fetch scope (primary/secondary arXiv categories).
- `topics`: relevance semantics used by local and LLM scoring.
- `projects`: title-based project boosts/floor behavior.
- `providers`: provider registry (`api_key_env`, `base_url`, model, client type).
- `llm`: primary provider/model defaults.
- `llm_fallback`: ordered fallback providers.
- `local_filter`, `topic_scorer`, `llm_scoring`, `summary`: stage behavior.
  - `llm_scoring.summary_tiers` controls which LLM-scored tiers proceed to summary generation.
  - `could_be_interesting` remains in the digest as title+link only.
- `output`, `api`: paths and transport/runtime controls.

## Design Decisions

1. RSS-first default:
- RSS follows announcement-date listings and avoids API submission-date drift.
- API remains available for explicit lookback workflows (`--days`, `--source api`).

2. Mandatory local filter:
- Stage 1 enforces domain relevance before downstream work.
- This reduces noise and unnecessary scoring/summarization load.

3. Primary-first provider ordering:
- Runtime and utility scripts align on `[primary] + fallback_without_duplicates`.
- Avoids accidental preference of fallback providers.

4. Fail-safe summary behavior:
- Summary generation is optional; digest quality degrades gracefully to abstract output.

5. Bounded digest-history dedup:
- Dedup scans only a recent date window by default (2 days) for runtime efficiency.
- Operators can expand the window (`--dedup-days`) or disable dedup (`--no-dedup`) when safe.

6. CI quality gates:
- `.github/workflows/ci.yml` enforces `uv run ruff check .` and `uv run pytest -q`.

## Reliability Invariants

- No destructive git/file operations are required for normal runtime.
- Stage 1 must succeed or runtime exits non-zero.
- Runtime output writes are atomic for digest/state files.
- Timeout flags are propagated through LLM client calls to network requests.

## Governance and Documentation

- This file (`docs/SPEC.md`) is the canonical architecture reference.
- Work planning and verification tracking are maintained in `docs/todo.md`.
- Review/audit context and findings are logged in `docs/journal/`.

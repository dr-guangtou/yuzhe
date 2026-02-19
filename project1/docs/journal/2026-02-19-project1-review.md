# Project 1 Review Summary (Structure, Code, Documentation)

Date: 2026-02-19
Branch: `fix/get-llm-score-thresholds`

## Scope

Reviewed repository structure, core pipeline modules, support scripts, documentation, and test/lint status.

## Verification Snapshot

- `uv run pytest -q` passed: `22 passed`.
- `uv run ruff check .` failed with `38` findings (quality debt not currently gated).

## Prioritized Findings

1. Critical: `src/get_llm_score.py` uses `config.scoring.*` but thresholds now live under `config.llm_scoring.tier_thresholds`; this can raise runtime `AttributeError`.
2. High: Summary fallback path can be slow when no LLM client exists because it still enters retry/backoff code before falling back.
3. High: Dedup checks only the latest digest file, which can re-include older papers in recovery windows.
4. High: Latest digest date lookup is year-limited and can break at year boundaries.
5. Medium: `get_llm_score` default provider order prefers fallback over primary.
6. Medium: `src/check_llm_apis.py --timeout` is accepted but not wired into actual HTTP timeout usage.
7. Medium: Documentation and entrypoint drift (`main.py` placeholder, stale usage text in `song_db/README.md`, legacy `src/main_old.py` still present).
8. Medium: Lint issues are not enforced in CI quality gates.
9. Documentation governance gap: missing `docs/SPEC.md` and task-tracking doc alignment.

## Recommended Remediation Order

1. Fix `src/get_llm_score.py` threshold schema usage.
2. Fast-path summary fallback when LLM client is unavailable.
3. Harden update/dedup logic across date windows and years.
4. Remove/mark stale entrypoints and align docs with actual CLI behavior.
5. Add CI checks for `ruff`, `pytest`, and basic CLI smoke tests.

## Progress Update (2026-02-19)

- Completed item 1: `src/get_llm_score.py` now reads thresholds from `config.llm_scoring.tier_thresholds`.
- Completed item 2: Stage 3 now fast-paths summary fallback when no LLM client is available, avoiding retry/backoff delays.
- Status note: item 2 is marked done per user confirmation after implementation and test coverage update.
- Completed items 3 and 4 together:
  - Dedup now reads IDs from all existing digest files instead of only the latest file.
  - Latest digest discovery now scans across all archive years instead of current year only.
- Completed medium-priority items 5, 6, 7, and 8 together:
  - Item 5: `get_llm_score` now resolves providers with primary-first order, then fallback providers.
  - Item 6: `--timeout` in `check_llm_apis.py` is now passed through to client calls and actual HTTP request timeout handling.
  - Item 7: Entry point and docs drift reduced:
    - Root `main.py` is now a compatibility wrapper that executes `src/main.py`.
    - Stale integration flags in `song_db/README.md` were replaced with current pipeline modes.
    - Removed legacy backup file `src/main_old.py`.
  - Item 8: Added CI quality gates and aligned lint status:
    - Added `.github/workflows/ci.yml` to run `ruff` and `pytest`.
    - Added `ruff` to dev dependencies.
    - Resolved existing lint violations so `uv run ruff check .` now passes.

# Governance Closure Todo (Review Item 9)

## Scope

Close the remaining governance/documentation gap by adding a current technical spec and validating runtime confidence with one small live run plus lint/test verification.

## Plan Checklist

| Status | Item | Verification |
| --- | --- | --- |
| [x] | Create and populate `docs/SPEC.md` with architecture and design decisions | File created and reviewed for current runtime flow |
| [x] | Run one small live digest check | `uv run python src/main.py --debug --use-llm-scoring --limit 3 --no-summary` |
| [x] | Run lint gate | `uv run ruff check .` |
| [x] | Run test gate | `uv run pytest -q` |
| [x] | Record outcomes and close review item 9 | Review section below completed |

## Verification Checklist

- Runtime: `uv run python src/main.py --debug --use-llm-scoring --limit 3 --no-summary`
- Lint: `uv run ruff check .`
- Tests: `uv run pytest -q`

## Review

- Date: 2026-02-19
- Branch: `docs/spec-governance-item9`
- Result: complete
- Notes:
  - Runtime check passed; digest written to `arxiv_digest/archive/2026/arxiv-2026-02-19.md`.
  - Runtime summary: 3 papers fetched after Stage 1 filter; 2 papers retained after LLM scoring.
  - Lint check passed: `uv run ruff check .` -> `All checks passed!`
  - Test check passed: `uv run pytest -q` -> `30 passed in 0.29s`.

---

# Dedup Window and Toggle (2026-02-27)

## Scope

Add runtime controls to bound digest-history dedup to a short date window and allow explicit opt-out when dedup is unnecessary.

## Plan Checklist

| Status | Item | Verification |
| --- | --- | --- |
| [x] | Add CLI flags for dedup window and dedup disable | `--dedup-days`, `--no-dedup` present in `src/main.py` |
| [x] | Apply dedup only within configurable date window | Dedup uses `get_previous_digest_ids(..., since_date=...)` |
| [x] | Add tests for dedup cutoff behavior | New tests in `tests/test_digest_filename.py` |
| [x] | Update user-facing docs | `README.md`, `docs/SPEC.md` updated |
| [x] | Run quality gates | `ruff` and `pytest` passed |

## Verification Checklist

- Lint: `uv run ruff check src/main.py tests/test_digest_filename.py README.md docs/SPEC.md`
- Tests: `uv run pytest -q tests/test_digest_filename.py`
- Full tests: `uv run pytest -q`

## Review

- Date: 2026-02-27
- Branch: `feature/dedup-window-no-dedup`
- Result: complete
- Notes:
  - Dedup now defaults to scanning only the last 2 days of digest files.
  - `--no-dedup` bypasses digest-history dedup entirely.
  - All tests passed after changes (`35 passed`).

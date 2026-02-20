Goal: finish the remaining governance/documentation gap (review item 9) and run a final runtime confidence check.

Current status:
- `main` is at commit `18c0b3e` with medium-priority fixes merged.
- Lint and tests pass (`ruff` + `pytest`).
- Core reliability fixes are complete (items 1-8).

Key files:
- `docs/journal/2026-02-19-project1-review.md`
- `src/main.py`, `src/get_llm_score.py`, `src/llm_client.py`
- `.github/workflows/ci.yml`

First actions:
1. Create `docs/SPEC.md` with current architecture and decisions.
2. Create `docs/todo.md` with remaining work and verification checklist.
3. Run one small live digest check: `uv run python src/main.py --debug --use-llm-scoring --limit 3 --no-summary`.

Verification commands:
- `uv run ruff check .`
- `uv run pytest -q`

Read `docs/journal/2026-02-27_handover.md` for full context.

Immediate next step: decide whether to run a real provider-backed digest sanity check with the new `summary_tiers`/`> 6.0` threshold behavior, or keep this session scoped to code-only changes and prepare the branch for review/commit. If calibrating further, compare a few user-judged borderline papers against the current LLM scores and adjust prompt/thresholds from evidence.

Warnings and verification gaps:
- There are unrelated pre-existing changes and untracked parent-level journal files outside this task scope; do not revert them.
- `config.yaml` had pre-existing modifications before this session.
- Real API-backed validation of the new 5-second LLM pacing was not run yet.

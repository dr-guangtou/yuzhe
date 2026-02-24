# Project 4: LaMian

LaMian is a local-first visual knowledge base for research figures and screenshots.

This folder is the planning and governance home for Project 4.  
Implementation code lives in `project4/lamian/`.

## Current Status

- Roadmap state: active development
- Current phase: Phase 1.x CLI expansion backlog execution
- Primary goal: close highest-priority post-review CLI gaps before Phase 2 GUI baseline

## Implemented Pre-GUI Scope

- `query save|run|list|delete` (saved search rules)
- `import` (batch ingest with strict provenance templates)
- `doctor` (read checks + DB-only safe fixes)
- `collection` (hybrid static and dynamic collections)
- `bundle export|inspect|import` (`tar.gz` for portable snapshots and preflight)
- `verify` (read-only filesystem-vs-DB integrity checks)

Recent hardening controls:

- `query` and `collection` support `--reference-mode auto|id|name`
- `bundle import` supports `--fail-on-link-loss`, `--dry-run`, and `--on-conflict skip|error|replace`

## Document Map

- `PLAN.md`: implementation phases, milestones, and gating criteria
- `SPEC.md`: technical and product specification
- `TODO.md`: actionable checklist with progress tracking and review
- `DECISIONS.md`: architecture and scope decisions
- `RISK_REGISTER.md`: known risks and mitigations
- `AGENTS.md`: project-level agent execution rules
- `CLAUDE.md`: project context and workflow guidance
- `journal/`: dated development journal entries
- `lamian/docs/`: extraction-ready standalone documentation set

## Directory Roles

- `project4/`: planning, requirements, decisions, and project management artifacts
- `project4/lamian/`: actual application development directory (source code)
- `project4/lamian/docs/`: standalone-repo oriented docs for future repository split

# Risk Register (LaMian)

| Risk ID | Risk | Severity | Probability | Mitigation | Owner | Status |
| --- | --- | --- | --- | --- | --- | --- |
| R-001 | Scope creep from early automation features (arXiv/publication modes) | High | High | Keep MVP limited to vault, metadata, tags, links, search, CLI+basic GUI | agent + user | Open |
| R-002 | Metadata quality drift if provenance is optional | High | Medium | Enforce strict provenance at ingest and validate required fields | agent | Open |
| R-003 | GUI framework choice may delay delivery | Medium | Medium | Implement CLI core first and keep GUI adapter minimal in MVP | agent | Open |
| R-004 | Data inconsistency between DB and exports | High | Medium | Define DB as canonical source and make exports one-way in MVP | agent | Open |
| R-005 | Copyright/source compliance ambiguity for imported figures | Medium | Medium | Store source metadata and add user-facing compliance note in docs | user + agent | Open |
| R-006 | Cross-platform ambitions increase complexity too early | Medium | Medium | Prioritize macOS validation, keep architecture portable without forcing full parity early | user + agent | Open |

## Review Cadence

- Revisit this register at each phase gate in `PLAN.md`.
- Promote any recurring implementation issue into this register with mitigation owner.


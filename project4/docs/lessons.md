# Lessons Learned (Project 4)

## 2026-02-19

- Keep planning artifacts in `project4/` and implementation in `project4/lamian/` to reduce context drift.
- Lock storage and provenance decisions early to avoid refactoring the full pipeline later.
- Use an idempotent migration path from day one so `init` can safely run multiple times without schema drift.
- If a subdirectory is expected to become a standalone repository, add repo-local `.gitignore`, license, toolchain pin, and docs early.

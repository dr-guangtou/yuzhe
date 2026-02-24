# Lessons

- When the full gate fails due to `static.crates.io` DNS resolution errors, record the failure context and retry once network access is restored before marking the slice complete.
- When `clippy -D warnings` flags tuple-heavy query row parsing (`type_complexity`), extract a dedicated row struct immediately to keep service code readable and gate-safe.
- Under `clippy -- -D warnings`, avoid adding error enum variants unless they are used in live paths, or dead-code warnings will fail the full gate.
- When adding dual-output branches (`--json` vs human-readable), prefer `else if` forms directly instead of nested `else { if ... }` blocks to avoid `clippy::collapsible_else_if` gate failures.
- With `eframe = 0.27.x`, `run_native` expects an app-creator closure returning `Box<dyn App>` (not `Result`), so use a direct boxed app factory to avoid type-mismatch gate failures.

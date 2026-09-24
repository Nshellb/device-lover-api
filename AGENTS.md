# Project Instructions

## Verification

- Do not run a full build (`cargo build` / `cargo build --release`) or `cargo clippy` unless the user explicitly requests it.
- For routine changes, prefer `cargo check` instead — it validates the code without a live DB connection (queries are dynamic, not compile-time macros) and without producing binaries — and only run it when it actually helps verify the change.

# P55.1 Codex tasks

## Task A — Audit only

Read AGENTS.md and docs/P55_1_SPEC.md.
Audit the current repo.
Do not modify code.
Return missing pieces for:
- diagnostics
- valid/invalid corpus
- JSON export
- Python/Rust equivalence
- CLI commands
- CI gates

## Task B — Diagnostics

Implement stable typed diagnostics with error codes.
Add tests for all invalid examples.
Do not weaken existing tests.

## Task C — Mutant corpus

Expand examples/invalid.
Required mutants:
- bad_version
- guard_active
- snapshot_full
- bad_action
- bad_safety
- layout_mismatch
- index_mismatch
- threshold_bad
- missing_families
- unknown_family

## Task D — JSON export

Implement deterministic JSON export.
Add tests that compare expected canonical output.

## Task E — CLI and CI

Ensure atlasc supports:
- check
- explain
- export --format json
- bench --mode smoke

Update GitHub Actions:
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
valid .atlas check
invalid .atlas refusal

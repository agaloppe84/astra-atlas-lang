# ASTRA-P57 Audit

Date: 2026-04-30

Scope: static repository audit for the ASTRA-P57 classical reintegration sprint.
No Rust validation commands were run for this audit, so this file does not claim
local cargo success or CI success.

## Repository Shape

- Cargo is currently a single root package, `atlas-cli`, not a multi-crate
  workspace. There is no `[workspace]` section.
- The package exposes one library, `astra_atlas_lang`, from `src/lib.rs`.
- The package exposes two binaries from `src/main.rs`: `atlas-cli` and `atlasc`.
  `atlasc` is a compatibility alias.
- There are no separate `atlas-parser`, `atlas-core`, or `atlas-runtime` crates.
  Parser, strict typechecker, diagnostics, and canonical export live in
  `src/lib.rs`; the P56 smoke runtime lives in `src/runtime.rs`; CLI dispatch
  lives in `src/main.rs`.
- `benches/` is absent. The only benchmark-like entry point is the CLI smoke
  command `bench --mode smoke`.
- `.github/workflows/rust.yml` exists and runs format, build, tests, P55.1
  check/export, invalid mutant refusal, and P56 runtime smoke checks.
- `target/` build artifacts are currently tracked by git, and no `.gitignore`
  file is tracked. P57 should not add more heavy generated artifacts.

## 1. P56 Runtime Capabilities Implemented

The P56 runtime is a deterministic smoke runtime, not a full production memory
engine.

Implemented capabilities:

- Build a `RuntimeConfig` from a checked `.atlas` program.
- Preserve strict P53 runtime settings, including `strict_p53=true` and
  `snapshot=incremental_manifest`.
- Select deterministic smoke workload families:
  `stream_processing`, `sparse_index`, and `columnar_table`.
- Encode deterministic records, read them, update a subset, snapshot runtime
  state, rebuild from the snapshot, and compare summaries.
- Refuse encoding into `guard` or any `action=refuse` family.
- Track deterministic counters: encoded segments, reads, updates, snapshots,
  rebuilds, pseudo-latencies, guard encodes, dangerous encoded values, and
  checksums.
- Export deterministic runtime metrics JSON with P56 gate fields:
  `P56_G0_build_test_ci` through `P56_G8_ci_source_of_truth`.
- Treat build/test/CI as external source-of-truth fields in the metrics JSON:
  `P56_G0_build_test_ci` and `P56_G8_ci_source_of_truth` are
  `external_required`.
- Check one embedded invalid regression in runtime metrics:
  `examples/invalid/snapshot_full.atlas` must still return
  `E_SNAPSHOT_FULL_STRICT`.

Important limitation:

- The runtime's internal `invalid_regression_checked` currently checks only
  `snapshot_full`. The broader invalid corpus is covered by Rust tests and CI,
  not by the runtime metrics function itself.

## 2. CLI Commands Present

Defined commands:

- `atlas-cli check <file.atlas>`
- `atlas-cli explain <E_CODE>`
- `atlas-cli export <file.atlas> --format json`
- `atlas-cli run <file.atlas> --mode smoke`
- `atlas-cli metrics <file.atlas> --format json`
- `atlas-cli bench --mode smoke`

Additional behavior:

- Running with no arguments checks `examples/p53_strict.atlas`.
- Running with a single path also checks that path.
- The same command surface is available through the `atlasc` binary alias.

## 3. Valid Examples

Current valid corpus:

- `examples/p53_strict.atlas`
- `examples/valid/p53_strict.atlas`

Both valid examples define the same strict P53 program with 12 families:

- `guard`
- `stream_processing`
- `sparse_index`
- `image_field_surrogate`
- `log_request_index`
- `columnar_table`
- `graph_lowrank_surrogate`
- `critical_sparse_archive`
- `compressible_but_wrong`
- `field_surrogate`
- `topological_field`
- `local_global_conflict`

The canonical JSON golden export is stored at
`tests/golden/p53_strict.json`.

## 4. Invalid Mutants

Current invalid corpus and expected diagnostic intent:

| File | Expected code | Mutation |
| --- | --- | --- |
| `examples/invalid/bad_version.atlas` | `E_VERSION_UNKNOWN` | `atlas version=9.9` |
| `examples/invalid/guard_active.atlas` | `E_GUARD_ACTIVE` | `guard` changed from refuse sentinel to active sparse config |
| `examples/invalid/snapshot_full.atlas` | `E_SNAPSHOT_FULL_STRICT` | `snapshot=full` under `strict_p53=true` |
| `examples/invalid/bad_action.atlas` | `E_ACTION_UNKNOWN` | unknown `stream_processing` action |
| `examples/invalid/bad_safety.atlas` | `E_SAFETY_UNKNOWN` | unknown safety mode |
| `examples/invalid/layout_mismatch.atlas` | `E_LAYOUT_INDEX_MISMATCH` | layout does not match P53 table |
| `examples/invalid/index_mismatch.atlas` | `E_LAYOUT_INDEX_MISMATCH` | index does not match P53 table |
| `examples/invalid/threshold_bad.atlas` | `E_THRESHOLD_INVALID` | threshold outside allowed range |
| `examples/invalid/missing_families.atlas` | `E_MISSING_FAMILIES` | omits required strict P53 family |
| `examples/invalid/unknown_family.atlas` | `E_FAMILY_UNKNOWN` | adds unknown `alien` family |
| `examples/invalid/missing_semicolon.atlas` | `E_PARSE` | missing terminating semicolon |
| `examples/invalid/active_without_safety.atlas` | `E_ACTIVE_WITHOUT_SAFETY` | active action with `safety=refuse` |
| `examples/invalid/dangerous_encoded.atlas` | `E_ACTION_UNKNOWN` | encoded-dangerous action string is not accepted |
| `examples/invalid/malformed_family.atlas` | `E_FIELD_MISSING` | bare `family;` line without a name |

## 5. Existing P55, P55.1, and P56 Test Coverage

Rust tests:

- `tests/atlas_tests.rs`
  - accepts the two valid P53 strict examples
  - refuses all 14 invalid mutants with expected diagnostic code, family, field,
    and message fragment
  - checks stable diagnostic code strings and `from_str`
  - checks canonical JSON against `tests/golden/p53_strict.json`
  - checks canonical family ordering
  - refuses invalid programs during JSON export
  - explicitly preserves `strict_p53`, refuses active guard, and refuses
    `snapshot_full`
- `tests/runtime_tests.rs`
  - instantiates runtime config from P53 strict
  - runs deterministic smoke workload
  - checks P56 smoke status and core metrics
  - checks deterministic metrics JSON
  - checks metrics JSON file path reporting
  - checks snapshot/rebuild roundtrip state equivalence
  - verifies invalid files still fail runtime entry points for bad version and
    strict snapshot full

CI workflow:

- `cargo fmt --all -- --check`
- `cargo build --workspace`
- `cargo test --workspace`
- valid `.atlas` check
- canonical JSON export diff against golden output
- invalid mutant refusal with expected diagnostic codes
- P56 runtime smoke and metrics checks

Docs:

- `docs/P55_1_SPEC.md` records P55.1 gates.
- `docs/P55_1_CODEX_TASKS.md` records the P55.1 task breakdown.
- `README.md` lists current cargo and CLI commands.
- `artifacts/p55_1/baseline_codespaces_log.txt` exists, but it contains old
  baseline output and package-name errors from before the current `atlas-cli`
  package naming was aligned. It should not be treated as current validation.

## 6. Missing for ASTRA-P57 Classical Reintegration

P57 is not represented yet as a named sprint in code, examples, docs, tests, or
trace artifacts.

Missing pieces:

- A P57 classical invariant map that says which classical ASTRA invariants must
  be expressible in the existing strict `.atlas` format.
- P57-specific valid examples, if the existing `p53_strict.atlas` corpus is not
  enough to demonstrate classical reintegration.
- P57-specific invalid mutants, if classical reintegration requires additional
  negative cases beyond the P55.1 mutants.
- Tests that explicitly name P57 and verify the classical invariant map.
- A lightweight P57 validation trace format for Codespaces and GitHub Actions
  handoff.
- A small summary of validation results generated after real Codespaces
  execution.
- A clear rule that `bench --mode smoke` is a deterministic smoke/proxy check,
  not a benchmark result.
- Repo hygiene around tracked `target/` artifacts and missing `.gitignore`.
  This should be handled separately and deliberately, because it touches many
  tracked generated files.

## 7. Minimal Recommended Changes

Recommended next changes, kept intentionally small:

1. Add a P57 docs/spec file or section that maps each classical ASTRA invariant
   to existing `.atlas` constructs: runtime settings, family table entries,
   guard refusal, snapshot policy, canonical export, and runtime smoke metrics.
2. Add only the smallest P57 `.atlas` corpus needed to demonstrate classical
   reintegration. Prefer reusing the existing strict family table instead of
   expanding `.atlas` into a general-purpose language.
3. Add focused P57 tests that assert the invariant map, valid corpus acceptance,
   invalid corpus refusal, stable diagnostic codes, and canonical export.
4. Add a P57 validation summary template and a small JSON summary schema for
   Codespaces handoff.
5. Keep CI strict. If P57 checks are added to CI, make them additional gates,
   not replacements for P55.1/P56 gates.
6. Plan a separate repository hygiene cleanup for tracked `target/` artifacts
   and `.gitignore`. Do not mix that cleanup with semantic P57 language/runtime
   changes.

No invasive code changes are recommended before the P57 invariant map is agreed.

## 8. Lightweight P57 Trace Files to Keep

Recommended versioned trace layout after real Codespaces validation:

- `docs/validation/astra-p57-validation-summary.md`
- `artifacts/p57/astra-p57-validation-summary.json`

Do not commit:

- `target/`
- build artifacts
- huge raw stdout dumps
- benchmark dumps
- unbounded GitHub Actions logs
- temporary files from `/tmp`

If raw logs are useful during debugging, keep them outside the repo or summarize
them into the two small files above.

### Markdown Summary

`docs/validation/astra-p57-validation-summary.md` should be the human-readable
handoff file. Recommended contents:

- sprint: `ASTRA-P57`
- trace schema version: `1`
- repository and commit SHA
- Codespaces environment summary
- GitHub Actions run URL, if available
- command table with command, status, exit code, and one-line evidence
- valid examples accepted
- invalid mutants refused with observed diagnostic codes
- canonical JSON export status
- P56 runtime smoke metrics excerpt
- P57 classical invariant coverage matrix
- notes and open issues

### JSON Summary

`artifacts/p57/astra-p57-validation-summary.json` should be the small
machine-readable handoff file. Recommended top-level shape:

```json
{
  "schema_version": 1,
  "sprint": "ASTRA-P57",
  "repo": "agaloppe84/astra-atlas-lang",
  "commit": "<commit-sha>",
  "validated_at": "<iso-8601>",
  "source": "codespaces",
  "commands": [
    {
      "command": "cargo fmt --all -- --check",
      "status": "pass",
      "exit_code": 0,
      "summary": "format check passed"
    }
  ],
  "valid_examples": [],
  "invalid_mutants": [],
  "canonical_exports": [],
  "runtime_metrics": {
    "mode": "smoke",
    "p56_status": "<observed>",
    "encoded_segments_total": null,
    "rebuild_matches": null,
    "strict_p53_preserved": null
  },
  "p57_classical_invariants": [],
  "ci": {
    "status": "not_checked",
    "url": null
  }
}
```

Keep the JSON concise. It should capture statuses and important observed values,
not full command output.

## Validation Commands for Future Codespaces Run

For P57 validation, run and summarize at least:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo run -p atlas-cli -- check examples/p53_strict.atlas
cargo run -p atlas-cli -- check examples/invalid/snapshot_full.atlas
cargo run -p atlas-cli -- export examples/p53_strict.atlas --format json
cargo run -p atlas-cli -- run examples/p53_strict.atlas --mode smoke
cargo run -p atlas-cli -- metrics examples/p53_strict.atlas --format json
```

The `snapshot_full` command is expected to fail with
`E_SNAPSHOT_FULL_STRICT`; record that as a passing refusal in the summary trace.

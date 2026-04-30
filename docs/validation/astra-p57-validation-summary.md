# ASTRA-P57 validation summary

## Git context

- Branch: `main`
- Commit before final P57 commit: `7e57d0b0c95ef8b41969981003defe4e4b56d10f`
- Date UTC: `2026-04-30T16:20:30Z`

## Environment

- Machine: Mac M1 / aarch64-apple-darwin
- ASTRA root: `/Users/work/Astra`
- Cargo: `/Users/work/Astra/.cargo/bin/cargo`
- Rustc: `/Users/work/Astra/.cargo/bin/rustc`
- Cargo version: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- Rustc version: `rustc 1.95.0 (59807616e 2026-04-14)`
- Rustup home: `/Users/work/Astra/.rustup`
- Cargo target dir: `/Users/work/Astra/.cargo-target`

## Required gates

| Gate | Status |
|---|---|
| cargo fmt --all -- --check | PASS |
| cargo build --workspace | PASS |
| cargo test --workspace | PASS |
| atlas_tests.rs | PASS, 9 passed |
| p57_tests.rs | PASS, 7 passed |
| runtime_tests.rs | PASS, 7 passed |
| check examples/p53_strict.atlas | PASS |
| export examples/p53_strict.atlas --format json | PASS |
| bench --mode smoke | PASS |
| run examples/p53_strict.atlas --mode smoke | PASS |
| metrics examples/p53_strict.atlas --format json | PASS |
| report examples/p53_strict.atlas --format json | PASS |
| invalid examples rejected | PASS, 13 rejected / 13 checked |

## P57 report summary

```json
{
  "astra_iteration": "ASTRA-P57",
  "atlas_version": "0.1",
  "strict_p53_enabled": true,
  "family_count": 12,
  "active_family_count": 11,
  "refused_family_count": 1,
  "guard_refused": true,
  "snapshot_policy": "incremental_manifest",
  "snapshot_full_refused": true,
  "runtime_available": true,
  "encode_available": true,
  "read_available": true,
  "update_available": true,
  "snapshot_available": true,
  "rebuild_available": true,
  "metrics_available": true,
  "export_json_stable": true,
  "family_thresholds_present": true,
  "safety_policies_present": true,
  "layout_policies_present": true,
  "index_policies_present": true,
  "valid_examples_pass": true,
  "invalid_examples_fail": true,
  "runtime_smoke_path_exists": true,
  "report_json_stable": true,
  "cargo_tests_no_regression": null,
  "external_validation_required": true,
  "astra_p57_decision": "RECALIBRATE_P57"
}
```

## Invalid examples

All invalid examples in `examples/invalid/*.atlas` were rejected.

Checked examples:

- active_without_safety.atlas
- bad_action.atlas
- bad_safety.atlas
- bad_version.atlas
- dangerous_encoded.atlas
- guard_active.atlas
- index_mismatch.atlas
- layout_mismatch.atlas
- malformed_family.atlas
- missing_families.atlas
- missing_semicolon.atlas
- snapshot_full.atlas
- threshold_bad.atlas
- unknown_family.atlas

## Final local status

`LOCAL_PASS_CI_REQUIRED`

## Blockers

- GitHub Actions / remote CI still required.
- P57 decision remains conservative as `RECALIBRATE_P57` until external CI evidence is available.
- Non-blocking warning: `src/main.rs` is present in multiple bin targets, `atlas-cli` and `atlasc`.

## Files to transmit to ChatGPT

- `docs/validation/astra-p57-validation-summary.md`
- `artifacts/p57/astra-p57-validation-summary.json`
- `docs/astra-p57-reintegration-classique.md`
- `tests/golden/p57_report.json`
- final commit hash after push
- GitHub Actions status

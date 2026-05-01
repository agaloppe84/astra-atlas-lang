# ASTRA-P70 — Contract Replay and Test Stack Hygiene Analysis

## 1. Executive summary

P70 adds the first contract replay layer after P69. It checks that the P69
representation contract remains coherent when replayed across multiple
address-fiber fixtures, compares declared contract bytes with deterministic
measured runtime bytes, detects contract drift, and introduces a mandatory Rust
test stack audit.

Current local decision:

```text
RECALIBRATE_P70_CONTRACT_DRIFT
```

No hard drift is observed in the local replay, but the first replay layer stays
conservative until more contract-bound campaigns exist.

## 2. Position after P69

P69 promoted the address-fiber representation contract runtime:

- `parse_ok = true`;
- `typecheck_ok = true`;
- `all_storage_counted = true`;
- `hidden_storage_risk = low`;
- `total_contract_bytes = 176128`;
- `virtual_effective_units = 8448000`;
- `contract_ratio_effective_per_byte = 47.965116`;
- invalides P69: `7/7` refused;
- decision: `PROMOTE_P69_ADDRESS_FIBER_CONTRACT_RUNTIME`.

P70 checks whether this contract remains honest under replay.

## 3. New process rule: test stack hygiene

Every repo-first milestone must audit the local Rust test stack:

- which tests still protect live invariants;
- which tests are historical non-regression guards;
- which tests are obsolete, redundant or too coupled to old architecture;
- which tests should be kept, updated, merged or deleted.

The audit is versioned in:

```text
docs/analysis/ASTRA-P70-test-stack-audit.md
```

## 4. Test stack audit

Summary:

| Status | Count |
|---|---:|
| KEEP | 4 |
| UPDATE | 1 |
| MERGE | 0 |
| DELETE | 0 |
| HISTORICAL_NON_REGRESSION | 10 |

No tests were deleted. `tests/atlas_tests.rs` was updated to include the P70
invalid drift mutants, and `tests/p70_tests.rs` was added.

## 5. Procedural contract replay

P70 adds `src/p70.rs`.

Main types:

- `ContractReplayReport`;
- `ContractReplayFixture`;
- `DeclaredVsMeasuredBytes`;
- `ContractDriftDetector`;
- `TestStackAuditSummary`;
- `P70Decision`.

Main command:

```bash
cargo run -p atlas-cli -- contract-replay examples/valid/p69_address_fiber_contract.atlas \
  --fixtures all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --tolerance-percent 5.0 \
  --export-dir artifacts/p70/contract_replay_standard \
  --format json

bash scripts/build_report.sh reports/P70/RPA_ASTRA-P70-Results_contract-replay-test-stack_v1.0_2026-05-01.tex
```

## 6. Fixtures

The replay uses four deterministic address-fiber fixtures inspired by P64/P66:

- `log_event_fiber_replay`;
- `sparse_row_fiber_replay`;
- `json_record_fiber_replay`;
- `hybrid_field_tile_fiber_replay`.

## 7. Declared vs measured bytes

Local replay summary:

| Metric | Value |
|---|---:|
| fixture_count | 4 |
| declared_contract_bytes | 176128 |
| measured_runtime_bytes_min | 177664 |
| measured_runtime_bytes_median | 180224 |
| measured_runtime_bytes_max | 182272 |
| max_byte_delta | 6144 |
| max_byte_delta_percent | 3.488372 |
| tolerance_percent | 5.000000 |

## 8. Drift detector

Drift statuses:

- `NO_DRIFT`;
- `WARN_DRIFT`;
- `HARD_DRIFT`;
- `INVALID_CONTRACT`.

Local replay status:

```text
NO_DRIFT
```

## 9. Invalid contracts

P70 adds five invalid drift fixtures:

- `p70_cache_unaccounted.atlas`;
- `p70_journal_unaccounted.atlas`;
- `p70_actor_state_unaccounted.atlas`;
- `p70_missing_audit_metadata.atlas`;
- `p70_hidden_storage_risk_high.atlas`.

They are refused by the P69/P70 contract path.

## 10. Local validation commands

Commands executed locally:

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p70_tests
cargo test --test p69_tests
bash scripts/validate_p58_local.sh

cargo run -p atlas-cli -- contract-replay examples/valid/p69_address_fiber_contract.atlas \
  --fixtures all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --tolerance-percent 5.0 \
  --export-dir artifacts/p70/contract_replay_standard \
  --format json
```

## 11. Replay results

P70 contract replay view:

```text
contract path                  : examples/valid/p69_address_fiber_contract.atlas
fixtures                       : 4
declared contract bytes         : 176128
measured runtime bytes          : min 177664 / median 180224 / max 182272
delta percent                  : max 3.488372
accounted storage ratio         : 1.000000
hidden storage risk             : low
drift status                   : NO_DRIFT
tests kept / updated / deleted  : 4 / 1 / 0
decision                       : RECALIBRATE_P70_CONTRACT_DRIFT
```

## 12. Test stack changes

- Added `tests/p70_tests.rs`.
- Updated `tests/atlas_tests.rs` invalid corpus table from 28 to 33 cases.
- No tests deleted.
- No tests merged.
- Historical non-regression tests kept.
- Local Results PDF compiled with Tectonic via `scripts/build_report.sh`.

## 13. Decisions

P70 decision:

```text
RECALIBRATE_P70_CONTRACT_DRIFT
```

Reason:

- replay passes without hard drift;
- declared-vs-measured delta stays below 5%;
- invalid P70 contracts are refused;
- test stack audit is versioned;
- but this is the first replay layer and still needs more contract-bound
  campaigns before a validation decision.

## 14. Limitations

- Measured runtime bytes are deterministic replay-accounting bytes, not external
  filesystem payload measurements across many datasets.
- The replay uses four synthetic fixtures, not external production datasets.
- No multi-machine replay yet.
- The decision intentionally remains conservative.

## 15. Recommendation for P71

P71 should add externalized contract replay profiles:

- compare declared contract bytes against generated filesystem artifacts;
- replay multiple valid contracts, not only one promoted P69 fixture;
- keep P70 drift mutants as non-regression gates;
- split local-only long tests from CI sanity tests in a documented matrix.

## 16. Reproducibility notes

Generated exports live under `artifacts/p70/` and are ignored by Git.

## 17. Journal

- P70 added the contract replay module and CLI.
- P70 added five invalid drift fixtures.
- P70 added test stack audit documentation.
- P70 kept historical tests as non-regression guards.
- P70 generated Results LaTeX/PDF after local validation.

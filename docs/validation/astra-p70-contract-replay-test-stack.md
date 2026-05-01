# ASTRA-P70 — Contract Replay and Test Stack Hygiene

P70 checks that the P69 representation contract remains coherent when replayed
across deterministic address-fiber fixtures. It also introduces a process rule:
every repo-first milestone must audit the Rust test stack.

## Command

```bash
cargo run -p atlas-cli -- contract-replay examples/valid/p69_address_fiber_contract.atlas \
  --fixtures all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --tolerance-percent 5.0 \
  --export-dir artifacts/p70/contract_replay_standard \
  --format json
```

## Exports

Exports are local-only and ignored by Git:

- `p70_contract_replay_report.json`;
- `p70_contract_replay_fixtures.jsonl`;
- `p70_contract_replay_drift.csv`;
- `p70_contract_replay_summary.md`.

## Drift statuses

- `NO_DRIFT`;
- `WARN_DRIFT`;
- `HARD_DRIFT`;
- `INVALID_CONTRACT`.

Candidate thresholds:

- tolerance: `5.0%`;
- warning threshold: `5.0%`;
- hard threshold: `15.0%`;
- `all_storage_counted` must stay true;
- `hidden_storage_risk` must not be high.

## Test stack hygiene rule

At each repo-first milestone, the Rust test stack must be audited. Obsolete,
redundant or misleading tests must be deleted, merged or recalibrated with
justification. Historical tests should remain when they protect real
non-regression invariants.

The validation source of truth remains local-first. CI remains minimal sanity.

## Validation

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p70_tests
cargo test --test p69_tests
bash scripts/validate_p58_local.sh
```

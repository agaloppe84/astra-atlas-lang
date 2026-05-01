# ASTRA-P64 Address-Local Realish Workloads

## Objective

ASTRA-P64 tests whether an addressable procedural runtime can reduce paid cost
by generating only a local neighborhood around a requested address instead of
materializing the full virtual space.

P64 is a system sprint. It does not change `.atlas`, does not weaken
`strict_p53`, and does not alter the invalid corpus or timing goldens.

## Command

```bash
cargo run -p atlas-cli -- ratio-realish examples/p53_strict.atlas \
  --workload all \
  --policy all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --neighborhood-radius 3 \
  --export-dir artifacts/p64/realish_standard_policy_compare \
  --format json
```

Options:

- `--workload realish_log_events|realish_sparse_csv|realish_json_records|realish_hybrid_field_fixture|all`
- `--policy full-materialization|global-indexed|address-local|all`
- `--mode smoke|standard|ambitious`
- `--runs N`
- `--queries N`
- `--neighborhood-radius N`
- `--export-dir PATH`
- `--format json|markdown`

## Workloads

| workload | role |
|---|---|
| `realish_log_events` | structured timestamp/service/request log events |
| `realish_sparse_csv` | sparse tabular rows and column groups |
| `realish_json_records` | JSON-like records with projections and tags |
| `realish_hybrid_field_fixture` | local patch proxy for `u = g + K_sigma * mu` |

These fixtures are deterministic and lightweight. They are not external
scientific datasets.

## Policies

| policy | interpretation |
|---|---|
| `full-materialization` | baseline cost of generating the declared virtual space |
| `global-indexed` | broader global/indexed generation |
| `address-local` | local neighborhood generation around requested addresses |

## Metrics

P64 reports:

- `virtual_declared_units`
- `virtual_reachable_units`
- `virtual_readable_units`
- `virtual_updatable_units`
- `virtual_safe_units`
- `virtual_effective_units`
- `virtual_generated_units`
- `local_generated_units_per_query`
- `locality_selectivity`
- persisted byte breakdown
- `ratio_effective_per_byte`
- `effective_gain_vs_materialized`
- `generated_gain_vs_materialized`
- local read/update/audit success rates
- guard and unsafe locality counters
- policy comparison decisions

Runtime observations are local and timing-dependent. They are never goldenized.

## Exports

When `--export-dir` is provided, P64 writes:

- `p64_campaign_report.json`
- `p64_runs.jsonl`
- `p64_summary.md`
- `p64_workload_metrics.csv`

Exports under `artifacts/p64/` are local generated artifacts and must not be
committed by default.

## Decisions

Per-workload policy decisions:

- `ADDRESS_LOCAL_STRONG`
- `ADDRESS_LOCAL_PROMISING`
- `GLOBAL_INDEXED_BETTER`
- `FULL_MATERIALIZATION_BASELINE_ONLY`
- `RECALIBRATE_WORKLOAD`
- `NO_GO_UNSAFE_LOCALITY`

Global P64 decisions:

- `RECALIBRATE_P64_ADDRESS_LOCAL_RATIO_MODEL`
- `PROMOTE_P64_ADDRESS_LOCAL_FOR_P65`
- `NO_GO_P64_ADDRESS_LOCALITY`

This sprint keeps the scientific decision conservative. It does not return a
final validation decision.

## Local Validation

Recommended local validation:

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p63_tests
bash scripts/validate_p58_local.sh

cargo run -p atlas-cli -- ratio-realish examples/p53_strict.atlas \
  --workload all \
  --policy all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --neighborhood-radius 3 \
  --export-dir artifacts/p64/realish_standard_policy_compare \
  --format json
```

Ambitious local campaign, not for CI:

```bash
cargo run -p atlas-cli -- ratio-realish examples/p53_strict.atlas \
  --workload all \
  --policy all \
  --mode ambitious \
  --runs 50 \
  --queries 5000 \
  --neighborhood-radius 5 \
  --export-dir artifacts/p64/realish_ambitious_policy_compare \
  --format json
```

## Limits

- P64 realish fixtures are internal and deterministic.
- No external dataset is included.
- No multi-machine run is included.
- No calibrated P64 thresholds exist yet.
- CI remains minimal and should not run long campaigns.

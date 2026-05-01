# ASTRA-P65-2 Local Actor Overhead Calibration

## Objective

P65-2 calibrates the overhead of `single_local_actor`, the best observed P65
strategy. It asks whether local actors can keep the P65 ratio gain while reducing
actor overhead toward a candidate target below 15%.

P65-2 does not change `.atlas`, does not weaken `strict_p53`, does not remove
invalid examples and does not add timing goldens.

## Command

```bash
cargo run -p atlas-cli -- ratio-actors-calibrate examples/p53_strict.atlas \
  --workload all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --radius-grid 1,2,3,5 \
  --budget-grid 262144,524288,1048576,2097152 \
  --cache-grid off,on \
  --journal-grid lazy,compact \
  --query-locality-grid clustered,random,mixed \
  --export-dir artifacts/p65/calibration_standard \
  --format json
```

Options:

- `--workload realish_log_events|realish_sparse_csv|realish_json_records|realish_hybrid_field_fixture|all`
- `--mode smoke|standard|ambitious`
- `--runs N`
- `--queries N`
- `--radius-grid a,b,c`
- `--budget-grid a,b,c`
- `--cache-grid off,on`
- `--journal-grid lazy,compact`
- `--query-locality-grid clustered,random,mixed`
- `--export-dir PATH`
- `--format json|markdown`

## Exports

When `--export-dir` is provided, P65-2 writes:

- `p65_actor_calibration_report.json`
- `p65_actor_calibration_runs.jsonl`
- `p65_actor_calibration_summary.md`
- `p65_actor_calibration_grid.csv`

Exports under `artifacts/p65/` are local generated artifacts and must remain
ignored by Git.

## Metrics

Each configuration reports:

- `actor_net_gain`;
- `actor_overhead_ratio`;
- `actor_overhead_bytes`;
- `ratio_effective_per_byte`;
- `effective_gain_vs_materialized`;
- `cache_hit_rate`;
- `conflicts`;
- `stale_reads`;
- `budget_refusal_count`;
- `generated_units_per_query`;
- `bytes_per_query`;
- `balanced_score`;
- `promotion_candidate`;
- structured decision.

The calibration report also exposes:

- `best_by_ratio`;
- `best_by_overhead`;
- `best_balanced`;
- `pareto_front`;
- `no_go_configs`;
- `decision_reasons`.

## Candidate thresholds

Experimental candidate thresholds:

- `candidate_actor_overhead_ratio_target = 0.15`
- `candidate_min_actor_net_gain = 1.20`
- `candidate_min_cache_hit_rate = 0.45`
- `candidate_max_conflicts = 0`
- `candidate_max_stale_reads = 0`
- `candidate_max_budget_refusal_rate = 0.10`

These thresholds do not automatically validate the architecture. P65-2 remains
conservative unless a later prompt adds a formal paired standard/ambitious
promotion gate.

## Decisions

P65-2 decisions:

- `PROMOTE_P66_LOCAL_ACTOR_ARCHITECTURE`
- `RECALIBRATE_P65_ACTOR_OVERHEAD`
- `NO_GO_P65_ACTOR_OVERHEAD`

This prompt keeps the default scientific decision at:

```text
RECALIBRATE_P65_ACTOR_OVERHEAD
```

## Local validation

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p65_tests
bash scripts/validate_p58_local.sh
```

Standard calibration:

```bash
cargo run -p atlas-cli -- ratio-actors-calibrate examples/p53_strict.atlas \
  --workload all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --radius-grid 1,2,3,5 \
  --budget-grid 262144,524288,1048576,2097152 \
  --cache-grid off,on \
  --journal-grid lazy,compact \
  --query-locality-grid clustered,random,mixed \
  --export-dir artifacts/p65/calibration_standard \
  --format json
```

Ambitious local calibration, not for CI:

```bash
cargo run -p atlas-cli -- ratio-actors-calibrate examples/p53_strict.atlas \
  --workload all \
  --mode ambitious \
  --runs 50 \
  --queries 5000 \
  --radius-grid 2,3,5 \
  --budget-grid 524288,1048576,2097152,4194304 \
  --cache-grid off,on \
  --journal-grid lazy,compact \
  --query-locality-grid clustered,mixed \
  --export-dir artifacts/p65/calibration_ambitious \
  --format json
```

## Limits

- Workloads are still deterministic internal realish fixtures.
- No external dataset is included.
- No multi-machine run is included.
- Compaction, update rate and audit rate are not independent grid dimensions yet.
- CI remains minimal and must not run long calibration campaigns.

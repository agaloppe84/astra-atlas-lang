# ASTRA-P65 Address-Local Actor Runtime

## Objective

ASTRA-P65 tests whether deterministic budgeted local actors can improve the
address-local ratio introduced by P64.

The central question:

```text
address-local generation + budgeted local actor
```

versus direct address-local generation without persistent actor state.

P65 does not change `.atlas`, does not weaken `strict_p53`, does not alter the
invalid corpus and does not add timing goldens.

## Command

```bash
cargo run -p atlas-cli -- ratio-actors examples/p53_strict.atlas \
  --workload all \
  --actor-strategy all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --neighborhood-radius 3 \
  --budget-bytes 1048576 \
  --export-dir artifacts/p65/actors_standard \
  --format json
```

Options:

- `--workload realish_log_events|realish_sparse_csv|realish_json_records|realish_hybrid_field_fixture|all`
- `--actor-strategy no-actor|single-local|specialized-crud|over-agentic|all`
- `--mode smoke|standard|ambitious`
- `--runs N`
- `--queries N`
- `--neighborhood-radius N`
- `--budget-bytes N`
- `--cache on|off`
- `--export-dir PATH`
- `--format json|markdown`

## Actor Strategies

| strategy | interpretation |
|---|---|
| `no-actor` | direct P64 address-local baseline |
| `single-local` | one deterministic budgeted actor per local neighborhood group |
| `specialized-crud` | read/update/delete/audit/cache/compaction split with coordination cost |
| `over-agentic` | excessive actor decomposition used as a degradation guardrail |

Actors are not autonomous intelligence. They are deterministic counted runtime
structures.

## LocalActor Cost Model

Each actor exposes:

- state bytes;
- cache bytes;
- index bytes;
- journal bytes;
- queue bytes;
- audit bytes;
- coordination bytes;
- action counts;
- cache hits and misses;
- evictions and compactions;
- conflicts and stale reads;
- budget refusals.

All actor state and all coordination are counted as real cost.

## Metrics

P65 preserves P64 metrics:

- virtual declared/generated/effective units;
- total persisted bytes;
- ratio effective per byte;
- gain vs materialized;
- locality selectivity.

P65 adds:

- actor overhead bytes and ratio;
- actor net gain;
- actor ratio delta;
- actor bytes delta;
- cache hit rate;
- conflict and stale read counts;
- budget refusal count.

## Exports

When `--export-dir` is provided, P65 writes:

- `p65_actor_campaign_report.json`
- `p65_actor_runs.jsonl`
- `p65_actor_summary.md`
- `p65_actor_metrics.csv`

Exports under `artifacts/p65/` are local generated artifacts and must not be
committed by default.

## Decisions

Per-workload strategy decisions:

- `LOCAL_ACTOR_STRONG`
- `LOCAL_ACTOR_PROMISING`
- `LOCAL_ACTOR_OVERHEAD_TOO_HIGH`
- `SPECIALIZED_ACTORS_TOO_EXPENSIVE`
- `NO_ACTOR_BASELINE_BETTER`
- `NO_GO_ACTOR_CONFLICTS`

Global P65 decisions:

- `PROMOTE_P65_LOCAL_ACTORS`
- `RECALIBRATE_P65_ACTOR_OVERHEAD`
- `NO_GO_P65_LOCAL_ACTORS`

This prompt keeps the scientific decision conservative and defaults to:

```text
RECALIBRATE_P65_ACTOR_OVERHEAD
```

## Local Validation

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p64_tests
bash scripts/validate_p58_local.sh

cargo run -p atlas-cli -- ratio-actors examples/p53_strict.atlas \
  --workload all \
  --actor-strategy all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --neighborhood-radius 3 \
  --budget-bytes 1048576 \
  --export-dir artifacts/p65/actors_standard \
  --format json
```

Ambitious local campaign, not for CI:

```bash
cargo run -p atlas-cli -- ratio-actors examples/p53_strict.atlas \
  --workload all \
  --actor-strategy all \
  --mode ambitious \
  --runs 50 \
  --queries 5000 \
  --neighborhood-radius 5 \
  --budget-bytes 4194304 \
  --export-dir artifacts/p65/actors_ambitious \
  --format json
```

## Limits

- Workloads are still internal deterministic realish fixtures.
- No external dataset is included.
- No multi-machine run is included.
- No calibrated P65 threshold profile exists yet.
- CI remains minimal and should not run long actor campaigns.

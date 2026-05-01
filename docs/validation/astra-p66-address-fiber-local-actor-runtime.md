# ASTRA-P66 Address-Fiber Local Actor Runtime

## Objective

P66 formalizes the address-fiber model:

```text
x in Omega_virtual
F_x = local fiber attached to address x
Eval(c, x) = controlled generation of F_x or F_{N(x,r)}
```

The goal is to compare point fibers, neighborhood fibers and actor-managed
fibers without changing `.atlas`, without weakening `strict_p53`, and without
goldenizing timings.

## Command

```bash
cargo run -p atlas-cli -- ratio-fibers examples/p53_strict.atlas \
  --workload all \
  --fiber-strategy all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --neighborhood-radius 3 \
  --budget-bytes 2097152 \
  --cache on \
  --journal compact \
  --export-dir artifacts/p66/fibers_standard \
  --format json
```

Options:

- `--workload realish_log_events|realish_sparse_csv|realish_json_records|realish_hybrid_field_fixture|all`
- `--fiber-strategy point-fiber|neighborhood-fiber|actor-fiber|actor-neighborhood-fiber|all`
- `--mode smoke|standard|ambitious`
- `--runs N`
- `--queries N`
- `--neighborhood-radius N`
- `--budget-bytes N`
- `--cache on|off`
- `--journal eager|lazy|compact`
- `--update-rate low|medium|high`
- `--audit-rate low|medium|high`
- `--export-dir PATH`
- `--format json|markdown`

## Exports

When `--export-dir` is provided, P66 writes:

- `p66_fiber_campaign_report.json`
- `p66_fiber_runs.jsonl`
- `p66_fiber_summary.md`
- `p66_fiber_metrics.csv`

Exports under `artifacts/p66/` are local generated artifacts and must remain
ignored by Git.

## Metrics

P66 reports:

- base address count and fiber count;
- fiber declared/generated/effective units;
- fiber selectivity and effective ratio;
- payload/index/cache/journal/audit/metadata/actor bytes;
- fiber total bytes;
- fiber ratio effective per byte;
- fiber gain vs materialized;
- update/audit success rates;
- compaction and eviction counts;
- actor overhead ratio;
- address fiber net gain;
- conflicts, stale reads and budget refusals.

## Decisions

Per-workload decisions:

- `FIBER_POINT_STRONG`
- `FIBER_NEIGHBORHOOD_STRONG`
- `ACTOR_FIBER_STRONG`
- `ACTOR_FIBER_PROMISING`
- `FIBER_OVERHEAD_TOO_HIGH`
- `ADDRESS_LOCAL_BASELINE_BETTER`
- `NO_GO_FIBER_UNSAFE`

Global P66 decisions:

- `PROMOTE_P66_ADDRESS_FIBER_ARCHITECTURE`
- `RECALIBRATE_P66_ADDRESS_FIBER_MODEL`
- `NO_GO_P66_ADDRESS_FIBER`

This prompt keeps the scientific decision conservative:

```text
RECALIBRATE_P66_ADDRESS_FIBER_MODEL
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

cargo run -p atlas-cli -- ratio-fibers examples/p53_strict.atlas \
  --workload all \
  --fiber-strategy all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --neighborhood-radius 3 \
  --budget-bytes 2097152 \
  --cache on \
  --journal compact \
  --export-dir artifacts/p66/fibers_standard \
  --format json
```

Ambitious local campaign, not for CI:

```bash
cargo run -p atlas-cli -- ratio-fibers examples/p53_strict.atlas \
  --workload all \
  --fiber-strategy all \
  --mode ambitious \
  --runs 50 \
  --queries 5000 \
  --neighborhood-radius 5 \
  --budget-bytes 4194304 \
  --cache on \
  --journal compact \
  --export-dir artifacts/p66/fibers_ambitious \
  --format json
```

## Limits

- Workloads are deterministic internal realish fixtures.
- No external dataset is included.
- No multi-machine run is included.
- P66 does not change `.atlas`.
- Actor-managed fiber overhead remains above the candidate 15% threshold.
- CI must not run long P66 campaigns.

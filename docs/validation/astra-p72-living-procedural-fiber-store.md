# ASTRA-P72 — Living Procedural Fiber Store Validation

P72 tests ASTRA as a living procedural fiber store. The store is not a virtual
disk that persists every generated fiber. It persists a cold procedural state
that is sufficient to regenerate useful fibers at runtime, while runtime cache
and materialized views may be discarded if close/reopen preserves the same
observable logical state.

## Scope

P72 reuses the P71 real-data corpora and hard `10 MiB` budget, then adds a
lifecycle:

- encode;
- open;
- read and query;
- update and delete;
- audit;
- compact;
- close;
- reopen;
- replay journal;
- verify logical equivalence.

## Lifecycle `.atlas`

The specialized P72 lifecycle syntax is intentionally declarative:

```atlas
atlas version=0.1;
p72_lifecycle id=p72_living_fiber_store architecture=address_fiber_actor_managed_v1;
lifecycle name=living_store persistence=cold_manifest runtime=materialize_on_read close=checkpoint_delta reopen=replay_journal compaction=threshold;
storage_form name=procedural_fiber_store generator=accounted parameters=accounted dictionary=accounted residuals=accounted journal=accounted cache=runtime_only actor_state=checkpointed_minimal audit_metadata=accounted checksums=accounted checkpoint=accounted;
lifecycle_gates reopen_equivalence=true all_persistent_storage_counted=true runtime_cache_not_required_for_correctness=true journal_replay_bounded=true guard_no_false_gain=true;
```

No general-purpose `.atlas` execution is added.

## CLI

```bash
cargo run -p atlas-cli -- living-store-bench \
  --corpus all \
  --budget-bytes 10485760 \
  --runs 30 \
  --queries 1000 \
  --updates 100 \
  --deletes 20 \
  --compact threshold \
  --adaptive on \
  --export-dir artifacts/p72/living_store_standard \
  --format json
```

## Exports

Generated artifacts stay under `artifacts/p72/` and are ignored by Git:

- `p72_living_report.json`
- `p72_summary.md`
- `p72_cost_breakdown.csv`
- `living_store/cold/`
- `living_store/runtime/`
- `living_store/reports/`

The `cold/` tree is the measured persistent cost. The `runtime/` tree is the
measured working set and must not be required for correctness after reopen.

## Metrics

P72 reports:

- `cold_persisted_bytes`
- `runtime_peak_bytes`
- `exact_recoverable_bytes`
- `ratio_persistent`
- `ratio_runtime`
- `ratio_living`
- `reopen_equivalence`
- `journal_replay_steps`
- `compaction_savings_bytes`
- `adaptive_rewrite_count`
- `guard_decision`
- `declared_vs_cold_delta_percent`
- `drift_status`

## Decision policy

Possible P72 decisions are:

- `VALIDATE_P72_LIVING_PROCEDURAL_STORE`
- `RECALIBRATE_P72_LIVING_COST_MODEL`
- `RECALIBRATE_P72_ADAPTIVE_ENCODING`
- `NO_GO_P72_LIVING_STORE`

The default decision remains conservative. P72 may only validate if reopen
equivalence, roundtrip, retrieval, journal replay, compaction, guard behavior,
invalid corpus refusal, filesystem cost accounting, and drift checks all pass.

## Local validation

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p72_tests
cargo test --test p71_tests
bash scripts/validate_p58_local.sh
```

The CI remains sanity-only and does not run living store campaigns.

## P72 local standard result

The local P72 standard campaign on 2026-05-01 produced:

- `cold_persisted_bytes = 332,405`
- `runtime_peak_bytes = 164,734`
- `exact_recoverable_bytes = 1,357,914`
- `ratio_persistent = 4.085119`
- `ratio_runtime = 8.243071`
- `ratio_living = 2.366879`
- `reopen_equivalence = true`
- `journal_replay_steps = 809`
- `compaction_savings_bytes = 27,483`
- `adaptive_rewrite_count = 3`
- `guard_decision = NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`
- `drift_status = HARD_DRIFT`
- `decision = RECALIBRATE_P72_LIVING_COST_MODEL`

The lifecycle is functional, but the living cost model remains conservative
because declared persistent bytes and measured cold persisted bytes still differ
by `78.359482%`.

# ASTRA-P65 — Address-Local Actor Runtime Analysis

## 1. Executive summary

ASTRA-P65 tests a stronger hypothesis after P64: the ratio does not come only
from local generation, but from the pair:

```text
address-local generation + budgeted local actor
```

A local actor is a counted runtime structure. It owns a neighborhood
`N(A, r)`, local cache, local index, journal, queue, audit and compaction
state. It is useful only if it amortizes more cost than it adds.

P65 adds `ratio-actors`, four deterministic actor strategies, compact exports,
strategy comparisons and tests. The local campaigns show that
`single_local_actor` improves the measured ratio on all four realish workloads,
while specialized CRUD actors and over-agentic stress add too much overhead.

Scientific decision remains conservative:

```text
RECALIBRATE_P65_ACTOR_OVERHEAD
```

## 2. Position after P64

P64 introduced realish workloads and compared:

- `full_materialization`;
- `global_indexed_generation`;
- `address_local_generation`.

P64 standard result:

| metric | value |
|---|---:|
| virtual_declared | 132,000,000 |
| virtual_generated_local | 2,296,000 |
| virtual_effective | 22,600,000 |
| real persisted bytes | 14,997,283 |
| ratio_effective_per_byte | 1.506940 |
| effective_gain_vs_materialized | 12.055517 |
| locality_selectivity | 0.017394 |
| decision | RECALIBRATE_P64_ADDRESS_LOCAL_RATIO_MODEL |

P64 selected `address_local_generation` as the best observed policy on all
four workloads, but showed that cost rises quickly when query pressure grows.

## 3. Central hypothesis: address-local actors

P65 tests:

```text
LocalActor(A, r) =
  neighborhood N(A, r)
  + cache local
  + index local
  + journal local
  + queue d'actions
  + audit local
  + budget bytes/temps/actions
  + règles de refus
```

The hypothesis is:

> Un acteur local est une mémoire comptée et limitée. Il n'est utile que s'il
> amortit plus de coût qu'il n'en ajoute.

P65 keeps the actors deterministic. It does not add LLM agents or autonomous
runtime intelligence.

## 4. Actor model

Each `LocalActor` reports:

- actor id and anchor address;
- neighborhood radius and assigned workload;
- byte budget and action budget;
- cache, journal, audit and compaction flags;
- state/cache/index/journal/queue/audit/coordination bytes;
- total actor overhead bytes;
- read/update/delete/audit/cache/evict/compact counts;
- conflict, stale read and budget refusal counts.

All actor state is real cost. All coordination is system cost.

## 5. Actor strategies

| strategy | role |
|---|---|
| `no_actor_address_local` | P64-style address-local baseline without persistent actor state |
| `single_local_actor` | one budgeted local actor per active neighborhood group |
| `specialized_crud_actors` | read/update/delete/audit/cache/compaction split with explicit coordination |
| `over_agentic_stress` | deliberately excessive actors and coordination to expose degradation |

## 6. Workloads reused from P64

P65 reuses the four P64 realish fixtures:

- `realish_log_events`;
- `realish_sparse_csv`;
- `realish_json_records`;
- `realish_hybrid_field_fixture`.

No external dataset is introduced in this prompt.

## 7. Metrics

P65 preserves the P64 virtual/real metrics and adds actor overhead metrics:

- `actor_count`;
- `actor_state_bytes`;
- `actor_cache_bytes`;
- `actor_index_bytes`;
- `actor_journal_bytes`;
- `actor_queue_bytes`;
- `actor_audit_bytes`;
- `actor_coordination_bytes`;
- `total_actor_overhead_bytes`;
- `actor_overhead_ratio`;
- `cache_hit_rate`;
- `actor_action_count`;
- `coordination_events`;
- `stale_read_count`;
- `conflict_count`;
- `eviction_count`;
- `compaction_count`;
- `budget_refusal_count`;
- `actor_net_gain`;
- `actor_ratio_delta`;
- `actor_bytes_delta`.

Timing observations are local and are not goldenized.

## 8. Local validation commands

Executed locally in the ASTRA isolated Rust environment:

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

Validation status:

- `cargo fmt --all -- --check`: PASS
- `cargo build --workspace`: PASS
- `cargo test --workspace`: PASS
- `cargo test --test p64_tests`: PASS
- `tests/p65_tests.rs`: PASS, 11 tests in workspace run
- `scripts/validate_p58_local.sh`: PASS, invalid corpus refused 21/21

Campaign durations observed with `/usr/bin/time -p`:

- standard actor campaign: `real 0.29s`
- ambitious actor campaign: `real 0.30s`

These are local runtime observations only.

## 9. Standard campaign results

Standard campaign parameters:

- mode: `standard`
- runs: `30`
- query_count: `1000`
- neighborhood_radius: `3`
- budget_bytes: `1,048,576`
- strategy entries: `16`
- strategy comparisons: `4`
- decision: `RECALIBRATE_P65_ACTOR_OVERHEAD`

Aggregate by actor strategy:

| strategy | real_bytes | actor_overhead_bytes | actor_overhead_ratio | ratio_effective_per_byte | effective_gain | cache_hit_rate | conflicts | stale_reads |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| no_actor_address_local | 14,997,283 | 0 | 0.000000 | 1.506940 | 12.055517 | 0.000000 | 0 | 0 |
| single_local_actor | 11,289,541 | 2,400,069 | 0.212592 | 2.001853 | 16.014823 | 0.473767 | 0 | 0 |
| specialized_crud_actors | 19,854,454 | 11,159,320 | 0.562056 | 1.138284 | 9.106269 | 0.393767 | 4 | 0 |
| over_agentic_stress | 100,129,000 | 79,783,244 | 0.796805 | 0.225709 | 1.805671 | 0.166867 | 4,040 | 2,020 |

Per-workload best strategy:

| workload | best strategy | decision | baseline ratio | single actor ratio | specialized ratio | over-agentic ratio | actor_net_gain | overhead_ratio | cache_hit_rate | conflicts | stale_reads |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| realish_log_events | single_local_actor | LOCAL_ACTOR_STRONG | 1.193000 | 1.502703 | 0.789472 | 0.149189 | 1.259600 | 0.249682 | 0.473767 | 0 | 0 |
| realish_sparse_csv | single_local_actor | LOCAL_ACTOR_STRONG | 1.948657 | 2.579034 | 1.458171 | 0.288269 | 1.323493 | 0.215200 | 0.473767 | 0 | 0 |
| realish_json_records | single_local_actor | LOCAL_ACTOR_STRONG | 0.678860 | 0.871442 | 0.470236 | 0.090192 | 1.283684 | 0.236763 | 0.473767 | 0 | 0 |
| realish_hybrid_field_fixture | single_local_actor | LOCAL_ACTOR_STRONG | 1.903213 | 2.683391 | 1.690660 | 0.360504 | 1.409927 | 0.168665 | 0.473767 | 0 | 0 |

## 10. Ambitious campaign results if executed

Ambitious campaign parameters:

- mode: `ambitious`
- runs: `50`
- query_count: `5000`
- neighborhood_radius: `5`
- budget_bytes: `4,194,304`
- strategy entries: `16`
- strategy comparisons: `4`
- decision: `RECALIBRATE_P65_ACTOR_OVERHEAD`

Aggregate by actor strategy:

| strategy | real_bytes | actor_overhead_bytes | actor_overhead_ratio | ratio_effective_per_byte | effective_gain | cache_hit_rate | conflicts | stale_reads |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| no_actor_address_local | 119,212,435 | 0 | 0.000000 | 0.189578 | 1.516620 | 0.000000 | 0 | 0 |
| single_local_actor | 89,887,897 | 19,276,561 | 0.214451 | 0.251424 | 2.011394 | 0.536252 | 0 | 0 |
| specialized_crud_actors | 154,377,322 | 85,213,260 | 0.551980 | 0.146395 | 1.171156 | 0.456252 | 36 | 16 |
| over_agentic_stress | 648,451,568 | 485,931,244 | 0.749372 | 0.034852 | 0.278818 | 0.198124 | 20,200 | 10,100 |

Per-workload best strategy:

| workload | best strategy | decision | baseline ratio | single actor ratio | specialized ratio | over-agentic ratio | actor_net_gain | overhead_ratio | cache_hit_rate | conflicts | stale_reads |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| realish_log_events | single_local_actor | LOCAL_ACTOR_STRONG | 0.149664 | 0.188291 | 0.101629 | 0.023222 | 1.258094 | 0.251292 | 0.536252 | 0 | 0 |
| realish_sparse_csv | single_local_actor | LOCAL_ACTOR_STRONG | 0.245105 | 0.323869 | 0.187546 | 0.044533 | 1.321347 | 0.217044 | 0.536252 | 0 | 0 |
| realish_json_records | single_local_actor | LOCAL_ACTOR_STRONG | 0.085248 | 0.109280 | 0.060516 | 0.014005 | 1.281899 | 0.238482 | 0.536252 | 0 | 0 |
| realish_hybrid_field_fixture | single_local_actor | LOCAL_ACTOR_STRONG | 0.240142 | 0.337963 | 0.217148 | 0.054963 | 1.407346 | 0.170593 | 0.536252 | 0 | 0 |

## 11. Strategy comparison

`single_local_actor` is the best observed strategy on all four workloads in
both standard and ambitious campaigns.

`specialized_crud_actors` adds enough coordination and actor state to lose
against the baseline in aggregate. `over_agentic_stress` demonstrates the
failure mode: too much state, too much coordination, many conflicts and stale
reads.

## 12. Actor overhead analysis

The standard single actor campaign improves aggregate ratio from `1.506940` to
`2.001853` with actor overhead ratio `0.212592`. That is a useful signal, but
still not a final validation.

The ambitious single actor campaign improves aggregate ratio from `0.189578`
to `0.251424` with actor overhead ratio `0.214451`. The actor still helps under
higher pressure, but the absolute ratio remains much lower than standard
because more local space and journal work is touched.

Specialized actors exceed `0.55` overhead ratio in both campaigns. Over-agentic
stress exceeds `0.74` overhead ratio and creates explicit conflicts/stale reads.

## 13. Cache/journal/audit analysis

Single local actors use cache and journal state to reduce repeated local cost:

- standard cache_hit_rate: `0.473767`;
- ambitious cache_hit_rate: `0.536252`;
- conflicts: `0`;
- stale reads: `0`.

Specialized CRUD actors retain cache benefits but pay coordination overhead and
produce a small conflict/stale-read surface. Over-agentic stress intentionally
shows that actor decomposition can become harmful.

## 14. Ratio/gain view

```text
Address-local actor view
baseline no actor ratio        : 1.506940 standard / 0.189578 ambitious
single local actor ratio       : 2.001853 standard / 0.251424 ambitious
specialized CRUD actors ratio  : 1.138284 standard / 0.146395 ambitious
actor overhead bytes           : 2,400,069 standard / 19,276,561 ambitious
actor net gain                 : 1.259600-1.409927 by workload standard
cache hit rate                 : 0.473767 standard / 0.536252 ambitious
conflicts / stale reads        : 0 / 0 for single local actor
best actor strategy            : single_local_actor
decision                       : RECALIBRATE_P65_ACTOR_OVERHEAD
```

## 15. Decisions

Observed implementation direction:

```text
CANDIDATE_P66_SINGLE_LOCAL_ACTOR
```

Scientific P65 decision:

```text
RECALIBRATE_P65_ACTOR_OVERHEAD
```

P65 does not return `PROMOTE_P65_LOCAL_ACTORS` because:

- workloads remain deterministic internal fixtures;
- only one local machine is used;
- actor thresholds are not calibrated;
- no external workload validates the cache/journal pattern;
- specialized actors and over-agentic stress show real degradation zones.

## 16. Limitations

- No external dataset.
- No multi-machine run.
- No calibrated actor overhead threshold.
- LocalActor is deterministic runtime accounting, not autonomous intelligence.
- Timings are local and not goldenized.
- No long-lived persisted actor store is implemented yet.
- Coordination cost is structural and deterministic in this prompt.

## 17. Recommendation for P66

P66 should focus on the single local actor path:

- calibrate actor overhead thresholds;
- add a true actor registry/campaign set if multiple actor campaigns are run;
- test locality clusters and repeated address reuse more directly;
- strengthen update/delete/audit semantics;
- add a persisted actor cache fixture without bloating the repo;
- keep over-agentic stress as a guardrail, not a target design.

## 18. Reproducibility notes

Generated campaign outputs are local artifacts:

```text
artifacts/p65/actors_standard/
artifacts/p65/actors_ambitious/
```

They are ignored by Git. The durable repo artifact is this Markdown analysis.

## 19. Journal

- P65-1: added `src/p65.rs`, `ratio-actors`, actor strategies, budgeted
  LocalActor metrics, compact exports, tests, docs and local standard/ambitious
  campaigns.
- Local validation: `cargo fmt`, `cargo build`, `cargo test`, targeted
  `p64_tests`, and `validate_p58_local.sh` passed.
- Decision remains `RECALIBRATE_P65_ACTOR_OVERHEAD`.

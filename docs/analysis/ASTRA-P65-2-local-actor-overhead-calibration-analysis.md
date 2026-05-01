# ASTRA-P65-2 — Local Actor Overhead Calibration Analysis

## 1. Executive summary

ASTRA-P65-2 calibrates the overhead of `single_local_actor`, the best observed
strategy from P65. The central question is whether the local actor can keep the
ratio gain while lowering actor overhead below a candidate target of 15% without
conflicts, stale reads or dominant budget refusals.

P65-2 adds a deterministic calibration grid:

- neighborhood radius;
- actor byte budget;
- cache on/off;
- journal policy `lazy|compact`;
- query locality `clustered|random|mixed`.

The local campaigns found promising candidate zones, especially:

```text
radius=5, cache=on, journal=compact, query_locality=clustered
```

However the scientific decision remains conservative:

```text
RECALIBRATE_P65_ACTOR_OVERHEAD
```

PROMOTE_P66_LOCAL_ACTOR_ARCHITECTURE is not returned in this prompt.

## 2. Position after P65

P65 established that deterministic local actors can improve the P64
address-local ratio, but actor state remains a real cost.

P65 standard result:

| metric | value |
|---|---:|
| baseline no actor ratio | 1.506940 |
| single local actor ratio | 2.001853 |
| relative gain | +32.84% |
| actor_overhead_bytes | 2,400,069 |
| actor_overhead_ratio | 0.212592 |
| cache_hit_rate | 0.473767 |
| conflicts | 0 |
| stale_reads | 0 |
| decision | RECALIBRATE_P65_ACTOR_OVERHEAD |

P65 ambitious result:

| metric | value |
|---|---:|
| baseline no actor ratio | 0.189578 |
| single local actor ratio | 0.251424 |
| relative gain | +32.62% |
| actor_overhead_bytes | 19,276,561 |
| actor_overhead_ratio | 0.214451 |
| cache_hit_rate | 0.536252 |
| conflicts | 0 |
| stale_reads | 0 |
| decision | RECALIBRATE_P65_ACTOR_OVERHEAD |

## 3. Calibration hypothesis

Hypothesis:

```text
single_local_actor can become a P66 candidate if actor_net_gain stays above 1.20
while actor_overhead_ratio falls below 0.15, with no conflicts and no stale reads.
```

Candidate thresholds used as experimental filters:

| threshold | value |
|---|---:|
| candidate_actor_overhead_ratio_target | 0.15 |
| candidate_min_actor_net_gain | 1.20 |
| candidate_min_cache_hit_rate | 0.45 |
| candidate_max_conflicts | 0 |
| candidate_max_stale_reads | 0 |
| candidate_max_budget_refusal_rate | 0.10 |

These thresholds are not a scientific validation profile. They are promotion
candidates for the next architecture decision.

## 4. Parameters tested

Implemented calibration dimensions:

| parameter | values |
|---|---|
| `neighborhood_radius` | standard: `1,2,3,5`; ambitious: `2,3,5` |
| `budget_bytes` | standard: `262144,524288,1048576,2097152`; ambitious: `524288,1048576,2097152,4194304` |
| `cache_policy` | `off,on` |
| `journal_policy` | `lazy,compact` |
| `query_locality` | standard: `clustered,random,mixed`; ambitious: `clustered,mixed` |

Not yet implemented as independent calibration grids:

- `compaction_policy`: reported as `not_available`;
- `update_rate`: inherited from workload model;
- `audit_rate`: inherited from workload model.

## 5. Calibration grid

Standard campaign grid:

| dimension | count |
|---|---:|
| workloads | 4 |
| radii | 4 |
| budgets | 4 |
| cache policies | 2 |
| journal policies | 2 |
| query locality profiles | 3 |
| configurations tested | 768 |

Ambitious campaign grid:

| dimension | count |
|---|---:|
| workloads | 4 |
| radii | 3 |
| budgets | 4 |
| cache policies | 2 |
| journal policies | 2 |
| query locality profiles | 2 |
| configurations tested | 384 |

## 6. Metrics

Each configuration reports:

- `ratio_effective_per_byte`;
- `effective_gain_vs_materialized`;
- `actor_net_gain`;
- `actor_overhead_ratio`;
- `actor_overhead_bytes`;
- `cache_hit_rate`;
- `conflicts`;
- `stale_reads`;
- `budget_refusal_count`;
- `budget_refusal_rate`;
- `generated_units_per_query`;
- `bytes_per_query`;
- `balanced_score`;
- `promotion_candidate`;
- structured decision.

Balanced score is an experimental heuristic:

```text
actor_net_gain
  * max(0, 1 - actor_overhead_ratio)
  * cache_hit_rate_adjustment
  * safety_factor
  * budget_factor
```

`safety_factor` is zero when conflicts or stale reads occur.

## 7. Local validation commands

Executed locally in the ASTRA isolated Rust environment:

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo fmt --all
cargo test --test p65_tests

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

Observed campaign durations with `/usr/bin/time -p`:

- standard calibration: `real 0.69s`;
- ambitious calibration: `real 0.89s`.

Final local validation status after implementation and documentation updates:

| command | status |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo build --workspace` | PASS |
| `cargo test --workspace` | PASS |
| `cargo test --test p65_tests` | PASS, 18 tests |
| `bash scripts/validate_p58_local.sh` | PASS |
| invalid corpus | PASS, 21/21 refused |
| `git diff --check` | PASS |
| `git status --ignored --short artifacts/p65` | PASS, `artifacts/p65/` ignored |

## 8. Standard calibration results

Standard campaign:

| metric | value |
|---|---:|
| configurations tested | 768 |
| pareto_front entries reported | 32 |
| rejected/no-go configs | 80 |
| promotion candidate configs | 52 |
| decision | RECALIBRATE_P65_ACTOR_OVERHEAD |

Best by ratio:

| field | value |
|---|---|
| config | `realish_hybrid_field_fixture:r1:b2097152:cacheoff:journalcompact:localityclustered` |
| actor_net_gain | 1.627783 |
| actor_overhead_ratio | 0.155122 |
| ratio_effective_per_byte | 8.172160 |
| cache_hit_rate | 0.000000 |
| conflicts / stale_reads | 0 / 0 |
| budget_refusal_count | 0 |

Best by overhead:

| field | value |
|---|---|
| config | `realish_hybrid_field_fixture:r5:b262144:cacheoff:journalcompact:localityclustered` |
| actor_net_gain | 1.686176 |
| actor_overhead_ratio | 0.059652 |
| ratio_effective_per_byte | 3.142575 |
| cache_hit_rate | 0.000000 |
| conflicts / stale_reads | 0 / 0 |
| budget_refusal_count | 0 |

Best balanced:

| field | value |
|---|---|
| config | `realish_hybrid_field_fixture:r5:b2097152:cacheon:journalcompact:localityclustered` |
| actor_net_gain | 1.641752 |
| actor_overhead_ratio | 0.084426 |
| ratio_effective_per_byte | 3.059780 |
| cache_hit_rate | 0.706238 |
| conflicts / stale_reads | 0 / 0 |
| budget_refusal_count | 0 |

## 9. Ambitious calibration results if executed

Ambitious campaign:

| metric | value |
|---|---:|
| configurations tested | 384 |
| pareto_front entries reported | 32 |
| rejected/no-go configs | 0 |
| promotion candidate configs | 12 |
| decision | RECALIBRATE_P65_ACTOR_OVERHEAD |

Best by ratio:

| field | value |
|---|---|
| config | `realish_hybrid_field_fixture:r2:b4194304:cacheoff:journalcompact:localityclustered` |
| actor_net_gain | 1.637673 |
| actor_overhead_ratio | 0.152918 |
| ratio_effective_per_byte | 0.992492 |
| cache_hit_rate | 0.000000 |
| conflicts / stale_reads | 0 / 0 |
| budget_refusal_count | 0 |

Best by overhead:

| field | value |
|---|---|
| config | `realish_hybrid_field_fixture:r5:b524288:cacheoff:journalcompact:localityclustered` |
| actor_net_gain | 1.673530 |
| actor_overhead_ratio | 0.088455 |
| ratio_effective_per_byte | 0.574104 |
| cache_hit_rate | 0.000000 |
| conflicts / stale_reads | 0 / 0 |
| budget_refusal_count | 2 |

Best balanced:

| field | value |
|---|---|
| config | `realish_hybrid_field_fixture:r5:b4194304:cacheon:journalcompact:localityclustered` |
| actor_net_gain | 1.612142 |
| actor_overhead_ratio | 0.121892 |
| ratio_effective_per_byte | 0.553045 |
| cache_hit_rate | 0.706251 |
| conflicts / stale_reads | 0 / 0 |
| budget_refusal_count | 0 |

## 10. Pareto front

Both campaigns produced non-empty Pareto fronts. The reported front is capped at
32 configurations to keep JSON compact.

The most useful pattern is:

```text
workload: realish_hybrid_field_fixture
radius: 5
cache_policy: on
journal_policy: compact
query_locality: clustered
```

Interpretation:

- larger radius improves amortization in this deterministic fixture;
- compact journal policy lowers actor overhead;
- clustered locality improves cache reuse;
- cache-on balanced configurations are more useful than cache-off ratio-only
  configurations, even when cache-off can produce low overhead.

## 11. Best configurations

Local actor calibration view:

```text
best balanced config        : realish_hybrid_field_fixture:r5:b2097152:cacheon:journalcompact:localityclustered
radius                      : 5
budget_bytes                : 2,097,152
cache_policy                : on
journal_policy              : compact
query_locality              : clustered
actor_net_gain              : 1.641752
actor_overhead_ratio        : 0.084426
ratio_effective_per_byte    : 3.059780
cache_hit_rate              : 0.706238
conflicts / stale reads     : 0 / 0
decision                    : RECALIBRATE_P65_ACTOR_OVERHEAD
```

Ambitious confirmation view:

```text
best balanced config        : realish_hybrid_field_fixture:r5:b4194304:cacheon:journalcompact:localityclustered
radius                      : 5
budget_bytes                : 4,194,304
cache_policy                : on
journal_policy              : compact
query_locality              : clustered
actor_net_gain              : 1.612142
actor_overhead_ratio        : 0.121892
ratio_effective_per_byte    : 0.553045
cache_hit_rate              : 0.706251
conflicts / stale reads     : 0 / 0
decision                    : RECALIBRATE_P65_ACTOR_OVERHEAD
```

## 12. Overhead analysis

The calibration succeeded in finding configurations below the 15% overhead
target while preserving actor net gain above 1.20.

However, the best ratio-only configurations are not the best architecture
candidates:

- standard best-by-ratio overhead is `0.155122`, just above the candidate
  overhead target;
- ambitious best-by-ratio overhead is `0.152918`, also above the target;
- best-balanced configurations are below the target but trade away peak ratio
  for cache usefulness and safety.

P65-2 therefore identifies a plausible parameter zone, not a final architecture
promotion.

## 13. Cache/journal/locality analysis

Observed qualitative result:

- `cache=on` improves the balanced score when locality is clustered;
- `journal=compact` is consistently preferred by best configurations;
- `query_locality=clustered` is the strongest locality profile;
- cache-off configurations can minimize bytes but lose cache reuse;
- random locality without cache can produce conflicts or stale reads and is
  rejected by the safety factor.

Compaction policy is not yet a real grid. P65-3 or P66 should make compaction
explicit instead of folding it into the journal/locality proxy.

## 14. Promotion criteria

Candidate promotion criteria:

| criterion | status |
|---|---|
| standard actor_net_gain > 1.20 | met by candidate configs |
| standard actor_overhead_ratio < 0.15 | met by candidate configs |
| standard cache_hit_rate >= 0.45 | met by best balanced |
| standard conflicts = 0 | met by best balanced |
| standard stale_reads = 0 | met by best balanced |
| ambitious actor_net_gain > 1.15 | met by best balanced |
| ambitious conflicts/stale_reads = 0 | met by best balanced |
| compaction grid implemented | not met |
| update/audit rate grids implemented | not met |
| multi-machine evidence | not met |

Because the missing criteria are material, P65-2 does not promote to P66.

## 15. Decision

Repo implementation decision:

```text
VALIDER_IMPL_P65_2_LOCAL_ACTOR_OVERHEAD_CALIBRATION
```

Scientific/runtime decision:

```text
RECALIBRATE_P65_ACTOR_OVERHEAD
```

PROMOTE_P66_LOCAL_ACTOR_ARCHITECTURE is not reached in this prompt.

## 16. Limitations

- Workloads remain deterministic internal realish fixtures.
- No external datasets are introduced.
- Single-machine local campaign only.
- No timing golden is used.
- `compaction_policy` is not independently modeled.
- `update_rate` and `audit_rate` are inherited from workload behavior.
- Promotion logic is intentionally conservative and does not aggregate standard
  plus ambitious campaigns into a formal validator yet.

## 17. Recommendation for P66

Recommended next step:

```text
P66 — Address-local actor architecture candidate
```

P66 should keep `single_local_actor` as the leading candidate, but add:

- explicit compaction policy;
- explicit update/audit rate profiles;
- paired standard/ambitious promotion gate;
- optional multi-machine repetition;
- real persistence layout stress;
- clearer separation between ratio-only optimum and architecture optimum.

If P66 is too early, a smaller P65-3 should implement compaction/update/audit
grids first.

## 18. Reproducibility notes

Final validation commands for this prompt:

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p65_tests
bash scripts/validate_p58_local.sh

git status --short
git status --ignored --short artifacts/p65
git diff --check
```

Generated campaign artifacts live under `artifacts/p65/` and are ignored by
Git. The report stores only compact synthesized values, not raw campaign dumps.

## 19. Journal

- 2026-05-01: Added `ratio-actors-calibrate` for P65-2.
- 2026-05-01: Added calibration grids for radius, budget, cache, journal and
  query locality.
- 2026-05-01: Added Pareto front, best-by-ratio, best-by-overhead and
  best-balanced selection.
- 2026-05-01: Ran standard and ambitious local calibration campaigns.
- 2026-05-01: Scientific decision remains
  `RECALIBRATE_P65_ACTOR_OVERHEAD`.

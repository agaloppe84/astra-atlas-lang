# ASTRA-P67 — Address-Fiber Overhead Calibration Analysis

## 1. Executive summary

P67 adds a local-first calibration layer for the P66 `actor_managed_fiber`
model. The calibration searches for configurations that keep
`address_fiber_net_gain` high while reducing `avg_actor_overhead_ratio` below
candidate thresholds, with conflicts, stale reads and budget refusals kept at
zero.

Local P67 found promotion-candidate configurations in both standard and
ambitious modes. The repo decision remains conservative:

```text
RECALIBRATE_P67_FIBER_OVERHEAD
```

P67 does not claim final scientific validation. It prepares a stricter P68
promotion gate.

## 2. Position after P66

P66 confirmed the address-fiber model:

```text
address point x -> local fiber F_x
runtime -> generate F_x or F_{N(x,r)}
actor -> deterministic manager of the fiber or fiber neighborhood
```

P66's best strategy was `actor_managed_fiber`, but overhead remained high:

- standard avg actor overhead ratio: `0.294461`;
- ambitious avg actor overhead ratio: `0.249963`.

P67 focuses only on reducing that overhead without hiding cache, journal, audit,
metadata or compaction costs.

## 3. Calibration hypothesis

`actor_managed_fiber` becomes a serious P68 candidate if the repo can identify a
configuration where:

- `address_fiber_net_gain > 3.0` in standard mode;
- `avg_actor_overhead_ratio < 0.15` in standard mode;
- ambitious remains coherent with `address_fiber_net_gain > 2.5`;
- conflicts = `0`;
- stale reads = `0`;
- budget refusals = `0` or very low;
- update/audit/compaction are still counted;
- no timing golden is introduced.

## 4. Parameters tested

P67-v1 tests:

- radius;
- budget bytes;
- cache policy;
- journal policy;
- audit policy;
- compaction policy;
- query locality;
- fiber projection depth.

Held constant in P67-v1:

- update rate: deterministic medium/inherited workload rate;
- metadata policy: deterministic standard.

These held dimensions are explicit limitations and candidates for P67-2/P68.

## 5. Calibration grid

Standard grid:

```text
radius-grid              : 1,2,3,5
budget-grid              : 524288,1048576,2097152,4194304
cache-grid               : on,compact
journal-grid             : lazy,compact
audit-grid               : minimal,sampled
compaction-grid          : threshold,aggressive
query-locality-grid      : clustered,mixed
fiber-projection-grid    : shallow,medium
configuration_count      : 4096
```

Ambitious grid:

```text
radius-grid              : 2,3,5
budget-grid              : 1048576,2097152,4194304
cache-grid               : on,compact
journal-grid             : compact
audit-grid               : minimal,sampled
compaction-grid          : threshold
query-locality-grid      : clustered,mixed
fiber-projection-grid    : shallow,medium
configuration_count      : 576
```

## 6. Metrics

Each configuration records:

- `fiber_ratio_effective_per_byte`;
- `address_fiber_net_gain`;
- `avg_actor_overhead_ratio`;
- `actor_overhead_bytes`;
- cache/journal/audit/metadata bytes;
- `update_count`, `audit_count`, `compaction_count`;
- `conflicts`, `stale_reads`, `budget_refusals`;
- `cache_hit_rate`;
- `bytes_per_query`;
- `generated_units_per_query`;
- `balanced_score`;
- `promotion_candidate`;
- structured decision.

The balanced score is a ranking heuristic, not a scientific law.

## 7. Local validation commands

Commands executed locally in this prompt:

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p67_tests
cargo test --test p66_tests
bash scripts/validate_p58_local.sh

cargo run -p atlas-cli -- ratio-fibers-calibrate examples/p53_strict.atlas \
  --workload all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --radius-grid 1,2,3,5 \
  --budget-grid 524288,1048576,2097152,4194304 \
  --cache-grid on,compact \
  --journal-grid lazy,compact \
  --audit-grid minimal,sampled \
  --compaction-grid threshold,aggressive \
  --query-locality-grid clustered,mixed \
  --fiber-projection-grid shallow,medium \
  --export-dir artifacts/p67/calibration_standard \
  --format json

cargo run -p atlas-cli -- ratio-fibers-calibrate examples/p53_strict.atlas \
  --workload all \
  --mode ambitious \
  --runs 50 \
  --queries 5000 \
  --radius-grid 2,3,5 \
  --budget-grid 1048576,2097152,4194304 \
  --cache-grid on,compact \
  --journal-grid compact \
  --audit-grid minimal,sampled \
  --compaction-grid threshold \
  --query-locality-grid clustered,mixed \
  --fiber-projection-grid shallow,medium \
  --export-dir artifacts/p67/calibration_ambitious \
  --format json
```

Observed campaign durations:

- standard calibration: `real 0.55s`;
- ambitious calibration: `real 0.41s`.

## 8. Standard calibration results

Standard mode tested `4096` configurations.

Best balanced configuration:

```text
config_id                    : realish_hybrid_field_fixture:r1:b4194304:cachecompact:journalcompact:auditminimal:compactaggressive:localityclustered:projectionshallow
fiber_ratio_effective/byte   : 30.068052
address_fiber_net_gain       : 17.379955
avg_actor_overhead_ratio     : 0.123446
cache_hit_rate               : 0.715000
updates/audits/compactions   : 1500 / 1200 / 31
conflicts / stale reads      : 0 / 0
budget refusals              : 0
bytes_per_query              : 251.293000
promotion_candidate          : true
decision                     : P67_PROMOTION_CANDIDATE
```

Best by overhead:

```text
config_id                    : realish_hybrid_field_fixture:r5:b524288:cacheon:journalcompact:auditminimal:compactaggressive:localitymixed:projectionmedium
fiber_ratio_effective/byte   : 4.458989
address_fiber_net_gain       : 2.577388
avg_actor_overhead_ratio     : 0.022003
cache_hit_rate               : 0.725000
budget refusals              : 0
```

## 9. Ambitious calibration results

Ambitious mode tested `576` configurations.

Best balanced configuration:

```text
config_id                    : realish_hybrid_field_fixture:r2:b4194304:cachecompact:journalcompact:auditminimal:compactthreshold:localityclustered:projectionshallow
fiber_ratio_effective/byte   : 3.229930
address_fiber_net_gain       : 13.335472
avg_actor_overhead_ratio     : 0.119345
cache_hit_rate               : 0.750000
updates/audits/compactions   : 12500 / 10000 / 39
conflicts / stale reads      : 0 / 0
budget refusals              : 0
bytes_per_query              : 467.867200
promotion_candidate          : true
decision                     : P67_PROMOTION_CANDIDATE
```

Best by ratio:

```text
config_id                    : realish_sparse_csv:r2:b4194304:cachecompact:journalcompact:auditminimal:compactthreshold:localityclustered:projectionshallow
fiber_ratio_effective/byte   : 3.263207
address_fiber_net_gain       : 13.049889
avg_actor_overhead_ratio     : 0.154992
cache_hit_rate               : 0.750000
budget refusals              : 0
```

## 10. Pareto front

Both reports produced a non-empty Pareto front:

- standard: `32` retained Pareto entries;
- ambitious: `32` retained Pareto entries.

The front favors compact cache, compact journal, minimal audit, clustered
locality and shallow/medium fiber projection. This is an R&D signal, not a final
scientific conclusion.

## 11. Best configurations

| mode | best balanced config | net gain | overhead | ratio/byte | cache hit | conflicts | stale reads | budget refusals |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| standard | `hybrid:r1:b4194304:compact/compact/minimal/aggressive/clustered/shallow` | 17.379955 | 0.123446 | 30.068052 | 0.715000 | 0 | 0 | 0 |
| ambitious | `hybrid:r2:b4194304:compact/compact/minimal/threshold/clustered/shallow` | 13.335472 | 0.119345 | 3.229930 | 0.750000 | 0 | 0 | 0 |

## 12. Overhead analysis

P67 reduces the best balanced overhead below the candidate target:

```text
standard overhead target       : < 0.15
standard observed              : 0.123446
ambitious overhead target      : < 0.18
ambitious observed             : 0.119345
```

The best-by-overhead configuration reaches lower overhead (`0.022003`) but does
not preserve enough net gain for the strict standard candidate threshold.

## 13. Cache/journal/audit/compaction analysis

Best balanced configurations both use:

- cache policy: `compact`;
- journal policy: `compact`;
- audit policy: `minimal`;
- query locality: `clustered`;
- projection depth: `shallow`.

Compaction differs:

- standard: `aggressive`, 31 compactions;
- ambitious: `threshold`, 39 compactions.

All update/audit/compaction counts remain explicit and non-zero. No conflicts or
stale reads were observed in the best balanced configurations.

## 14. Promotion criteria

P67-v1 found configurations that satisfy the numerical candidate checks in both
standard and ambitious reports. However, the implementation deliberately keeps
the report-level decision conservative because:

- standard and ambitious are emitted as separate local reports;
- no external fixture is included;
- no multi-machine replay is included;
- update rate and metadata policy are not yet grid dimensions;
- the promotion gate should be formalized as a paired standard+ambitious check
  in P68.

## 15. Decision

Repo implementation decision:

```text
VALIDER_IMPL_P67_ADDRESS_FIBER_OVERHEAD_CALIBRATION
```

Scientific/runtime decision:

```text
RECALIBRATE_P67_FIBER_OVERHEAD
```

Promotion candidate:

```text
yes
```

P67 identifies a strong candidate zone, but does not return
`PROMOTE_P67_ADDRESS_FIBER_ARCHITECTURE`.

## 16. Limitations

- Workloads remain deterministic internal realish fixtures.
- No external dataset is included.
- No multi-machine evidence is included.
- Update rate and metadata policy are fixed in P67-v1.
- The high ratios are calibration outputs, not final scientific claims.
- No timing golden is introduced.

## 17. Recommendation for P68

Recommended next step:

```text
P68 — Formal paired promotion gate for address-fiber actors
```

P68 should:

- combine standard and ambitious reports in one promotion evaluator;
- make update_rate and metadata_policy explicit grid dimensions;
- add a replay protocol for a second local machine;
- preserve no timing goldens;
- keep all fiber/actor overhead counted;
- only then consider `PROMOTE_P67_ADDRESS_FIBER_ARCHITECTURE` or its P68
  successor decision.

## 18. Reproducibility notes

Generated artifacts live under `artifacts/p67/` and are ignored by Git.
The versioned report stores synthesized values only, not raw campaign dumps.

Results LaTeX/PDF are generated locally with:

```bash
bash scripts/build_report.sh reports/P67/RPA_ASTRA-P67-Results_address-fiber-overhead-calibration_v1.0_2026-05-01.tex
```

## 19. Journal

- 2026-05-01: Added `p67_address_fiber_overhead_calibration_v1`.
- 2026-05-01: Added `ratio-fibers-calibrate`.
- 2026-05-01: Added P67 tests, exports and docs.
- 2026-05-01: Ran standard and ambitious local P67 calibrations.
- 2026-05-01: Generated P67 Results LaTeX/PDF after local validation.
- 2026-05-01: Scientific decision remains `RECALIBRATE_P67_FIBER_OVERHEAD`.

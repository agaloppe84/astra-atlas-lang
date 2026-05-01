# ASTRA-P66 — Address-Fiber Local Actor Runtime Analysis

## 1. Executive summary

ASTRA-P66 formalizes the address-fiber model:

```text
x in Omega_virtual
F_x = local fiber attached to address x
Eval(c, x) = controlled generation of F_x or F_{N(x,r)}
```

P66 adds `AddressFiber`, fiber generation strategies, fiber-level CRUD/audit/
compaction accounting, compact exports and the `ratio-fibers` CLI.

Local standard and ambitious campaigns show that `actor_managed_fiber` is the
best observed fiber strategy across all four realish workloads. It improves the
ratio versus the address-local baseline, but actor overhead remains above the
candidate 15% threshold.

Scientific decision:

```text
RECALIBRATE_P66_ADDRESS_FIBER_MODEL
```

## 2. Position after P65-2

P65-2 calibrated `single_local_actor` overhead and found a promising zone:

| campaign | best balanced config | actor_net_gain | actor_overhead_ratio | cache_hit_rate |
|---|---|---:|---:|---:|
| standard | `realish_hybrid_field_fixture:r5:b2097152:cacheon:journalcompact:localityclustered` | 1.641752 | 0.084426 | 0.706238 |
| ambitious | `realish_hybrid_field_fixture:r5:b4194304:cacheon:journalcompact:localityclustered` | 1.612142 | 0.121892 | 0.706251 |

P65-2 stayed conservative because compaction/update/audit were not yet full
calibration dimensions. P66 moves the model from local neighborhood actor to
address point plus local fiber.

## 3. Central hypothesis: address point plus local fiber

P66 tests:

```text
address point + local fiber + optional local actor binding
```

The conceptual hypothesis:

> L'adresse est un point dans l'espace virtuel. La donnee utile est une fibre
> locale au-dessus de ce point. Le runtime doit generer la fibre utile, ou un
> voisinage fibre, sans materialiser l'espace global.

## 4. AddressFiber model

`AddressFiber` exposes:

- `address_id`;
- `base_coordinate`;
- `fiber_kind`;
- declared/reachable/readable/updatable/safe/effective/generated units;
- payload/index/cache/journal/audit/metadata bytes;
- optional actor binding;
- safety status;
- decision reasons.

All fiber and actor bytes are counted. The fiber is not a free semantic object.

## 5. Fiber generation strategies

| strategy | interpretation |
|---|---|
| `point_fiber_only` | generate only `F_x` |
| `neighborhood_fiber` | generate `F_{N(x,r)}` without persistent actor |
| `actor_managed_fiber` | a deterministic actor manages `F_x` |
| `actor_managed_neighborhood_fiber` | a deterministic actor manages `F_{N(x,r)}` |

## 6. Workloads as fibers

| workload | address | fiber |
|---|---|---|
| `realish_log_events` | timestamp bucket / service / request id | `log_event_fiber` |
| `realish_sparse_csv` | row id / column group | `sparse_row_fiber` |
| `realish_json_records` | record id / projection path | `json_record_fiber` |
| `realish_hybrid_field_fixture` | point or tile | `hybrid_field_tile_fiber` |

## 7. Metrics

P66 preserves P64/P65 metrics and adds fiber-specific metrics:

- `base_address_count`;
- `fiber_count`;
- `fiber_declared_units`;
- `fiber_generated_units`;
- `fiber_effective_units`;
- `fiber_selectivity`;
- `fiber_effective_ratio`;
- `fiber_payload_bytes`;
- `fiber_index_bytes`;
- `fiber_cache_bytes`;
- `fiber_journal_bytes`;
- `fiber_audit_bytes`;
- `fiber_metadata_bytes`;
- `fiber_actor_bytes`;
- `fiber_total_bytes`;
- `fiber_ratio_effective_per_byte`;
- `fiber_gain_vs_materialized`;
- `fiber_update_success_rate`;
- `fiber_audit_success_rate`;
- `fiber_compaction_count`;
- `fiber_eviction_count`;
- `address_fiber_net_gain`.

## 8. Local validation commands

Executed locally:

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo fmt --all
cargo test --test p66_tests

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

Observed campaign durations:

- standard: `real 0.28s`;
- ambitious: `real 0.31s`.

Full validation status is recorded in section 20 from the local commands that
were actually executed in this prompt.

## 9. Standard campaign results

Standard campaign parameters:

- runs: `30`;
- queries: `1000`;
- neighborhood_radius: `3`;
- budget_bytes: `2,097,152`;
- cache: `on`;
- journal: `compact`;
- entries: `16`;
- decision: `RECALIBRATE_P66_ADDRESS_FIBER_MODEL`.

Best strategy by workload:

| workload | best strategy | ratio | fiber_selectivity | actor_overhead_ratio | net_gain | generated | effective | bytes | update | audit | compact | decision |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `realish_log_events` | `actor_managed_fiber` | 3.967310 | 0.001500 | 0.302117 | 3.602743 | 18,000 | 3,168,000 | 798,526 | 3,000 | 600 | 126 | `FIBER_OVERHEAD_TOO_HIGH` |
| `realish_sparse_csv` | `actor_managed_fiber` | 7.568229 | 0.000750 | 0.291075 | 4.237254 | 36,000 | 6,336,000 | 837,184 | 5,400 | 900 | 126 | `FIBER_OVERHEAD_TOO_HIGH` |
| `realish_json_records` | `actor_managed_fiber` | 2.436762 | 0.002000 | 0.303644 | 4.085486 | 16,000 | 1,936,000 | 794,497 | 3,600 | 600 | 126 | `FIBER_OVERHEAD_TOO_HIGH` |
| `realish_hybrid_field_fixture` | `actor_managed_fiber` | 9.645586 | 0.000844 | 0.281008 | 5.575348 | 54,000 | 8,448,000 | 875,841 | 1,500 | 1,200 | 126 | `FIBER_OVERHEAD_TOO_HIGH` |

Aggregate best-strategy view:

| metric | value |
|---|---:|
| fiber_generated_units | 124,000 |
| fiber_effective_units | 19,888,000 |
| total_persisted_bytes | 3,306,048 |
| total_ratio_effective_per_byte | 6.015642 |
| average actor_overhead_ratio | 0.294461 |
| average address_fiber_net_gain | 4.375208 |
| update_count | 13,500 |
| audit_count | 3,300 |
| compaction_count | 504 |

## 10. Ambitious campaign results if executed

Ambitious campaign parameters:

- runs: `50`;
- queries: `5000`;
- neighborhood_radius: `5`;
- budget_bytes: `4,194,304`;
- cache: `on`;
- journal: `compact`;
- entries: `16`;
- decision: `RECALIBRATE_P66_ADDRESS_FIBER_MODEL`.

Best strategy by workload:

| workload | best strategy | ratio | fiber_selectivity | actor_overhead_ratio | net_gain | generated | effective | bytes | update | audit | compact | decision |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `realish_log_events` | `actor_managed_fiber` | 0.565347 | 0.007500 | 0.253635 | 4.089019 | 90,000 | 3,168,000 | 5,603,640 | 25,000 | 5,000 | 626 | `FIBER_OVERHEAD_TOO_HIGH` |
| `realish_sparse_csv` | `actor_managed_fiber` | 1.091486 | 0.003750 | 0.248316 | 4.854740 | 180,000 | 6,336,000 | 5,804,930 | 45,000 | 7,500 | 626 | `FIBER_OVERHEAD_TOO_HIGH` |
| `realish_json_records` | `actor_managed_fiber` | 0.346736 | 0.010000 | 0.254547 | 4.623061 | 80,000 | 1,936,000 | 5,583,498 | 30,000 | 5,000 | 626 | `FIBER_OVERHEAD_TOO_HIGH` |
| `realish_hybrid_field_fixture` | `actor_managed_fiber` | 1.406542 | 0.004219 | 0.243353 | 6.439436 | 270,000 | 8,448,000 | 6,006,219 | 12,500 | 10,000 | 626 | `FIBER_OVERHEAD_TOO_HIGH` |

Aggregate best-strategy view:

| metric | value |
|---|---:|
| fiber_generated_units | 620,000 |
| fiber_effective_units | 19,888,000 |
| total_persisted_bytes | 22,998,287 |
| total_ratio_effective_per_byte | 0.864760 |
| average actor_overhead_ratio | 0.249963 |
| average address_fiber_net_gain | 5.001564 |
| update_count | 112,500 |
| audit_count | 27,500 |
| compaction_count | 2,504 |

## 11. Fiber strategy comparison

Standard comparison:

| workload | point | neighborhood | actor fiber | actor neighborhood | best |
|---|---:|---:|---:|---:|---|
| `realish_log_events` | 1.618457 | 3.037811 | 3.967310 | 1.287877 | `actor_managed_fiber` |
| `realish_sparse_csv` | 3.013378 | 5.397226 | 7.568229 | 2.446707 | `actor_managed_fiber` |
| `realish_json_records` | 0.997276 | 1.737498 | 2.436762 | 0.773780 | `actor_managed_fiber` |
| `realish_hybrid_field_fixture` | 3.758308 | 5.891441 | 9.645586 | 3.106634 | `actor_managed_fiber` |

Ambitious comparison:

| workload | point | neighborhood | actor fiber | actor neighborhood | best |
|---|---:|---:|---:|---:|---|
| `realish_log_events` | 0.206103 | 0.378588 | 0.565347 | 0.168302 | `actor_managed_fiber` |
| `realish_sparse_csv` | 0.393614 | 0.674198 | 1.091486 | 0.320031 | `actor_managed_fiber` |
| `realish_json_records` | 0.126616 | 0.216825 | 0.346736 | 0.101153 | `actor_managed_fiber` |
| `realish_hybrid_field_fixture` | 0.502170 | 0.738412 | 1.406542 | 0.406684 | `actor_managed_fiber` |

## 12. Actor-fiber overhead analysis

`actor_managed_fiber` wins on ratio across all workloads, but overhead remains
too high:

- standard overhead range for best entries: `0.281008` to `0.303644`;
- ambitious overhead range for best entries: `0.243353` to `0.254547`;
- candidate target from P65-2: `< 0.15`.

P66 therefore validates the address-fiber direction as an R&D model, but does
not promote the architecture.

## 13. CRUD/audit/compaction analysis

P66 explicitly counts update, audit and compaction:

| campaign | update_count | audit_count | compaction_count |
|---|---:|---:|---:|
| standard best entries | 13,500 | 3,300 | 504 |
| ambitious best entries | 112,500 | 27,500 | 2,504 |

Update and audit success rates are `1.0` for the best entries. Conflicts and
stale reads are `0`. The failing criterion is overhead, not safety.

## 14. Ratio/gain view

Address-fiber view:

```text
base address count              : 1000 standard / 5000 ambitious
best fiber strategy             : actor_managed_fiber
standard fiber generated units  : 124,000
standard fiber effective units  : 19,888,000
standard persisted bytes        : 3,306,048
standard ratio effective/byte   : 6.015642
standard avg actor overhead     : 0.294461
standard avg net gain           : 4.375208

ambitious fiber generated units : 620,000
ambitious fiber effective units : 19,888,000
ambitious persisted bytes       : 22,998,287
ambitious ratio effective/byte  : 0.864760
ambitious avg actor overhead    : 0.249963
ambitious avg net gain          : 5.001564
decision                        : RECALIBRATE_P66_ADDRESS_FIBER_MODEL
```

## 15. Conceptual interpretation

P66 clarifies a core ASTRA idea:

```text
The address is not the data.
The useful data is a local fiber over the address.
```

This separates addressability from materialization. It also makes actor cost
more honest: if an actor manages a fiber, its cache, journal, audit and
coordination must be counted against the ratio.

## 16. Technical/system interpretation

Implementation result:

- `src/p66.rs` adds the P66 model;
- `ratio-fibers` exposes standard and ambitious local campaigns;
- JSON/Markdown/JSONL/CSV exports are compact;
- tests cover fiber kinds, strategies, ratios, CRUD/audit/compaction and CLI;
- no timing golden is added;
- no `.atlas` grammar change is introduced.

## 17. Decision

Repo implementation decision:

```text
VALIDER_IMPL_P66_ADDRESS_FIBER_RUNTIME
```

Scientific/runtime decision:

```text
RECALIBRATE_P66_ADDRESS_FIBER_MODEL
```

`PROMOTE_P66_ADDRESS_FIBER_ARCHITECTURE` is not reached because actor overhead
is still above the candidate threshold.

## 18. Limitations

- Workloads are deterministic internal realish fixtures.
- No external dataset is included.
- No multi-machine evidence is included.
- Actor-managed fiber overhead remains above 15%.
- The first P66 model does not yet reuse the full P65-2 calibration grid.
- Timing observations are local only and are not goldenized.

## 19. Recommendation for P67

Recommended next step:

```text
P67 — Fiber actor overhead reduction and promotion gate
```

P67 should:

- reuse the P65-2 calibration machinery for `actor_managed_fiber`;
- optimize actor binding bytes/cache/journal/compaction at fiber granularity;
- add paired standard/ambitious promotion gates;
- keep safety gates conflicts/stale_reads/budget_refusals;
- optionally replay on a second machine before any scientific promotion.

## 20. Reproducibility notes

Final validation commands for this prompt:

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p65_tests
bash scripts/validate_p58_local.sh

bash scripts/build_report.sh reports/P66/RPA_ASTRA-P66-Results_address-fiber-local-actor-runtime_v1.0_2026-05-01.tex

git status --short
git status --ignored --short artifacts/p66
git diff --check
```

Generated campaign artifacts live under `artifacts/p66/` and are ignored by
Git. The report stores synthesized values, not raw campaign dumps.

Observed local validation status:

| command | status | important output |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | no formatting diff |
| `cargo build --workspace` | PASS | workspace build completed |
| `cargo test --workspace` | PASS | P66 tests included, `p66_tests.rs`: 11 passed |
| `cargo test --test p65_tests` | PASS | 18 passed |
| `bash scripts/validate_p58_local.sh` | PASS | invalid corpus checked: 21/21 refused |
| P66 standard campaign | PASS | `real 0.28s`, decision `RECALIBRATE_P66_ADDRESS_FIBER_MODEL` |
| P66 ambitious campaign | PASS | `real 0.31s`, decision `RECALIBRATE_P66_ADDRESS_FIBER_MODEL` |
| `bash scripts/build_report.sh ...P66...tex` | PASS | PDF generated, 52K |
| `git status --ignored --short artifacts/p66` | PASS | `artifacts/p66/` ignored |

PDF compilation note: the first sandboxed Tectonic call could not initialize its
default bundle/cache. The final compilation was executed through
`scripts/build_report.sh` with Tectonic outside the sandbox, generated the PDF,
and left only non-blocking overfull-box warnings on long technical identifiers.

## 21. Journal

- 2026-05-01: Added `AddressFiber`, fiber strategies and `ratio-fibers`.
- 2026-05-01: Added P66 exports and tests.
- 2026-05-01: Ran standard and ambitious local P66 campaigns.
- 2026-05-01: Generated P66 analysis report and Results LaTeX/PDF.
- 2026-05-01: Scientific decision remains
  `RECALIBRATE_P66_ADDRESS_FIBER_MODEL`.

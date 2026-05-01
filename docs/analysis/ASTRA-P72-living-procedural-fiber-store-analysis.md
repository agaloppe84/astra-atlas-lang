# ASTRA-P72 — Living Procedural Fiber Store Analysis

## 1. Executive summary

P72 introduces a living procedural fiber store. The store separates the cold
persistent ASTRA state from the hot runtime working set, then tests whether
close/reopen restores the same logical observable state after read, query,
update, delete, audit and compaction actions.

Local validation status: `PASS`.

Decision from the standard campaign: `RECALIBRATE_P72_LIVING_COST_MODEL`.

## 2. Position after P71

P71 validated the first functional filesystem Fiber Store pipeline:

- budget: `10,485,760` bytes;
- source dataset bytes: `1,332,506`;
- filesystem store bytes: `747,457`;
- exact recoverable bytes: `1,266,970`;
- useful retrieved bytes: `79,972`;
- roundtrip success rate: `1.000000`;
- retrieval success rate: `1.000000`;
- guard: `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`;
- declared vs measured drift: `HARD_DRIFT`, `324.382835%`;
- decision: `RECALIBRATE_P71_CONTRACT_COST_MODEL`.

P72 keeps the functional P71 pipeline but splits persistent cost, runtime working
cost, and reopen/replay cost.

## 3. Central hypothesis: ASTRA as living procedural storage

ASTRA does not store data as a complete global table. ASTRA stores a cold
procedural state sufficient to regenerate useful fibers at runtime. The runtime
may materialize hot fibers, cache entries, actor state, temporary indexes and
decoded views, but those hot structures may disappear if reopen restores the
same observable logical state.

## 4. Cold state vs runtime state

Cold state contains:

- contract;
- generators;
- quantized parameters;
- dictionaries;
- indexes;
- residuals;
- journal;
- checkpoints;
- manifest;
- checksums;
- audit metadata;
- safety metadata;
- actor policy.

Runtime state contains:

- materialized fibers;
- hot cache;
- actor states;
- action queues;
- temporary indexes;
- decoded views;
- runtime working bytes and peak bytes.

## 5. Lifecycle `.atlas` contract

P72 adds a specialized lifecycle syntax:

```atlas
atlas version=0.1;
p72_lifecycle id=p72_living_fiber_store architecture=address_fiber_actor_managed_v1;
lifecycle name=living_store persistence=cold_manifest runtime=materialize_on_read close=checkpoint_delta reopen=replay_journal compaction=threshold;
storage_form name=procedural_fiber_store generator=accounted parameters=accounted dictionary=accounted residuals=accounted journal=accounted cache=runtime_only actor_state=checkpointed_minimal audit_metadata=accounted checksums=accounted checkpoint=accounted;
lifecycle_gates reopen_equivalence=true all_persistent_storage_counted=true runtime_cache_not_required_for_correctness=true journal_replay_bounded=true guard_no_false_gain=true;
```

The extension remains declarative and specialized. It does not add loops,
functions, or general-purpose execution to `.atlas`.

## 6. LivingFiberStore model

`LivingFiberStore` is represented by the P72 report model:

- lifecycle contract;
- P71 seed store;
- measured cold directory;
- measured runtime directory;
- reopen equivalence report;
- journal replay report;
- living compaction report;
- adaptive encoding report;
- living cost breakdown;
- decision reasons.

## 7. Real data corpora

P72 reuses the P71 corpora:

| corpus | role |
|---|---|
| `real_code_corpus` | repository code and docs, exact recovery required |
| `realish_logs_corpus` | deterministic structured logs |
| `realish_json_records` | deterministic nested records |
| `sparse_csv_table` | deterministic sparse table |
| `incompressible_guard_blob` | guard against false gain |

## 8. Encoding policies

P72 reuses the P71 policies and adds the living adaptive phase:

- `raw_fiber`;
- `dictionary_fiber`;
- `template_delta_fiber`;
- `generated_plus_residual_fiber`;
- `adaptive_living_fiber`;
- `refused_fiber`.

The adaptive phase is conservative: it can rewrite a fiber representation only
when exactness and reopen equivalence are preserved.

## 9. Runtime actions

The standard P72 campaign exercises:

- read;
- query;
- update;
- delete;
- audit;
- compact;
- close;
- reopen.

The runtime cache is explicitly not required for correctness.

## 10. Close/reopen equivalence

Observable equivalence checks:

- logical state hash before close;
- logical state hash after reopen;
- reopened read success rate;
- reopened query success rate;
- reopened roundtrip success rate;
- journal replay success;
- tombstone/update visibility.

## 11. Compaction and adaptive encoding

Compaction writes a checkpointed cold state and verifies that the logical state
hash remains preserved. Adaptive encoding reports the number of rewrites and
whether exactness, guard behavior and reopen equivalence are preserved.

## 12. Filesystem cost breakdown

P72 distinguishes:

- `cold_persisted_bytes`: persistent cost of the ASTRA file/store;
- `runtime_peak_bytes`: hot working set cost while the store is open;
- reopen/replay cost: bounded journal replay and regenerated runtime bytes.

Reports and summaries are kept outside the useful storage denominator.

## 13. Persistent/runtime/reopen ratios

P72 reports:

- `ratio_persistent = exact_recoverable_bytes / cold_persisted_bytes`;
- `ratio_runtime = exact_recoverable_bytes / runtime_peak_bytes`;
- `ratio_living = exact_recoverable_bytes / (cold_persisted_bytes + runtime_peak_bytes + reopen_runtime_bytes)`;
- `useful_retrieved_bytes_per_persistent_byte`;
- `procedural_store_gain_vs_raw`;
- `living_gain_vs_raw`.

## 14. Guard incompressible

The incompressible guard remains a no-false-gain check. It must be refused,
stored as explicit raw fallback, or classified no-go. It must not inflate the
reported procedural ratio.

## 15. Declared vs measured drift

P72 recalibrates the P71 drift by comparing declared persistent contract bytes
with measured cold persisted bytes. Runtime working bytes are reported
separately instead of being hidden inside persistent storage.

## 16. Test stack hygiene P72

P72 adds `tests/p72_tests.rs` and updates `tests/atlas_tests.rs` to track six
new invalid lifecycle examples. No useful test was deleted.

See `docs/analysis/ASTRA-P72-test-stack-audit.md`.

## 17. Local validation commands

Commands executed locally:

```bash
git pull --ff-only
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p72_tests
cargo test --test p71_tests
bash scripts/validate_p58_local.sh
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
bash scripts/build_report.sh reports/P72/RPA_ASTRA-P72-Results_living-procedural-fiber-store_v1.0_2026-05-01.tex
```

Observed local command status before Results compilation:

- `git pull --ff-only`: already up to date.
- `cargo fmt --all -- --check`: PASS.
- `cargo build --workspace`: PASS.
- `cargo test --workspace`: PASS.
- `cargo test --test p72_tests`: PASS, `13 passed`.
- `cargo test --test p71_tests`: PASS, `14 passed`.
- `bash scripts/validate_p58_local.sh`: PASS, `44` invalid examples refused.
- P72 invalid loop: `6/6` invalid lifecycle programs refused.
- P72 standard campaign: PASS, observed `real 0.35s`.
- Results PDF: generated with Tectonic through `scripts/build_report.sh`, size
  `39K`; remaining LaTeX warnings are non-blocking underfull boxes.

## 18. Results

P72 living store view

```text
budget bytes                       : 10,485,760
source dataset bytes                : 1,423,450
cold persisted bytes                : 332,405
runtime peak bytes                  : 164,734
exact recoverable bytes             : 1,357,914
ratio persistent                    : 4.085119
ratio runtime                       : 8.243071
ratio living                        : 2.366879
reopen equivalence                  : true
journal replay steps                : 809
compaction savings                  : 27,483 bytes / 82.467143%
adaptive rewrite count              : 3
guard decision                      : NO_GO_GUARD_INCOMPRESSIBLE_REFUSED
declared vs measured drift          : HARD_DRIFT / 78.359482%
decision                            : RECALIBRATE_P72_LIVING_COST_MODEL
```

### Filesystem cost split

| scope | metric | bytes |
|---|---|---:|
| cold | `cold_persisted_bytes` | 332,405 |
| cold | `manifest_bytes` | 85 |
| cold | `contract_bytes` | 108 |
| cold | `residual_bytes` | 292,651 |
| cold | `journal_bytes` | 5,768 |
| cold | `checkpoint_bytes` | 75 |
| cold | `checksum_bytes` | 17,042 |
| cold | `audit_metadata_bytes` | 61 |
| cold | `safety_metadata_bytes` | 79 |
| runtime | `runtime_peak_bytes` | 164,734 |
| runtime | `runtime_materialized_fiber_bytes` | 43,422 |
| runtime | `runtime_cache_bytes` | 24,576 |
| runtime | `runtime_actor_state_bytes` | 13,888 |
| runtime | `runtime_queue_bytes` | 6,272 |
| runtime | `runtime_temp_index_bytes` | 11,040 |
| runtime | `runtime_decoded_view_bytes` | 65,536 |

### Reopen and replay

| metric | value |
|---|---:|
| `reopen_equivalence` | true |
| `journal_replay_success` | true |
| `journal_replay_steps` | 809 |
| `roundtrip_success_rate` | 1.000000 |
| `retrieval_success_rate` | 1.000000 |
| `runtime_cache_required_for_correctness` | false |

### Ratio and drift

| metric | value |
|---|---:|
| `source_dataset_bytes` | 1,423,450 |
| `exact_recoverable_bytes` | 1,357,914 |
| `useful_retrieved_bytes` | 86,845 |
| `useful_retrieved_bytes_per_persistent_byte` | 0.261263 |
| `procedural_store_gain_vs_raw` | 4.282276 |
| `living_gain_vs_raw` | 2.481110 |
| `declared_persistent_bytes` | 186,368 |
| `measured_cold_persisted_bytes` | 332,405 |
| `declared_vs_cold_delta_percent` | 78.359482 |
| `drift_status` | HARD_DRIFT |

## 19. Decision

`RECALIBRATE_P72_LIVING_COST_MODEL`.

P72 passes the functional living-store gates: exact roundtrip remains true,
retrieval remains useful, journal replay succeeds, close/reopen preserves the
logical observable state, compaction preserves the logical state hash, and the
incompressible guard remains refused without false gain.

The decision remains conservative because declared persistent bytes
(`186,368`) and measured cold persisted bytes (`332,405`) still diverge by
`78.359482%`. This is an improvement over P71's global store drift, but it is
still a hard drift. P72 therefore validates the lifecycle path, not the final
living cost model.

## 20. Limitations

- Local-only campaign.
- The runtime working set is deterministic and compact, not a general storage
  runtime.
- Adaptive encoding is a first measured protocol, not a final optimizer.
- Drift remains `HARD_DRIFT` and prevents validation.
- The current benchmark uses one close/reopen cycle. Longer reopen/replay
  histories are deferred.

## 21. Recommendation for P73

If P72 passes reopen equivalence and the guard remains honest, P73 should test
longer living-store sessions with repeated open/close cycles, larger update
bursts, corruption injection, and explicit recovery modes.

## 22. Reproducibility notes

Generated stores stay under `artifacts/p72/` and are ignored by Git. The report
summarizes the important metrics instead of committing heavy artifacts.

## 23. Journal

- 2026-05-01: P72 implementation started from the P71 filesystem fiber store.
- 2026-05-01: Added lifecycle `.atlas` contract, invalid lifecycle examples,
  living-store CLI, P72 tests, and test stack audit.
- 2026-05-01: Local validation passed; P72 standard campaign produced
  `reopen_equivalence=true`, `ratio_living=2.366879`, and
  `decision=RECALIBRATE_P72_LIVING_COST_MODEL`.

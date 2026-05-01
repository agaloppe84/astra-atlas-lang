# ASTRA-P73 — Cubical Fiber Living Store Analysis

## 1. Executive summary

P73 implements and tests a Cubical Fiber Living Store. The store maps P71/P72
corpora into cubical cells, gives every cell six faces, stores gluing
constraints, audits face consistency, injects corruption, recovers controlled
corruption, and compares the living ratio against the frozen P72 baseline.

Local validation status: `PASS`.

Decision: `RECALIBRATE_P73_CUBICAL_FIBER_TOPOLOGY`.

## 2. Position after P72

P72 proved the living procedural store cycle:

- `source_dataset_bytes = 1,423,450`;
- `cold_persisted_bytes = 332,405`;
- `runtime_peak_bytes = 164,734`;
- `exact_recoverable_bytes = 1,357,914`;
- `ratio_living = 2.366879`;
- `reopen_equivalence = true`;
- `journal_replay_steps = 809`;
- `compaction_savings = 27,483 bytes / 82.467143%`;
- `guard_decision = NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`;
- `declared_vs_measured drift = HARD_DRIFT / 78.359482%`;
- decision: `RECALIBRATE_P72_LIVING_COST_MODEL`.

P73 tests whether a cubical topology can improve living ratio and localize
update/audit/compaction costs.

## 3. Central hypothesis: cubical topology of fibers

The cube does not store more information per bit. It organizes useful fiber
state into interior, faces and gluing constraints. Any gain must come from
factorizing structured boundaries, bounding face updates, localizing audit, and
compacting face journals more finely.

## 4. Cubical model: cells, faces, gluing

Each `CubicalCell` contains:

- `cell_id`;
- 3D coordinate;
- interior fiber bytes;
- six `CubicalFace` entries;
- neighbors;
- gluing constraints;
- local actor id;
- journal bytes;
- checksum;
- audit metadata;
- tombstone status;
- cost bytes.

Required face directions:

- `plus_x`;
- `minus_x`;
- `plus_y`;
- `minus_y`;
- `plus_z`;
- `minus_z`.

Gluing constraints checked:

- checksum match;
- delta match;
- boundary summary match;
- residual consistency;
- tombstone consistency.

## 5. `.atlas` cubical lifecycle syntax

P73 adds a declarative specialized syntax:

```atlas
atlas version=0.1;
p73_topology id=cubical_3d cell=cube faces=6 adjacency=von_neumann_6 boundary_policy=shared_faces gluing=checked;
p73_fiber_schema id=cubical_code_fiber cell_payload=generated_plus_residual face_payload=boundary_summary gluing_rule=checksum_and_delta projection=shallow journal=compact audit=face_checks compaction=threshold;
p73_actor_policy id=cubical_local_actor scope=cell_plus_faces budget_bytes=4194304 cache=face_aware journal=face_delta audit=gluing_consistency compaction=face_threshold;
p73_cubical_gates face_gluing_consistency=true hidden_face_storage=false face_update_propagation_bounded=true cubical_reopen_equivalence=true runtime_cache_not_required_for_correctness=true guard_no_false_gain=true;
```

This remains a declarative ASTRA contract surface, not a general-purpose
language.

## 6. Corpus-to-cube mapping

P73 reuses the local P71/P72 corpora and maps them into cubes:

| corpus | cube axes |
|---|---|
| `cubical_code_corpus` | file/module bucket, symbol/test bucket, line/content window |
| `cubical_logs_corpus` | time bucket, service, request group |
| `cubical_json_corpus` | record type, id bucket, projection path |
| `cubical_sparse_csv` | row bucket, column group, sparsity band |
| `incompressible_guard_blob` | refused/no-go guard |

## 7. Encoding strategies

Implemented strategies:

- `cell_generated_plus_residual`;
- `face_boundary_summary`;
- `face_shared_delta`;
- `cubical_generated_residual`;
- `cubical_adaptive_living`;
- `refused_guard`.

## 8. Decoding and CRUD actions

P73 reports:

- cell reads;
- face reads;
- interior updates;
- face updates;
- delete/tombstone propagation;
- gluing audit;
- face journal compaction;
- neighbor boundary rebuild;
- reopen.

## 9. Living store cycle

The local campaign executes:

- encode cubical store;
- open;
- read/query;
- update interiors and faces;
- delete/tombstone;
- audit gluing;
- compact face journals;
- inject corruption;
- recover;
- close;
- reopen;
- verify equivalence.

## 10. Long-session protocol

Standard campaign:

- `cycles = 10`;
- `queries = 5,000`;
- `updates = 1,000`;
- `deletes = 100`;
- `corruptions = 3`;
- compaction: `threshold`;
- adaptive: `on`.

Ambitious campaign:

- `cycles = 25`;
- `queries = 20,000`;
- `updates = 5,000`;
- `deletes = 500`;
- `corruptions = 10`;
- compaction: `aggressive`;
- adaptive: `on`.

## 11. Corruption/recovery protocol

Corruption is injected into the deterministic cubical campaign and must be
detected. Recovery is controlled; unrecovered corruption forces no-go in the
decision logic.

## 12. Filesystem cost breakdown

P73 counts:

- cubical topology;
- cell interiors;
- face summaries;
- face residuals;
- gluing constraints;
- face journals;
- cell journals;
- checksums;
- audit metadata;
- neighbor index;
- manifest;
- runtime materialized cells/faces;
- runtime cache;
- actor state;
- temp indexes.

## 13. Standard campaign results

Command:

```bash
cargo run -p atlas-cli -- cubical-store-bench \
  --corpus all \
  --budget-bytes 10485760 \
  --cycles 10 \
  --queries 5000 \
  --updates 1000 \
  --deletes 100 \
  --corruptions 3 \
  --compact threshold \
  --adaptive on \
  --compare-p72 baseline \
  --export-dir artifacts/p73/cubical_store_standard \
  --format json
```

Observed duration: `real 0.33s`.

| metric | value |
|---|---:|
| `cells` | 356 |
| `faces` | 2,136 |
| `source_dataset_bytes` | 1,514,888 |
| `exact_recoverable_bytes` | 1,449,352 |
| `cold_persisted_bytes` | 422,477 |
| `runtime_peak_bytes` | 98,037 |
| `ratio_living` | 2.679054 |
| `P72 baseline ratio_living` | 2.366879 |
| `cubical_gain_vs_p72` | 1.131893 |
| `face_factorization_gain` | 0.540997 |
| `gluing_overhead_ratio` | 0.072411 |
| `topology_overhead_ratio` | 0.009695 |
| `journal_replay_steps` | 1,466 |
| `face_journal_replay_steps` | 2,600 |
| `compaction_savings_bytes` | 85,334 |
| `face_compaction_savings_bytes` | 12,800 |
| `corruptions detected/recovered` | 3 / 3 |
| `drift_status` | NO_DRIFT |

## 14. Ambitious campaign results

Command:

```bash
cargo run -p atlas-cli -- cubical-store-bench \
  --corpus all \
  --budget-bytes 10485760 \
  --cycles 25 \
  --queries 20000 \
  --updates 5000 \
  --deletes 500 \
  --corruptions 10 \
  --compact aggressive \
  --adaptive on \
  --compare-p72 baseline \
  --export-dir artifacts/p73/cubical_store_ambitious \
  --format json
```

Observed duration: `real 0.33s`.

| metric | value |
|---|---:|
| `cells` | 356 |
| `faces` | 2,136 |
| `cold_persisted_bytes` | 550,477 |
| `runtime_peak_bytes` | 105,717 |
| `ratio_living` | 2.141876 |
| `P72 baseline ratio_living` | 2.366879 |
| `cubical_gain_vs_p72` | 0.904937 |
| `face_factorization_gain` | 0.540997 |
| `gluing_overhead_ratio` | 0.195089 |
| `journal_replay_steps` | 5,881 |
| `face_journal_replay_steps` | 13,000 |
| `compaction_savings_bytes` | 426,667 |
| `face_compaction_savings_bytes` | 64,000 |
| `corruptions detected/recovered` | 10 / 10 |
| `drift_status` | NO_DRIFT |

The ambitious run is correct but does not beat the P72 baseline. The gluing and
face-journal overhead rises to `0.195089`, which is too close to the
recalibration threshold for promotion.

## 15. P72 baseline comparison

| campaign | ratio living | vs P72 |
|---|---:|---:|
| P72 baseline | 2.366879 | 1.000000 |
| P73 standard | 2.679054 | 1.131893 |
| P73 ambitious | 2.141876 | 0.904937 |

Standard supports the cubical hypothesis. Ambitious shows that heavy face
journals can erase the gain.

## 16. Ratio/gain view

P73 cubical store view

```text
budget bytes                       : 10,485,760
cells                              : 356
faces                              : 2,136
cold persisted bytes                : 422,477 standard / 550,477 ambitious
runtime peak bytes                  : 98,037 standard / 105,717 ambitious
ratio living                       : 2.679054 standard / 2.141876 ambitious
P72 ratio living baseline           : 2.366879
cubical gain vs P72                 : 1.131893 standard / 0.904937 ambitious
face factorization gain             : 0.540997
gluing overhead ratio               : 0.072411 standard / 0.195089 ambitious
face gluing consistency             : true
reopen equivalence                  : true
corruptions detected/recovered      : 3/3 standard, 10/10 ambitious
guard decision                      : NO_GO_GUARD_INCOMPRESSIBLE_REFUSED
decision                            : RECALIBRATE_P73_CUBICAL_FIBER_TOPOLOGY
```

## 17. Topology overhead and gluing cost

Topology overhead is small in both campaigns:

- standard: `0.009695`;
- ambitious: `0.007441`.

Gluing overhead is the limiting factor:

- standard: `0.072411`;
- ambitious: `0.195089`.

This supports recalibrating face journal policy and gluing audit frequency
before any promotion.

## 18. Guard incompressible

The incompressible guard remains refused:
`NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`.

No cubical ratio includes false gain from guard bytes.

## 19. Test stack hygiene P73

P73 added `tests/p73_tests.rs` and updated `tests/atlas_tests.rs` to include
eight new invalid cubical examples. No useful test was deleted.

See `docs/analysis/ASTRA-P73-test-stack-audit.md`.

## 20. Decision

`RECALIBRATE_P73_CUBICAL_FIBER_TOPOLOGY`.

Reasons:

- standard improves `ratio_living` vs P72;
- ambitious does not maintain the gain;
- gluing overhead rises sharply under update pressure;
- face gluing, reopen, corruption recovery and guard all pass;
- promotion requires stability across standard and ambitious, not standard
  alone.

## 21. Limitations

- Local-only campaigns.
- Cubical mapping is deterministic and compact, not a production spatial index.
- Ambitious campaign is still synthetic.
- Corruption/recovery is controlled, not adversarial filesystem recovery.
- No external dataset.
- No claim that cubes increase information density.

## 22. Recommendation for P74

P74 should calibrate cubical gluing and face-journal policy:

- adaptive face journal compaction;
- lower-cost gluing summaries;
- update batching by face direction;
- selective audit cadence;
- stress tests where locality becomes random or faces churn heavily.

## 23. Reproducibility notes

Generated stores stay under `artifacts/p73/` and are ignored by Git. Reports
summarize the metrics instead of committing heavy store artifacts.

## 24. Journal

- 2026-05-02: Added P73 cubical lifecycle syntax, valid example and eight
  invalid examples.
- 2026-05-02: Added `cubical-store-bench`, P73 exports and `tests/p73_tests.rs`.
- 2026-05-02: Executed standard and ambitious local campaigns.
- 2026-05-02: Decision kept conservative:
  `RECALIBRATE_P73_CUBICAL_FIBER_TOPOLOGY`.
- 2026-05-02: Generated Results PDF with Tectonic via
  `scripts/build_report.sh`; output size was 37K. Tectonic reported one
  non-blocking underfull hbox warning.

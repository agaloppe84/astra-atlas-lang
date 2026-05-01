# ASTRA-P73 — Cubical Fiber Living Store Validation

P73 tests a cubical topology for the living procedural fiber store. The cube is
not claimed to store more information per bit. It organizes each local fiber
into interior payload, six faces, and gluing constraints so structured
boundaries can be factored, updated, audited and compacted locally.

## Scope

P73 compares the P72 living-store baseline against a cubical store with:

- cubical cells;
- six required faces: `plus_x`, `minus_x`, `plus_y`, `minus_y`, `plus_z`,
  `minus_z`;
- boundary summaries;
- gluing constraints;
- face journals;
- face-level audit;
- corruption detection and controlled recovery;
- close/reopen equivalence.

## Cubical `.atlas`

The specialized P73 syntax is declarative:

```atlas
atlas version=0.1;
p73_topology id=cubical_3d cell=cube faces=6 adjacency=von_neumann_6 boundary_policy=shared_faces gluing=checked;
p73_fiber_schema id=cubical_code_fiber cell_payload=generated_plus_residual face_payload=boundary_summary gluing_rule=checksum_and_delta projection=shallow journal=compact audit=face_checks compaction=threshold;
p73_actor_policy id=cubical_local_actor scope=cell_plus_faces budget_bytes=4194304 cache=face_aware journal=face_delta audit=gluing_consistency compaction=face_threshold;
p73_cubical_gates face_gluing_consistency=true hidden_face_storage=false face_update_propagation_bounded=true cubical_reopen_equivalence=true runtime_cache_not_required_for_correctness=true guard_no_false_gain=true;
```

No general-purpose `.atlas` execution is added.

## CLI

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

## Exports

Generated artifacts stay under `artifacts/p73/` and are ignored by Git:

- `p73_cubical_store_report.json`
- `p73_cubical_cells.jsonl`
- `p73_cubical_faces.jsonl`
- `p73_gluing_audit.csv`
- `p73_cost_breakdown.csv`
- `p73_corruption_recovery.json`
- `p73_summary.md`
- `cubical_store/cold/`
- `cubical_store/runtime/`
- `cubical_store/reports/`

## Local standard result

- `cells = 356`
- `faces = 2,136`
- `cold_persisted_bytes = 422,477`
- `runtime_peak_bytes = 98,037`
- `ratio_living = 2.679054`
- `P72 baseline ratio_living = 2.366879`
- `cubical_gain_vs_p72 = 1.131893`
- `face_factorization_gain = 0.540997`
- `gluing_overhead_ratio = 0.072411`
- `face_gluing_consistency = true`
- `cubical_reopen_equivalence = true`
- `corruptions detected/recovered = 3/3`
- `guard_decision = NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`
- `drift_status = NO_DRIFT`
- `decision = RECALIBRATE_P73_CUBICAL_FIBER_TOPOLOGY`

## Local ambitious result

- `cycles = 25`
- `queries = 20,000`
- `updates = 5,000`
- `deletes = 500`
- `corruptions detected/recovered = 10/10`
- `ratio_living = 2.141876`
- `cubical_gain_vs_p72 = 0.904937`
- `gluing_overhead_ratio = 0.195089`

The ambitious run remains correct but no longer beats the P72 baseline because
face journals and gluing overhead dominate under heavier update pressure.

## Decision policy

Possible P73 decisions are:

- `PROMOTE_P73_CUBICAL_FIBER_STORE`
- `RECALIBRATE_P73_CUBICAL_FIBER_TOPOLOGY`
- `RECALIBRATE_P73_GLUE_AUDIT_COST`
- `NO_GO_P73_CUBICAL_FIBER_STORE`

P73 does not promote because the standard run improves the living ratio, but
the ambitious run falls below the P72 baseline. The next step is topology and
gluing-cost recalibration.

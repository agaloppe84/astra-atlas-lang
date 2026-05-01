# ASTRA-P75 — Mixed-Topology Fiber Router Analysis

## 1. Executive summary

P75 adds a deterministic mixed-topology router for living procedural fiber
stores. Instead of choosing a single topology globally, the router selects a
local topology from observable fiber features: path-like, relation-heavy,
tag-heavy, sparse-tile, update-heavy or guard.

Decision: `RECALIBRATE_P75_ROUTER_POLICY`.

The router preserves the high `ratio_living` signal from P74 while lowering
update and audit costs versus `hierarchical_only`. It is not promoted yet
because P75 still lacks a per-fiber wrong-route oracle; P76 should test that
before making the router a default architecture.

## 2. Position after P74

P74 showed that `hierarchical_tile_fiber` is the best single topology observed
for global ratio and retrieval, but `baseline_linear_fiber` remains better for
simple update cost. Code, logs, JSON and sparse CSV also prefer different
topologies when evaluated per corpus. P75 therefore tests whether a router can
keep the ratio while routing local fibers to a better topology family.

## 3. Process rules: living memory, max ratio, local procedural virtual space

P75 keeps three project axes central:

- living memory: the decisive run includes encode, open, read/query, update,
  delete, audit, compact, close and reopen;
- maximum ratio: `ratio_living`, `cold_persisted_bytes`,
  `runtime_peak_bytes`, `exact_recoverable_bytes`, `useful_retrieved_bytes` and
  update/audit cost are reported together;
- local procedural virtual space: the virtual space is not materialized
  globally. The router only chooses the local fiber topology when an address is
  reached.

Small parser/typechecker tests remain useful, but they do not justify the P75
architecture decision alone.

## 4. Central hypothesis: no single topology is optimal

The P75 hypothesis is that the best living ratio comes from a route policy:
hierarchical tiles for sparse tiles and generic retrieval, tries for paths and
prefixes, graphs for file/symbol/test/doc relations, hypergraphs for tag-heavy
records and linear fibers for simple update-heavy cells.

## 5. Router model

`MixedTopologyRouter` is deterministic. It uses `FiberFeatureExtractor` output
to produce a `RoutedFiberDecision`:

- selected topology;
- routing reason;
- fallback status;
- confidence;
- expected ratio class;
- expected update cost class.

Router metadata and overhead are counted as paid bytes.

## 6. Routing features

The implemented feature set is:

- `corpus_kind`;
- `address_kind`;
- `update_pressure`;
- `retrieval_priority`;
- `locality_profile`;
- `relation_density`;
- `tag_density`;
- `sparsity_level`;
- `path_depth`;
- `guard_flag`.

## 7. Topology candidates

P75 routes among:

- `baseline_linear_fiber`;
- `cubical_6face_fiber`;
- `trie_prefix_fiber`;
- `graph_adjacency_fiber`;
- `hypergraph_tag_fiber`;
- `hierarchical_tile_fiber`.

The guard corpus is not routed to a success topology.

## 8. Corpora and 10 MiB source generation

The benchmark uses deterministic local data only. For `--corpus all`, the
target and actual source bytes are both 10,485,760. The source is split across
code, logs, JSON, sparse CSV and incompressible guard corpora. Deterministic
expansion is used where needed to hit the target; no external dataset is used.

## 9. Living benchmark protocol

Each campaign writes source corpora and real store files under `artifacts/p75/`.
The living path includes encode, open, read/query, update, delete, audit,
compact, close, reopen and guard verification. Cold, runtime and replay costs
are separated and measured from filesystem metadata.

## 10. Baselines

P75 compares:

| Baseline | Meaning |
|---|---|
| `mixed_router` | route per local fiber feature |
| `hierarchical_only` | use hierarchical tile everywhere |
| `linear_only` | use baseline linear everywhere |
| `cubical_only` | use cubical six-face everywhere |

Additional CLI policies exist for trie-only, graph-only and hypergraph-only
compatibility, but the core comparison uses the four baselines above.

## 11. Standard campaign results

Command:

```bash
cargo run -p atlas-cli -- mixed-topology-bench \
  --corpus all \
  --router mixed \
  --target-source-bytes 10485760 \
  --cycles 10 \
  --queries 10000 \
  --updates 1000 \
  --deletes 100 \
  --compact threshold \
  --adaptive on \
  --locality mixed \
  --update-pressure medium \
  --export-dir artifacts/p75/mixed_topology_standard \
  --format json
```

Observed duration: `real 0.41s`.

| Metric | Value |
|---|---:|
| target_source_bytes | 10,485,760 |
| actual_source_bytes | 10,485,760 |
| ratio_living router | 4.759326 |
| ratio_living hierarchical_only | 4.831165 |
| router / hierarchical ratio | 0.985130 |
| update cost router | 31,250 |
| update cost hierarchical | 42,290 |
| audit cost router | 805 |
| audit cost hierarchical | 845 |
| retrieval_success_rate | 1.000000 |
| reopen_equivalence | true |
| drift_status | `NO_DRIFT` |
| guard_decision | `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED` |

Selected topology mix:

| Topology | Routed weight |
|---|---:|
| `hierarchical_tile_fiber` | 640 |
| `trie_prefix_fiber` | 384 |
| `hypergraph_tag_fiber` | 256 |
| `baseline_linear_fiber` | 128 |
| `graph_adjacency_fiber` | 128 |
| `refused_guard` | 128 |

## 12. Ambitious campaign results if executed

Command:

```bash
cargo run -p atlas-cli -- mixed-topology-bench \
  --corpus all \
  --router mixed \
  --target-source-bytes 10485760 \
  --cycles 25 \
  --queries 50000 \
  --updates 5000 \
  --deletes 500 \
  --compact adaptive \
  --adaptive on \
  --locality mixed \
  --update-pressure high \
  --export-dir artifacts/p75/mixed_topology_ambitious \
  --format json
```

Observed duration: `real 0.38s`.

| Metric | Value |
|---|---:|
| target_source_bytes | 10,485,760 |
| actual_source_bytes | 10,485,760 |
| ratio_living router | 4.743249 |
| ratio_living hierarchical_only | 4.843523 |
| router / hierarchical ratio | 0.979297 |
| update cost router | 183,099 |
| update cost hierarchical | 248,235 |
| audit cost router | 3,575 |
| audit cost hierarchical | 3,675 |
| retrieval_success_rate | 1.000000 |
| reopen_equivalence | true |
| drift_status | `NO_DRIFT` |
| guard_decision | `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED` |

## 13. Router vs hierarchical-only

The router does not beat hierarchical-only on raw `ratio_living`, but it remains
above the target threshold:

- standard: 0.985130 of hierarchical-only;
- ambitious: 0.979297 of hierarchical-only.

It improves update and audit cost in both campaigns.

## 14. Router vs linear-only

`linear_only` remains a simple update-cost reference, but it loses too much
ratio:

- standard `ratio_living_linear_only`: 3.694047;
- ambitious `ratio_living_linear_only`: 3.330396.

P75 keeps linear as a routed option for update-heavy sparse cells, not as the
global topology.

## 15. Router vs cubical-only

`cubical_only` remains useful as a P73 reference, but it is not competitive as a
global router policy:

- standard `ratio_living_cubical_only`: 2.719257;
- ambitious `ratio_living_cubical_only`: 2.179646.

## 16. Routing decision analysis

The routing mix confirms the intended policy:

- code path-heavy fibers use tries;
- code relation-heavy fibers use graph adjacency;
- log and JSON tag-heavy fibers use hypergraphs;
- sparse tiles use hierarchical tiles;
- update-heavy sparse cells use linear fibers;
- guard data is refused/no-go.

`route_confidence_avg` is 0.838462. `wrong_route_count` is currently 0 because
P75 does not yet implement an independent wrong-route oracle. This is the main
reason promotion is withheld.

## 17. Phase map

| Campaign | Green | Yellow | Red | Grey |
|---|---:|---:|---:|---:|
| Standard | 96 | 92 | 52 | 0 |
| Ambitious | 0 | 168 | 72 | 0 |

The ambitious phase map is intentionally more conservative because high update
pressure keeps many cells in recalibration.

## 18. Ratio/gain view

P75 keeps the living ratio well above P72 and P73 baselines:

| Metric | Standard | Ambitious |
|---|---:|---:|
| router ratio_living | 4.759326 | 4.743249 |
| hierarchical-only ratio_living | 4.831165 | 4.843523 |
| P72 baseline ratio_living | 2.366879 | 2.366879 |
| P73 cubical ratio_living | 2.679054 | 2.679054 |

## 19. Update/audit cost view

| Metric | Standard | Ambitious |
|---|---:|---:|
| update cost router | 31,250 | 183,099 |
| update cost hierarchical | 42,290 | 248,235 |
| audit cost router | 805 | 3,575 |
| audit cost hierarchical | 845 | 3,675 |

The router reduces update cost by selecting linear fibers only where the local
structure calls for them.

## 20. Guard incompressible

The guard corpus is refused and does not contribute to success:

- guard decision: `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`;
- guard no false gain: true;
- guard refusal count: 128.

## 21. Impact on .atlas

P75 adds a specialized declarative syntax:

```atlas
topology_router id=mixed_router default=hierarchical_tile guard_policy=refuse_or_raw_no_gain fallback=bounded hidden_router_storage=false;
route corpus=code path_heavy=trie_prefix relation_heavy=graph_adjacency default=hierarchical_tile;
router_gates living_memory_only=true target_source_bytes=10485760 reopen_equivalence=true guard_no_false_gain=true hidden_router_storage=false ratio_living_reported=true;
```

This is not a general-purpose language extension. It is a narrow ASTRA router
contract.

## 22. Encoding/decoding implications

The store no longer assumes a universal topology. Encoding can attach a route
decision to each local fiber family. Decoding remains local: reach an address,
read the route metadata, instantiate the selected topology and regenerate the
useful fiber without materializing the global virtual space.

## 23. Decision

`RECALIBRATE_P75_ROUTER_POLICY`.

The target ratio and update/audit criteria are met, and
`promotion_candidate=true` in the report. The final decision remains
conservative because P75 lacks an independent wrong-route oracle and broader
stress for route stability. P76 should test route quality directly.

## 24. Limitations

- deterministic local corpora only;
- deterministic expansion to 10 MiB, no external dataset;
- route confidence is heuristic;
- no independent wrong-route oracle yet;
- no multi-machine run;
- stores are benchmark artifacts, not a final storage format.

## 25. Recommendation for P76

P76 should add a router-quality oracle:

- compare selected route against best per-fiber topology;
- measure wrong-route penalties;
- add route churn under updates;
- test mixed-router manifests over longer living sessions;
- promote only if wrong-route cost remains bounded.

## 26. Reproducibility notes

Artifacts are generated under `artifacts/p75/` and ignored by Git. The versioned
trace is this report plus the Results PDF.

## 27. Journal

- 2026-05-02: added `src/p75.rs`;
- 2026-05-02: added `mixed-topology-bench`;
- 2026-05-02: added P75 `.atlas` valid contract and 8 invalid contracts;
- 2026-05-02: ran standard and ambitious living-memory campaigns;
- 2026-05-02: kept decision conservative as
  `RECALIBRATE_P75_ROUTER_POLICY`.

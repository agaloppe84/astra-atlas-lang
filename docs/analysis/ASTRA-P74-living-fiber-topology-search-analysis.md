# ASTRA-P74 — Living Fiber Topology Search Analysis

## 1. Executive summary

P74 adds a living-memory topology search for address-fiber stores. The campaign
compares six topology families on deterministic local source data targeting
10 MiB, with living operations: encode, open, read/query, update, delete, audit,
compact, close, reopen and guard verification.

Decision: `RECALIBRATE_P74_FIBER_TOPOLOGY_SEARCH`.

The main signal is positive but not yet promotable. `hierarchical_tile_fiber`
has the best single observed `ratio_living` on sparse CSV, while code, logs and
JSON prefer different topologies. P74 therefore confirms that topology is a core
parameter, not a single universal answer.

## 2. Position after P73

P73 showed that cubical topology helps in standard mode but loses ground under
ambitious pressure when gluing and face journals grow. P74 generalizes the
question: instead of asking whether cubes are good, it asks which topology fits
which corpus under living-memory constraints.

## 3. New process rule: living-memory decisions on about 10 MiB

Starting with P74, architectural decisions about the virtual/real memory ratio
must be based on living-memory campaigns around 10 MiB of source data. Smaller
tests remain necessary for parser, typechecker and non-regression coverage, but
they cannot justify a new architecture by themselves.

The P74 campaigns use:

- source data target = 10,485,760 bytes;
- living topology stores;
- read/query/update/delete/audit/compact/close/reopen;
- `reopen_equivalence`;
- incompressible guard refusal;
- measured filesystem cold/runtime bytes;
- `ratio_living`;
- retrieval results.

## 4. Central hypothesis: topology of fibers

A topology cannot store more information per bit. It can only organize fibers so
that existing structure is factored better: prefixes, graph edges, tags,
hierarchies, faces, residuals and local update scopes.

P74 tests whether a topology matched to the corpus can improve `ratio_living`
without hiding topology, index, journal, audit, checksum or residual overhead.

## 5. Topologies tested

| Topology | Intended fit |
|---|---|
| `baseline_linear_fiber` | P72-style simple living fiber baseline |
| `cubical_6face_fiber` | regular local neighborhoods and boundary gluing |
| `trie_prefix_fiber` | paths, tokens, services, JSON paths, prefixes |
| `graph_adjacency_fiber` | files, symbols, tests, docs and related records |
| `hypergraph_tag_fiber` | tags, severities, services, types and categories |
| `hierarchical_tile_fiber` | row/column tiles, pyramids, directories, sparse blocks |

## 6. Corpora and 10 MiB generation

The campaign does not use network or external datasets. It builds deterministic
local corpora and splits the target source bytes across selected corpus
families:

| Corpus | Bytes in all-corpus run | Note |
|---|---:|---|
| `real_code_corpus_10m` | 2,097,152 | repo-like code/docs/tests expanded deterministically |
| `realish_logs_10m` | 2,097,152 | deterministic structured logs |
| `realish_json_10m` | 2,097,152 | deterministic JSON records |
| `sparse_csv_10m` | 2,097,152 | deterministic sparse table |
| `incompressible_guard_10m` | 2,097,152 | deterministic guard, refused/no-go |

Total actual source bytes: 10,485,760.

## 7. Mapping corpus to topology

| Corpus | Strong mapping |
|---|---|
| Code | graph adjacency and trie prefixes |
| Logs | hypergraph tags and trie prefixes |
| JSON | trie prefixes and hypergraph tags |
| Sparse CSV | hierarchical tiles |
| Guard | refused/no false gain |

## 8. Living benchmark protocol

The command creates source corpus files and per-topology stores under
`artifacts/p74/`. For each topology/corpus pair, the report tracks measured
cold persisted bytes, runtime peak bytes, topology/interface overhead,
journal replay steps, compaction savings, reopen equivalence, retrieval and
guard behavior.

## 9. Metrics

Core metrics:

- `source_dataset_bytes`;
- `cold_persisted_bytes`;
- `runtime_peak_bytes`;
- `exact_recoverable_bytes`;
- `useful_retrieved_bytes`;
- `ratio_persistent`;
- `ratio_runtime`;
- `ratio_living`;
- `topology_overhead_bytes`;
- `topology_overhead_ratio`;
- `interface_or_edge_overhead`;
- `journal_replay_steps`;
- `compaction_savings`;
- `reopen_equivalence`;
- `retrieval_success_rate`;
- `roundtrip_success_rate`;
- `guard_no_false_gain`;
- `drift_status`.

## 10. Standard campaign results

Command:

```bash
cargo run -p atlas-cli -- topology-living-bench \
  --corpus all \
  --topology all \
  --target-source-bytes 10485760 \
  --cycles 10 \
  --queries 10000 \
  --updates 1000 \
  --deletes 100 \
  --compact threshold \
  --adaptive on \
  --locality mixed \
  --update-pressure medium \
  --export-dir artifacts/p74/topology_living_standard \
  --format json
```

Observed duration: `real 0.40s`.

| Metric | Value |
|---|---:|
| target_source_bytes | 10,485,760 |
| actual_source_bytes | 10,485,760 |
| best_topology_overall | `hierarchical_tile_fiber` |
| best_topology_by_ratio_living | `hierarchical_tile_fiber` |
| best_topology_by_retrieval | `hierarchical_tile_fiber` |
| best_topology_by_update_cost | `baseline_linear_fiber` |
| best_ratio_living | 4.742439 |
| phase map green / yellow / red | 163 / 125 / 72 |
| guard_decision | `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED` |
| reopen_equivalence | true |
| drift_status | `NO_DRIFT` |
| decision | `RECALIBRATE_P74_FIBER_TOPOLOGY_SEARCH` |

Best per structured corpus:

| Corpus | Best topology | ratio_living | cold bytes | runtime peak | topology overhead |
|---|---|---:|---:|---:|---:|
| code | `graph_adjacency_fiber` | 4.443323 | 285,706 | 95,006 | 0.095448 |
| logs | `hypergraph_tag_fiber` | 4.543068 | 266,163 | 84,068 | 0.103658 |
| JSON | `trie_prefix_fiber` | 4.343689 | 282,994 | 92,562 | 0.075323 |
| sparse CSV | `hierarchical_tile_fiber` | 4.742439 | 263,462 | 84,767 | 0.083891 |

## 11. Ambitious campaign results if executed

Command:

```bash
cargo run -p atlas-cli -- topology-living-bench \
  --corpus all \
  --topology all \
  --target-source-bytes 10485760 \
  --cycles 25 \
  --queries 50000 \
  --updates 5000 \
  --deletes 500 \
  --compact adaptive \
  --adaptive on \
  --locality mixed \
  --update-pressure high \
  --export-dir artifacts/p74/topology_living_ambitious \
  --format json
```

Observed duration: `real 0.50s`.

| Metric | Value |
|---|---:|
| target_source_bytes | 10,485,760 |
| actual_source_bytes | 10,485,760 |
| best_topology_overall | `hierarchical_tile_fiber` |
| best_topology_by_ratio_living | `hierarchical_tile_fiber` |
| best_topology_by_retrieval | `hierarchical_tile_fiber` |
| best_topology_by_update_cost | `baseline_linear_fiber` |
| best_ratio_living | 4.742450 |
| phase map green / yellow / red | 163 / 125 / 72 |
| guard_decision | `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED` |
| reopen_equivalence | true |
| drift_status | `NO_DRIFT` |
| decision | `RECALIBRATE_P74_FIBER_TOPOLOGY_SEARCH` |

Best per structured corpus:

| Corpus | Best topology | ratio_living | cold bytes | runtime peak | topology overhead |
|---|---|---:|---:|---:|---:|
| code | `graph_adjacency_fiber` | 4.443352 | 285,704 | 106,406 | 0.095448 |
| logs | `hypergraph_tag_fiber` | 4.543098 | 266,161 | 94,156 | 0.103659 |
| JSON | `trie_prefix_fiber` | 4.343708 | 282,992 | 103,670 | 0.075324 |
| sparse CSV | `hierarchical_tile_fiber` | 4.742450 | 263,460 | 94,940 | 0.083891 |

## 12. Phase map

P74 generated a phase map over topology, corpus, locality and update pressure.

| Campaign | Green | Yellow | Red | Grey |
|---|---:|---:|---:|---:|
| Standard | 163 | 125 | 72 | 0 |
| Ambitious | 163 | 125 | 72 | 0 |

Red includes the guard corpus and failure zones where topology fit is poor.

## 13. Ranking by ratio_living

Best single observed ratio:

- `hierarchical_tile_fiber` on `sparse_csv_10m`;
- standard `ratio_living = 4.742439`;
- ambitious `ratio_living = 4.742450`.

This is above the frozen baselines:

- P72 `ratio_living = 2.366879`;
- P73 cubical standard `ratio_living = 2.679054`.

## 14. Ranking by retrieval

Retrieval success ties at 1.0 for the best structured configurations in this
deterministic campaign. The tie-breaking in the report selects
`hierarchical_tile_fiber`, but the interpretation is not that hierarchy is
universally best for retrieval. P74 shows that retrieval ranking needs a more
discriminating query set in P75.

## 15. Ranking by update/audit cost

`baseline_linear_fiber` has the lowest update/interface overhead, as expected,
because it carries minimal topology. This is not the best ratio topology; it is
a low-overhead baseline.

## 16. Guard incompressible

The incompressible guard is refused/no-go and does not contribute false gain.

Guard decision: `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`.

## 17. Comparison with P72/P73

| Reference | ratio_living |
|---|---:|
| P72 living baseline | 2.366879 |
| P73 cubical standard | 2.679054 |
| P74 best standard | 4.742439 |
| P74 best ambitious | 4.742450 |

P74 improves the best observed ratio, but the winning topology is tied to the
corpus. This supports topology search rather than immediate promotion.

## 18. Impact on .atlas

P74 adds specialized declarative syntax:

```atlas
fiber_topology id=topology_name kind=trie_prefix adjacency=bounded interface_policy=compact gluing=checked update_scope=local audit=selective edge_policy=local hyperedge_policy=bounded;
fiber_schema id=code_fiber topology=trie_prefix payload=generated_plus_residual index=local journal=compact audit=selective;
living_topology_gates reopen_equivalence=true guard_no_false_gain=true hidden_topology_storage=false topology_overhead_counted=true ratio_living_reported=true runtime_cache_not_required_for_correctness=true;
```

This remains specialized ASTRA metadata. No loops, functions or general-purpose
execution are added.

## 19. Encoding/decoding implications

P74 implies that encoding should choose topology per corpus or per fiber family:

- code: graph adjacency and trie prefixes;
- logs: hypergraph tags;
- JSON: trie prefixes;
- sparse CSV: hierarchical tiles;
- guard: refuse or raw/no-go.

## 20. Decision

`RECALIBRATE_P74_FIBER_TOPOLOGY_SEARCH`.

Reasons:

- living-memory protocol passed;
- source data reached 10,485,760 bytes;
- `reopen_equivalence=true`;
- guard remained refused/no false gain;
- `drift_status=NO_DRIFT`;
- topology wins are corpus-dependent;
- retrieval ranking needs a more discriminating query set before P75 promotion.

## 21. Limitations

- Corpus is deterministic and local, not external.
- Code corpus is expanded deterministically to meet the 10 MiB target.
- Retrieval ties in the current query set hide topology differences.
- Store files are deterministic proxies for topology cost; no claim of final
  production storage format.
- No multi-machine validation.

## 22. Recommendation for P75

P75 should implement topology selection per corpus/fiber family:

- a routing policy that chooses graph/trie/hypergraph/hierarchical based on
  measured structure;
- a stronger retrieval suite that breaks the 1.0 tie;
- update/audit stress by topology;
- validation that topology overhead remains counted;
- a candidate mixed-topology architecture manifest.

## 23. Reproducibility notes

Artifacts are generated under `artifacts/p74/` and ignored by Git. The report
summarizes metrics instead of committing stores.

## 24. Journal

- 2026-05-02: Added `src/p74.rs`, `topology-living-bench`, specialized P74
  `.atlas` syntax, one valid example and eight invalid examples.
- 2026-05-02: Added `tests/p74_tests.rs` and P74 test stack audit.
- 2026-05-02: Ran standard and ambitious P74 living-memory campaigns with
  `target_source_bytes=10485760`.
- 2026-05-02: Decision kept conservative:
  `RECALIBRATE_P74_FIBER_TOPOLOGY_SEARCH`.

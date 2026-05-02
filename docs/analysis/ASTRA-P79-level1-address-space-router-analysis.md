# ASTRA-P79 - Level-1 Address Space Router Analysis

## 1. Executive summary

P79 implements a deterministic Level-1 Address Router above the P78 virtual address space. The router chooses between path trie, content DAG, product typed space, graph address space, hybrid multi-index, and grid baselines by local file/address features. The run remains living-memory only: encode, open, address lookup, read/query, update/delete, audit, compact, close, reopen.

P79 improves the index footprint versus hybrid-only and keeps lookup cost close to path-trie, but it misses the promotion ratio gate by a narrow margin:

- `ratio_living_router = 5.174901`
- `ratio_living_hybrid_only = 5.340001`
- `router/hybrid = 0.969082`
- target gate: `>= 0.970000`

Decision: `RECALIBRATE_P79_LEVEL1_ROUTER_POLICY`.

## 2. Position after P78

P78 found that `hybrid_multi_index_space` maximizes `ratio_living`, while `path_trie` gives the best address lookup cost. The limiting factor was `index_size`. P79 tests whether a router can keep the hybrid ratio while reducing lookup and index cost.

## 3. Central hypothesis: route level-1 address spaces

The hypothesis is that no single level-1 topology should be global. Path-like addresses can use a trie, chunked binary can use a content DAG, typed files can use a product typed space, relation-heavy objects can use a graph, and multi-access records can keep the hybrid index.

## 4. Level1AddressRouter model

`Level1AddressRouter` owns a `Level1RoutePolicy`:

- default: `product_typed_space`
- path-like: `path_trie`
- chunked binary: `content_addressed_dag`
- typed namespace: `product_typed_space`
- relation-heavy: `graph_address_space`
- multi-access: `hybrid_multi_index_space`
- regular grid: `grid3d`
- guard: `refuse_or_raw_no_gain`

The router never routes guard data to a success topology.

## 5. Routing features

P79 extracts deterministic features:

- file extension and file type class;
- address shape;
- path depth and prefix entropy;
- content hashability and chunk repetition;
- relation and namespace density;
- query pattern, update pressure, retrieval priority, locality profile;
- estimated index cost and lookup steps;
- guard flag.

## 6. Level-1 topologies

The candidate space remains the P78 set: `grid2d`, `grid3d`, `hierarchical_tree`, `path_trie`, `content_addressed_dag`, `graph_address_space`, `product_typed_space`, `hybrid_multi_index_space`.

## 7. Oracle model

`Level1Oracle` compares the router selection with the best observed topology per feature group. It produces wrong-route counts and cost. P79 standard reports:

- wrong route count: `110`
- wrong route cost: `359`
- router/oracle ratio: `0.957960`

Ambitious reports the same count and ratio, with wrong route cost `407` because update pressure is higher.

## 8. Universal file support

P79 preserves the P78 universal store contract. Unknown extensions use explicit raw fallback. Incompressible guard data is refused as `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`, with no false gain.

## 9. Living-memory benchmark

The standard campaign used:

- target source bytes: `10,485,760`
- actual source bytes: `10,485,760`
- cycles: `10`
- queries: `10,000`
- updates: `1,000`
- deletes: `100`
- compact: `adaptive`
- adaptive: `on`

The ambitious campaign used:

- cycles: `25`
- queries: `50,000`
- updates: `5,000`
- deletes: `500`

Both campaigns executed locally through `atlas-cli level1-router-bench`.

## 10. Virtual space metrics

P79 keeps virtual bytes explicit as materialization equivalents:

- effective addresses: `222,464,000,000`
- virtual cells: `10,000`
- virtual fibers: `40,000`
- virtual chunks: `40,000`
- virtual effective bytes equivalent: `96,415,629`
- limiting factor: `index_size`

These bytes are not stored bytes.

## 11. Addressing and CRUD metrics

Standard P79:

- lookup p95 router: `7.800` steps
- lookup p95 path-trie: `7.000` steps
- lookup p95 bytes read: `3,840`
- CRUD success rate: `1.000000`
- retrieval success rate: `1.000000`
- reopen equivalence: `true`
- drift status: `NO_DRIFT`

## 12. Standard campaign results

P79 level-1 router view:

| Metric | Value |
|---|---:|
| target source bytes | 10,485,760 |
| actual source bytes | 10,485,760 |
| router policy | p79-router |
| best single topology | hybrid_multi_index_space |
| ratio living router | 5.174901 |
| ratio living hybrid-only | 5.340001 |
| ratio living oracle | 5.402000 |
| router/oracle ratio | 0.957960 |
| lookup p95 router | 7.800 |
| lookup p95 path-trie | 7.000 |
| index bytes router | 1,080,033 |
| index bytes hybrid | 1,342,177 |
| index saved vs hybrid | 262,144 |
| CRUD success rate | 1.000000 |
| reopen equivalence | true |
| guard decision | NO_GO_GUARD_INCOMPRESSIBLE_REFUSED |
| decision | RECALIBRATE_P79_LEVEL1_ROUTER_POLICY |

## 13. Ambitious campaign results if executed

Ambitious executed locally. It matches the standard ratio and lookup metrics, and increases wrong route cost from `359` to `407` because update pressure is higher. Runtime remained short on this deterministic synthetic/local benchmark.

## 14. Router vs hybrid-only

The router saves `262,144` level-1 index bytes versus hybrid-only, and improves lookup p95 from `8.000` to `7.800` steps. However, it loses `0.165100` `ratio_living` points versus hybrid-only.

The promotion gate requires:

`ratio_living_router >= 0.97 * ratio_living_hybrid_only`.

Observed:

`5.174901 / 5.340001 = 0.969082`.

## 15. Router vs path-trie-only

Path-trie keeps the lower lookup bound at `7.000` p95 steps. The router remains within the required lookup envelope:

`7.800 <= 7.000 * 1.15`.

The router also has higher ratio than path-trie-only.

## 16. Router vs oracle

The oracle reaches `ratio_living = 5.402000`. The router reaches `5.174901`, or `0.957960` of oracle. Wrong routes concentrate on relation-heavy code, logs service prefixes, and CSV regular projections.

## 17. Phase map

P79 phase map:

- green: `82`
- yellow: `46`
- red: `11`
- grey: `0`

Failure modes:

- router misses the hybrid ratio gate by a small margin;
- some relation-heavy features should use hybrid rather than graph;
- some high-pressure log prefixes should use product typed rather than path trie;
- regular CSV projections need a better choice than grid3d.

## 18. Decision

`RECALIBRATE_P79_LEVEL1_ROUTER_POLICY`

Rationale:

- living pipeline passes;
- retrieval, CRUD, drift, guard and reopen all pass;
- index bytes improve versus hybrid-only;
- lookup p95 stays close to path-trie;
- router/hybrid ratio is `0.969082`, just below the `0.970000` promotion gate;
- oracle wrong-route count/cost is still non-zero.

## 19. Limitations

The router is deterministic and intentionally compact. It is not an opaque ML model. The universal binary/image/video-like fixtures remain deterministic local substitutes, not external datasets. P79 does not claim that routing creates information; it only reorganizes address lookup and index cost.

## 20. Recommendation for P80

P80 should calibrate the Level-1 Router policy against the P79 oracle:

- reduce wrong routes for relation-heavy code;
- route high-pressure log prefixes to product typed when update cost dominates;
- reconsider grid3d for sparse CSV projections;
- keep index savings while crossing the `0.97 * hybrid` ratio gate.

## 21. Reproducibility notes

Executed commands:

```bash
cargo run -p atlas-cli -- level1-router-bench \
  --corpus all \
  --level1-router p79-router \
  --target-source-bytes 10485760 \
  --cycles 10 \
  --queries 10000 \
  --updates 1000 \
  --deletes 100 \
  --compact adaptive \
  --adaptive on \
  --compare router,oracle,hybrid,path-trie,product-typed,content-dag \
  --export-dir artifacts/p79/level1_router_standard \
  --format json
```

```bash
cargo run -p atlas-cli -- level1-router-bench \
  --corpus all \
  --level1-router p79-router \
  --target-source-bytes 10485760 \
  --cycles 25 \
  --queries 50000 \
  --updates 5000 \
  --deletes 500 \
  --compact adaptive \
  --adaptive on \
  --compare router,oracle,hybrid,path-trie,product-typed,content-dag \
  --export-dir artifacts/p79/level1_router_ambitious \
  --format json
```

Artifacts are ignored under `artifacts/p79/`.

## 22. Journal

P79 adds `src/p79.rs`, `tests/p79_tests.rs`, a specialized `.atlas` syntax for the Level-1 Router, invalid contracts, reports, validation docs, and Results material.


# ASTRA-P78 — Level-1 Virtual Space and Universal Fiber Store Analysis

## 1. Executive summary

P78 moves the search one layer above fiber topology. It tests the topology of
the level-1 virtual address space and introduces a universal file fiber store.
The best observed level-1 topology is `hybrid_multi_index_space`, with
`ratio_living = 5.340001` on the living-memory 10 MiB protocol. The decision is
`RECALIBRATE_P78_LEVEL1_TOPOLOGY`: hybrid wins ratio, but `path_trie` remains
best for address lookup, so no single topology dominates all objectives.

## 2. Position after P77

P77 reduced wrong-route cost from `2034` to `783`, raised routing accuracy to
`0.974155`, and kept `NO_DRIFT`, guard refusal and reopen equivalence. It still
missed the strict `router/oracle >= 0.985` promotion gate.

P78 keeps the P77 calibrated router as the fiber router and tests whether the
level-1 address space can improve the living ratio without materializing the
global virtual space.

## 3. Central question: topology of level-1 address space

The level-1 address space is not necessarily a 2D grid. It may be a tree, path
trie, content-addressed DAG, graph, typed product space, or hybrid multi-index
space.

Question: which level-1 topology maximizes effective virtual addressability and
`ratio_living` while preserving local-on-address evaluation, CRUD, retrieval,
guard behavior, and honest cost accounting?

## 4. Process principles

P78 follows the project process rules:

- living-memory only for R&D decisions;
- `ratio_living` remains the central metric;
- virtual space is constructed locally when an address is reached;
- virtual byte fields are materialization equivalents, not stored bytes;
- universal file support must not create false gain on incompressible data;
- test stack hygiene is audited;
- LaTeX Results quality is checked and warnings are corrected where reasonable.

## 5. Level1AddressSpace model

`Level1AddressSpace` contains:

- `space_id`;
- `topology_kind`;
- `components`;
- `address_bits`;
- `local_on_address`;
- `materialization`.

The P78 valid contract uses:

`topology=hybrid_multi_index`, `components=path_trie,content_dag,product_typed_space,graph_overlay`,
`address_bits=64`, `local_on_address=true`, and
`materialization=global_forbidden`.

## 6. Topologies compared

P78 compares:

- `grid_2d`;
- `grid_3d`;
- `hierarchical_tree`;
- `path_trie`;
- `content_addressed_dag`;
- `graph_address_space`;
- `product_typed_space`;
- `hybrid_multi_index_space`.

`hybrid_multi_index_space` combines path trie, content DAG, product typed space
and graph overlay. It is useful for universal file storage because it can route
paths, chunks, file types and relations without pretending that a single index
structure is optimal.

## 7. VirtualSpaceEstimator

The P78 estimator computes:

- `level1_declared_address_count`;
- `level1_reachable_address_count`;
- `level1_effective_address_count`;
- `virtual_cell_count`;
- `virtual_fiber_count`;
- `virtual_chunk_count`;
- `virtual_version_count`;
- `virtual_declared_bytes_equivalent`;
- `virtual_effective_bytes_equivalent`;
- `materialization_avoidance_ratio`;
- `addressability_ratio`;
- `level1_density`;
- `level1_index_bytes`;
- `max_computable_addresses_under_budget`;
- `limiting_factor`.

The estimate for the P78 hybrid run is:

```text
level1_declared_address_count       : 281,600,000,000
level1_reachable_address_count      : 247,808,000,000
level1_effective_address_count      : 222,464,000,000
virtual_cell_count                  : 10,000
virtual_fiber_count                 : 40,000
virtual_chunk_count                 : 40,000
virtual_version_count               : 4
virtual_effective_bytes_equivalent  : 96,415,629
bytes_are_equivalent_not_stored     : true
```

## 8. Limiting factors of virtual space size

The size of virtual space is not limited only by the number of addresses. It is
limited by local `Eval` cost, index size, generator cost, runtime memory,
journal replay, audit cost, residual bytes, raw fallback and guard refusal.

For the best P78 topology the limiting factor is `index_size`: the reachable
space is bounded by level-1 indexes and local lookup cost.

## 9. Universal file support

P78 introduces `UniversalFiberStore` and `FileTypeClassifier`. The store accepts:

- text/code;
- JSON;
- CSV;
- logs;
- image-like deterministic binary;
- video-like deterministic chunks;
- arbitrary binary;
- unknown extension.

Unknown or arbitrary files use explicit `raw_fallback`. Raw fallback is accepted
as storage, but it is not credited as procedural gain.

## 10. Encoding/decoding pipeline

The pipeline is:

`file -> classify -> split into chunks/fibers -> choose codec -> map into level1 address space -> encode -> open -> lookup -> read/query -> update/delete -> audit -> compact -> close -> reopen -> verify`.

Codecs used by the deterministic corpus:

- `grammar_token`;
- `json_path`;
- `csv_sparse`;
- `text_dictionary`;
- `chunk_dedup`;
- `content_dag`;
- `raw_fallback`;
- `refused_guard`.

## 11. Living-memory benchmark

Standard protocol:

- source bytes: `10,485,760`;
- cycles: `10`;
- queries: `10,000`;
- updates: `1,000`;
- deletes: `100`;
- compact: `adaptive`;
- adaptive: `on`.

Ambitious protocol:

- source bytes: `10,485,760`;
- cycles: `25`;
- queries: `50,000`;
- updates: `5,000`;
- deletes: `500`;
- compact: `adaptive`;
- adaptive: `on`.

Both runs include encode, open, address lookup, read/query, update, delete,
audit, compact, close, reopen, and guard verification.

## 12. Addressing and CRUD metrics

For the best topology:

- `address_lookup_p95_steps = 8.000`;
- `address_lookup_bytes_read_p95 = 4608`;
- `address_lookup_success_rate = 1.0`;
- `CRUD success rate = 1.0`;
- collisions and hash collisions are `0`.

No timing is goldenized. The reported costs are deterministic units and byte
reads, not machine-dependent latency gates.

## 13. Standard campaign results

P78 level-1 virtual space view:

```text
target source bytes                 : 10,485,760
actual source bytes                 : 10,485,760
best level-1 topology               : hybrid_multi_index_space
virtual address count               : 222,464,000,000 effective addresses
virtual cell count                  : 10,000
virtual fiber count                 : 40,000
virtual effective bytes equivalent  : 96,415,629
limiting factor                     : index_size
cold persisted bytes                : 1,268,502
runtime peak bytes                  : 559,633
ratio_living                        : 5.340001
address lookup p95 steps            : 8.000
CRUD success rate                   : 1.000000
raw fallback bytes                  : 327,680
guard decision                      : NO_GO_GUARD_INCOMPRESSIBLE_REFUSED
decision                            : RECALIBRATE_P78_LEVEL1_TOPOLOGY
```

Phase summary:

- green: `4`;
- yellow: `2`;
- red: `2`;
- best by ratio: `hybrid_multi_index_space`;
- best by address lookup: `path_trie`;
- best by universal codec: `hybrid_multi_index_space`.

## 14. Ambitious campaign results if executed

The ambitious run was executed. It uses larger cycle/query/update/delete counts
and produces the same structural result:

- best topology: `hybrid_multi_index_space`;
- `ratio_living = 5.340001`;
- `cold_persisted_bytes = 1,268,502`;
- `runtime_peak_bytes = 559,633`;
- `reopen_equivalence = true`;
- `drift_status = NO_DRIFT`;
- `guard_decision = NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`.

## 15. Universal codec results

Universal codec summary:

```text
file_count                 : 640
extension_count            : 9
raw_fallback_count         : 1
raw_fallback_bytes         : 327,680
exact_roundtrip_rate       : 1.000000
decode_success_rate        : 1.000000
retrieval_success_rate     : 1.000000
update_success_rate        : 1.000000
guard_source_bytes         : 524,288
guard_store_bytes          : 0
guard_no_false_gain        : true
```

The raw fallback path is explicit and counted. It does not create a high-ratio
success claim.

## 16. Guard incompressible

The incompressible guard remains refused:

`NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`.

The guard contributes no false gain. Its source bytes are counted as a guard
input, and `guard_store_bytes = 0` for the refused path.

## 17. Ratio/gain view

P78 improves the best living ratio relative to P77:

- P77 calibrated router: `4.998098`;
- P78 best level-1 topology: `5.340001`.

This is not an information-theoretic claim. The gain comes from better level-1
addressing, local lookup, universal codec routing and reduced materialization
overhead.

## 18. Impact on .atlas

P78 adds specialized declarative lines:

```text
level1_address_space ...
universal_fiber_store ...
virtual_space_gates ...
```

The syntax remains intentionally narrow. It does not add loops, arbitrary
functions, execution or general-purpose language features.

## 19. Decision

`RECALIBRATE_P78_LEVEL1_TOPOLOGY`.

Reason: `hybrid_multi_index_space` wins ratio and universal codec coverage, but
`path_trie` is still the best address lookup topology. A level-1 router or
per-file-class level-1 policy should be tested before promotion.

## 20. Limitations

- The universal binary corpora are deterministic local surrogates, not external
  image/video datasets.
- The benchmark writes measurable filesystem artifacts, but the source data is
  generated/expanded locally.
- The level-1 topology search is broad but still coarse.
- Raw fallback is explicit and counted, but P79 should stress larger arbitrary
  binary mixtures.

## 21. Recommendation for P79

P79 should test a level-1 address-space router:

- path trie for paths and lookup-heavy files;
- content DAG for chunked binary and deduplication;
- product typed space for namespace/file-type/object/chunk/version indexing;
- graph overlay for relationships;
- hybrid multi-index as the default only when its overhead is justified.

## 22. Reproducibility notes

Commands:

```bash
cargo run -p atlas-cli -- level1-space-bench --corpus all --level1-topology all --fiber-router p77-calibrated --target-source-bytes 10485760 --cycles 10 --queries 10000 --updates 1000 --deletes 100 --compact adaptive --adaptive on --export-dir artifacts/p78/level1_space_standard --format json

cargo run -p atlas-cli -- level1-space-estimate --level1-topology hybrid-multi-index --target-source-bytes 10485760 --address-bits 64 --file-type-count 16 --object-count 10000 --chunk-count 40000 --version-count 4 --fibers-per-object 4 --format json
```

Artifacts are local-only and ignored under `artifacts/p78/`.

## 23. Journal

- Added `src/p78.rs`.
- Added `level1-space-bench` and `level1-space-estimate`.
- Added P78 valid and invalid `.atlas` examples.
- Added `tests/p78_tests.rs`.
- Updated the invalid corpus from 84 to 92 cases.
- Generated P78 standard and ambitious living-memory artifacts.

# ASTRA-P78 — Level-1 Virtual Space and Universal Fiber Store

P78 adds the level-1 address-space layer above fiber topology. It compares
`grid_2d`, `grid_3d`, `hierarchical_tree`, `path_trie`,
`content_addressed_dag`, `graph_address_space`, `product_typed_space`, and
`hybrid_multi_index_space`.

## Commands

```bash
cargo run -p atlas-cli -- level1-space-bench \
  --corpus all \
  --level1-topology all \
  --fiber-router p77-calibrated \
  --target-source-bytes 10485760 \
  --cycles 10 \
  --queries 10000 \
  --updates 1000 \
  --deletes 100 \
  --compact adaptive \
  --adaptive on \
  --export-dir artifacts/p78/level1_space_standard \
  --format json

cargo run -p atlas-cli -- level1-space-estimate \
  --level1-topology hybrid-multi-index \
  --target-source-bytes 10485760 \
  --address-bits 64 \
  --file-type-count 16 \
  --object-count 10000 \
  --chunk-count 40000 \
  --version-count 4 \
  --fibers-per-object 4 \
  --format json
```

## .atlas syntax

P78 uses specialized key/value lines:

```text
atlas version=0.1;
p78_level1_probe mode=level1_virtual_space;
level1_address_space id=universal_space_v1 topology=hybrid_multi_index components=path_trie,content_dag,product_typed_space,graph_overlay address_bits=64 local_on_address=true materialization=global_forbidden;
universal_fiber_store id=universal_store_v1 accept_any_file=true raw_fallback=explicit guard_no_false_gain=true codec_selection=entropy_and_structure;
virtual_space_gates virtual_space_metrics_required=true virtual_bytes_claim=equivalent ratio_living_primary=true address_lookup_bounded=true local_materialization_only=true guard_no_false_gain=true hidden_level1_overhead=false;
```

This is not a general-purpose `.atlas` expansion. It is a declarative contract
for level-1 address-space evaluation.

## Expected local outcome

- best topology: `hybrid_multi_index_space`;
- `ratio_living = 5.340001`;
- `address_lookup_p95_steps = 8.000`;
- `CRUD success rate = 1.0`;
- raw fallback explicit and counted;
- guard refused with no false gain;
- `reopen_equivalence = true`;
- `drift_status = NO_DRIFT`;
- decision: `RECALIBRATE_P78_LEVEL1_TOPOLOGY`.

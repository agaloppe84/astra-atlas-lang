# ASTRA-P79 - Level-1 Address Space Router Validation

P79 adds a living-memory Level-1 Address Router. It chooses the level-1 address-space topology per local file/address feature rather than selecting a single global topology.

## CLI

Standard campaign:

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

Estimate:

```bash
cargo run -p atlas-cli -- level1-router-estimate \
  --level1-router p79-router \
  --target-source-bytes 10485760 \
  --format json
```

## .atlas Syntax

P79 uses a specialized declarative syntax:

```atlas
atlas version=0.1;
p79_level1_router_probe mode=level1_address_router;
level1_router id=p79-router default=product_typed_space path_like=path_trie chunked_binary=content_addressed_dag typed_namespace=product_typed_space relation_heavy=graph_address_space multi_access=hybrid_multi_index_space regular_grid=grid3d guard_policy=refuse_or_raw_no_gain;
level1_router_gates living_memory_only=true ratio_living_primary=true virtual_space_metrics_required=true local_on_address=true guard_no_false_gain=true hidden_level1_index_storage=false address_lookup_bounded=true router_oracle_ratio_min=0.97 virtual_bytes_claim=equivalent;
```

No loops, functions, arbitrary execution, or general-purpose language features are added.

## Required Invariants

- virtual bytes are materialization equivalents, not stored bytes;
- local-on-address construction is mandatory;
- guard data is refused or raw no-gain;
- hidden level-1 index storage is refused;
- address lookup must be bounded;
- living-memory-only gates are mandatory for R&D decisions.

## Current Result

P79 standard result:

- `ratio_living_router = 5.174901`
- `ratio_living_hybrid_only = 5.340001`
- `router/hybrid = 0.969082`
- `lookup_p95_router = 7.800`
- `lookup_p95_path_trie = 7.000`
- `index_saved_vs_hybrid = 262,144`
- `CRUD = 1.000000`
- `retrieval = 1.000000`
- `reopen_equivalence = true`
- `drift = NO_DRIFT`
- `guard = NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`

Decision: `RECALIBRATE_P79_LEVEL1_ROUTER_POLICY`.


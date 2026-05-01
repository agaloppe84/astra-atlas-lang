# ASTRA-P75 — Mixed-Topology Fiber Router

P75 introduces `mixed-topology-bench`, a living-memory benchmark for a router
that chooses the local fiber topology from observed structure.

## Command

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

Ambitious:

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

## Router policy

- code path-heavy fibers route to `trie_prefix_fiber`;
- code relation-heavy fibers route to `graph_adjacency_fiber`;
- logs tag-heavy fibers route to `hypergraph_tag_fiber`;
- JSON path-heavy fibers route to `trie_prefix_fiber`;
- sparse CSV update-heavy fibers route to `baseline_linear_fiber`;
- sparse CSV tiles route to `hierarchical_tile_fiber`;
- guard fibers are refused or raw/no-go and never credited as success.

## Exports

```text
artifacts/p75/<campaign>/
  p75_mixed_topology_report.json
  p75_router_decisions.jsonl
  p75_topology_comparison.csv
  p75_phase_map.csv
  p75_cost_breakdown.csv
  p75_summary.md
  topology_stores/
    mixed-router/
    hierarchical-only/
    linear-only/
    cubical-only/
```

These artifacts are ignored by Git.

## Decision rule

P75 remains living-memory only for architectural decisions: about 10 MiB source
data, open/read/query/update/delete/audit/compact/close/reopen, guard refusal,
measured filesystem costs and `ratio_living`.

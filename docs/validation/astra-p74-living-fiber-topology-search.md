# ASTRA-P74 — Living Fiber Topology Search

P74 compares fiber topologies in living-memory mode. Architectural decisions on
the virtual/real memory ratio now require a meaningful living campaign around
10 MiB of deterministic source data. Unit tests still protect parsing,
typechecking and non-regression, but they do not justify the P74 architecture
decision by themselves.

## Command

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

Ambitious:

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

## Topologies

- `baseline_linear_fiber`
- `cubical_6face_fiber`
- `trie_prefix_fiber`
- `graph_adjacency_fiber`
- `hypergraph_tag_fiber`
- `hierarchical_tile_fiber`

## Corpora

The source corpus is local and deterministic. For `--corpus all`, P74 splits the
target byte budget across:

- `real_code_corpus_10m`
- `realish_logs_10m`
- `realish_json_10m`
- `sparse_csv_10m`
- `incompressible_guard_10m`

The corpus names include `10m` because the full selected dataset targets about
10 MiB. The code corpus may be expanded deterministically; it is not an
external dataset.

## Exports

Generated files stay ignored under `artifacts/p74/`:

```text
artifacts/p74/<campaign>/
  p74_topology_living_report.json
  p74_topology_results.jsonl
  p74_phase_map.csv
  p74_cost_breakdown.csv
  p74_summary.md
  topology_stores/
    <topology>/<corpus>/cold/
    <topology>/<corpus>/runtime/
    <topology>/<corpus>/reports/
```

## Local result summary

Standard:

- target_source_bytes = 10,485,760
- actual_source_bytes = 10,485,760
- best_topology_overall = `hierarchical_tile_fiber`
- best_topology_by_ratio_living = `hierarchical_tile_fiber`
- best_topology_by_retrieval = `hierarchical_tile_fiber` (retrieval tie at 1.0 in this run)
- best_topology_by_update_cost = `baseline_linear_fiber`
- best_ratio_living = 4.742439
- phase map = 163 green / 125 yellow / 72 red
- guard = `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`
- reopen_equivalence = true
- drift_status = `NO_DRIFT`
- decision = `RECALIBRATE_P74_FIBER_TOPOLOGY_SEARCH`

Ambitious:

- target_source_bytes = 10,485,760
- actual_source_bytes = 10,485,760
- best_topology_overall = `hierarchical_tile_fiber`
- best_topology_by_ratio_living = `hierarchical_tile_fiber`
- best_topology_by_retrieval = `hierarchical_tile_fiber` (retrieval tie at 1.0 in this run)
- best_topology_by_update_cost = `baseline_linear_fiber`
- best_ratio_living = 4.742450
- phase map = 163 green / 125 yellow / 72 red
- guard = `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`
- reopen_equivalence = true
- drift_status = `NO_DRIFT`
- decision = `RECALIBRATE_P74_FIBER_TOPOLOGY_SEARCH`

## Decision rule

P74 does not claim that any topology stores more information per bit. A topology
only helps when it matches structure already present in the data. Promotion is
withheld because the observed winner is corpus-dependent: graph adjacency wins
for code, hypergraph tags win for logs, trie prefixes win for JSON, and
hierarchical tiles win for sparse CSV.

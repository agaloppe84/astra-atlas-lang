# Atlas Language Spec P76

## Status

This is a declarative, specialized snapshot. `.atlas` must not become a
general-purpose or Turing-complete language.

## Accepted Families Before P76

The language keeps the specialized blocks introduced by P69-P75:

- representation contracts;
- address/fiber schemas;
- actor policies;
- lifecycle gates;
- cubical topology gates;
- living topology gates;
- topology routers.

## P76 Blocks

P76 adds three declarative blocks.

```atlas
routing_oracle id=oracle_v1
  compare=oracle,mixed,hierarchical,trie,graph,hypergraph,linear
  regret_metric=ratio_living_and_update_cost
  wrong_route_budget=controlled
  hidden_router_overhead=false;

virtual_space_model id=living_10m_v1
  target_source_bytes=10485760
  virtual_metric=effective_bytes_equivalent
  materialization_avoidance=measured
  local_on_address=true
  virtual_space_metrics_required=true
  virtual_bytes_claim=equivalent;

astra_process_gates living_memory_only=true
  ratio_living_primary=true
  procedural_virtual_space_local=true
  virtual_space_metrics_required=true
  guard_no_false_gain=true
  hidden_router_overhead=false
  ratio_living_reported=true;
```

The optional `p76_process_probe` line is a parser probe used by tests and the
legacy strict parser invalid corpus.

## Typecheck Rules

- `compare` entries must be known router/oracle/topology targets.
- `regret_metric` must be `ratio_living_and_update_cost`.
- `wrong_route_budget` must be `controlled`.
- hidden router overhead must be false.
- `target_source_bytes` must be at least 10,485,760.
- `virtual_metric` must be `effective_bytes_equivalent`.
- `virtual_bytes_claim` must be `equivalent`, never `stored`.
- `local_on_address`, `living_memory_only`, `ratio_living_primary`,
  `procedural_virtual_space_local`, `virtual_space_metrics_required`,
  `guard_no_false_gain` and `ratio_living_reported` must be true.

## Forbidden Expansions

No loops, arbitrary functions, runtime code execution, mutable global memory, or
general-purpose control flow are introduced by P76.

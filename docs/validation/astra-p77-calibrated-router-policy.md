# ASTRA-P77 — Calibrated Router Policy

Policy id: `p77_calibrated_router_v1`

The retained policy is deterministic and local. It adjusts the P75/P76 mixed
router toward the oracle without adding opaque learning or global
materialization.

## Parameters

- `confidence_threshold`: `0.50`
- `fallback_threshold`: `0.20`
- `hierarchy_bias`: `0.92`
- `linear_update_bias`: `1.20`
- `trie_prefix_bias`: `1.05`
- `graph_relation_bias`: `1.15`
- `hypergraph_tag_bias`: `1.10`
- `guard_threshold`: `strict`

## Rationale

The policy reduces overuse of hierarchical fallback, strengthens linear routing
for update-heavy sparse cells, and raises graph/hypergraph preference where the
P76 oracle observed lower wrong-route regret.

## Observed P77 Standard Result

- `ratio_living_calibrated_router`: `4.998098`
- `ratio_living_oracle`: `5.076273`
- router/oracle ratio: `0.984600`
- routing accuracy: `0.974155`
- wrong-route count: `570 -> 214`
- wrong-route cost: `2034 -> 783`
- guard: `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`
- reopen equivalence: `true`
- drift: `NO_DRIFT`

## Limits

The policy is a calibrated candidate, not a final promotion. P77 keeps the
decision conservative unless the living-memory run satisfies all promotion gates.
The current policy misses the strict router/oracle promotion threshold by
`0.000400`.

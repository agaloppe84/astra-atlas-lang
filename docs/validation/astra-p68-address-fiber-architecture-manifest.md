# ASTRA-P68 — Address-Fiber Architecture Manifest

Ce manifeste est la version committable compacte du manifeste JSON P68 genere
localement sous `artifacts/p68/`.

## Architecture

- `architecture_id`: `address_fiber_actor_managed_v1`
- `promotion_status`: `promoted_for_p69`
- `default_workload_family`: `realish_hybrid_field_fixture`
- `default_radius`: `1`
- `default_budget_bytes`: `4194304`
- `cache_policy`: `compact`
- `journal_policy`: `compact`
- `audit_policy`: `minimal`
- `compaction_policy`: `threshold_or_aggressive_by_query_pressure`
- `metadata_policy`: `standard`
- `fiber_projection_depth`: `shallow`

## Expected local ranges

- `expected_overhead_range`: `0.119345-0.123446`
- `expected_net_gain_range`: `13.335472-17.379955`

These ranges come from the local P67/P68 deterministic standard+ambitious pair.
They are not general scientific guarantees.

## Known failure modes

- small budgets create clean budget refusals;
- large radius can move the profile out of the green phase;
- random locality lowers cache reuse;
- verbose metadata increases overhead;
- local/global conflict must be refused.

## Required gates for P69

- keep conflicts and stale reads at zero;
- keep `budget_refusal_rate < 0.02`;
- preserve update/audit/compaction accounting;
- keep cache/journal/audit/metadata/coordination costs visible;
- repeat with external or multi-machine fixtures before scientific validation.

## Recommended next step

P69 should implement `address_fiber_actor_managed_v1` as a guarded runtime
default candidate and extend multi-fixture replay.

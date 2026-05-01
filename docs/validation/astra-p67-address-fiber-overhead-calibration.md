# ASTRA-P67 — Address-Fiber Overhead Calibration

P67 calibre l'overhead du modele `actor_managed_fiber` introduit en P66.
L'objectif est de verifier si le gain `address_fiber_net_gain` peut rester fort
tout en ramenant `avg_actor_overhead_ratio` sous les seuils candidats, sans
conflits, stale reads ou refus budgetaires dominants.

P67 ne change pas la grammaire `.atlas`, ne modifie pas `strict_p53`, ne retire
aucun invalide et ne goldenise aucun timing.

## Commande principale

```bash
cargo run -p atlas-cli -- ratio-fibers-calibrate examples/p53_strict.atlas \
  --workload all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --radius-grid 1,2,3,5 \
  --budget-grid 524288,1048576,2097152,4194304 \
  --cache-grid on,compact \
  --journal-grid lazy,compact \
  --audit-grid minimal,sampled \
  --compaction-grid threshold,aggressive \
  --query-locality-grid clustered,mixed \
  --fiber-projection-grid shallow,medium \
  --export-dir artifacts/p67/calibration_standard \
  --format json
```

Les exports de campagne sont compacts mais restent ignores par Git:

- `p67_fiber_calibration_report.json`
- `p67_fiber_calibration_runs.jsonl`
- `p67_fiber_calibration_grid.csv`
- `p67_fiber_calibration_summary.md`

## Parametres calibres

P67-v1 calibre:

- `neighborhood_radius`;
- `budget_bytes`;
- `cache_policy`;
- `journal_policy`;
- `audit_policy`;
- `compaction_policy`;
- `query_locality`;
- `fiber_projection_depth`.

P67-v1 garde `update_rate` et `metadata_policy` comme valeurs deterministes
standard. Ces dimensions restent candidates pour P67-2/P68.

## Metriques

Chaque configuration expose:

- `fiber_ratio_effective_per_byte`;
- `address_fiber_net_gain`;
- `avg_actor_overhead_ratio`;
- `actor_overhead_bytes`;
- bytes cache/journal/audit/metadata;
- `update_count`, `audit_count`, `compaction_count`;
- `conflicts`, `stale_reads`, `budget_refusals`;
- `cache_hit_rate`;
- `bytes_per_query`;
- `balanced_score`;
- `promotion_candidate`.

Le score equilibre est heuristique:

```text
balanced_score =
  address_fiber_net_gain
  * max(0, 1 - avg_actor_overhead_ratio)
  * cache_factor
  * compaction_factor
  * safety_factor
```

Il sert a classer les configurations, pas a etablir une loi scientifique.

## Decisions

Decisions P67 possibles:

- `PROMOTE_P67_ADDRESS_FIBER_ARCHITECTURE`
- `RECALIBRATE_P67_FIBER_OVERHEAD`
- `NO_GO_P67_ADDRESS_FIBER_OVERHEAD`

Dans P67-v1, un rapport local isole peut produire des configurations candidates,
mais la decision globale reste conservatrice:

```text
RECALIBRATE_P67_FIBER_OVERHEAD
```

Une promotion exige une lecture conjointe standard + ambitious et reste exclue
des tests timing-dependants ou des goldens.

## Validation locale

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p67_tests
cargo test --test p66_tests
bash scripts/validate_p58_local.sh
```

Les campagnes longues restent locales. La CI GitHub reste minimale et ne porte
pas la calibration scientifique.

# ASTRA-P68 — Address-Fiber Promotion Gate

P68 transforme le signal P67 en decision architecturale explicite. Le sprint ne
relance pas seulement un benchmark: il ajoute un evaluateur code qui paire le
meilleur candidat standard et le meilleur candidat ambitious, puis verifie des
gates de promotion stricts.

## Objectif

P68 decide si `address_fiber_actor_managed_v1` peut devenir le candidat
architectural principal pour P69.

La promotion n'est autorisee que si:

- le candidat standard passe les seuils overhead/gain/surete;
- le candidat ambitious passe les seuils overhead/gain/surete;
- les deux candidats sont comparables;
- les metriques update/audit/compaction sont presentes;
- les conflits, stale reads et refus budgetaires restent nuls ou sous seuil;
- les couts cache, journal, audit, metadata et compaction restent comptes.

## Commande principale

```bash
cargo run -p atlas-cli -- ratio-fibers-promote examples/p53_strict.atlas \
  --run-ablations \
  --run-stress \
  --phase-map \
  --export-dir artifacts/p68/promotion_gate \
  --format json
```

La commande regenere en interne les candidats P67 standard et ambitious
deterministes, puis produit le rapport P68.

## Exports locaux

Les exports P68 restent ignores par Git:

- `p68_promotion_report.json`
- `p68_ablations.jsonl`
- `p68_stress.jsonl`
- `p68_phase_map.csv`
- `p68_summary.md`
- `address_fiber_architecture_manifest.json`

## Decisions

Decisions P68 possibles:

- `PROMOTE_P68_ADDRESS_FIBER_ARCHITECTURE`
- `RECALIBRATE_P68_PROMOTION_GATE`
- `NO_GO_P68_ADDRESS_FIBER_ARCHITECTURE`

`PROMOTE_P68_ADDRESS_FIBER_ARCHITECTURE` signifie promotion repo/runtime vers un
candidat P69. Cela ne signifie pas validation scientifique finale: P69 doit
encore implementer les gates comme comportement runtime garde et preparer des
replays multi-fixtures/multi-machines.

## Validation locale

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p68_tests
cargo test --test p67_tests
bash scripts/validate_p58_local.sh
```

Les campagnes longues restent locales. La CI GitHub reste une sanity minimale et
ne porte pas la promotion scientifique.

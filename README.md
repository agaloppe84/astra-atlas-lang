# ASTRA Atlas Lang

Prototype P55 d'un mini-langage `.atlas` pour programmes mémoire procéduraux ASTRA.

## Commandes

```bash
cargo test --workspace
cargo run -p atlas-cli -- check examples/p53_strict.atlas
cargo run -p atlas-cli -- explain E_GUARD_ACTIVE
cargo run -p atlas-cli -- export examples/p53_strict.atlas --format json
cargo run -p atlas-cli -- run examples/p53_strict.atlas --mode smoke
cargo run -p atlas-cli -- run examples/p53_strict.atlas --mode standard
cargo run --bin atlas-cli -- metrics examples/p53_strict.atlas --format json
cargo run -p atlas-cli -- metrics examples/p53_strict.atlas --mode standard --format json
cargo run -p atlas-cli -- report examples/p53_strict.atlas --format json
cargo run -p atlas-cli -- report examples/p53_strict.atlas --mode standard --format markdown
cargo run -p atlas-cli -- bench --mode smoke
cargo run -p atlas-cli -- bench --mode standard
cargo run -p atlas-cli -- bench --mode standard --format json
cargo run -p atlas-cli -- ratio examples/p53_strict.atlas --mode smoke --format json
cargo run -p atlas-cli -- ratio-real examples/p53_strict.atlas --mode smoke --format json --runs 3
cargo run -p atlas-cli -- ratio-real examples/p53_strict.atlas --mode smoke --format json --runs 5 --export-dir artifacts/p63/smoke --threshold-profile p63
cargo run -p atlas-cli -- ratio-campaign-compare artifacts/p63/smoke/campaign_report.json artifacts/p63/standard/campaign_report.json --format json
```

After `cargo build`, the same CLI is available as `atlas-cli`. The `atlasc`
binary name is kept as a compatibility alias.

## Codespaces

1. Créer un repo GitHub `astra-atlas-lang`.
2. Copier ce dossier à la racine du repo.
3. Ouvrir `Code -> Codespaces -> Create codespace on main`.
4. Le devcontainer Rust installe l'environnement et lance `cargo test --workspace`.

## Objectif P55

Le but n'est pas de créer un langage généraliste, mais de figer un format testable :
parser, typechecker, runtime minimal, refus des mutants, et équivalence des invariants P53/P54.1.

## Validation P58

P58 ajoute des workloads runtime deterministes `smoke`, `standard` et
`ambitious`, ainsi que des rapports JSON/Markdown stables. La validation locale
complete se lance avec:

```bash
bash scripts/validate_p58_local.sh
```

Voir [docs/validation_p58.md](docs/validation_p58.md) pour le detail des gates,
des decisions attendues et du perimetre CI.

## Validation P59 classique

P59 interprete les acquis P58 comme invariants ASTRA classiques, sans nouveau
runtime ni changement de grammaire `.atlas`. Voir
[docs/validation_p59_classique.md](docs/validation_p59_classique.md).

## Validation P60 systeme

P60 nettoie les entrees CLI, specialise les diagnostics invalides et ajoute une
trace JSON de benchmark structurel. Voir
[docs/validation_p60_sys_cleanup.md](docs/validation_p60_sys_cleanup.md).

## Validation P61 ratio virtuel

P61 introduit un laboratoire deterministe et conservateur pour le ratio virtuel
effectif `virtual_effective / real_total_cost_units`. Le modele de cout reste un
proxy, sans revendication de validation scientifique. Voir
[docs/validation_p61_virtual_ratio_lab.md](docs/validation_p61_virtual_ratio_lab.md).

## Validation P62 mesure reelle

P62 ajoute une commande locale `ratio-real --runs N` qui mesure des timings
`Instant` et des tailles de fichiers temporaires reelles, sans golden de timing. Voir
[docs/validation/astra-p62-real-measurement-plan.md](docs/validation/astra-p62-real-measurement-plan.md).

## ASTRA-P63 analysis reports

P63 prepare la calibration scientifique du ratio mesure. Les rapports d'analyse
Markdown committables vivent dans [docs/analysis/](docs/analysis/) et doivent
conserver les commandes, resultats resumes, decisions, limites et recommandations
suivantes sans stocker de gros logs.

Le premier export compact de campagne P63 s'active avec
`ratio-real --threshold-profile p63 --export-dir <path>` et produit
`campaign_report.json`, `runs.jsonl`, `runs.csv` et `summary.md` localement.
Le profil `p63` resout vers `p63_conservative_v1`, et les campagnes peuvent etre
comparees avec `ratio-campaign-compare`.

Prompt P63 ajoute aussi un registre local ignore par Git:
`ratio-campaign-register <campaign_report.json> --registry artifacts/p63/registry.json --name <name> --format json`,
puis `ratio-campaign-summary artifacts/p63/registry.json --format json`.
Les campagnes standard comparables peuvent etre synthetisees avec
`ratio-campaign-set-summary artifacts/p63/registry.json --mode standard --threshold-profile p63 --format json`.
P63-v6 expose aussi les metriques coeur `virtual_*_units`,
`ratio_effective_per_byte` et `gain_vs_materialized` dans les rapports de
campagne et les syntheses de campaign set.

## Validation P64 address-local

P64 ajoute `ratio-realish` pour comparer des fixtures realish legeres avec les
politiques `full-materialization`, `global-indexed` et `address-local`. Le but
est de mesurer si le runtime paie seulement le voisinage local autour d'une
adresse demandee, sans materialiser tout l'espace virtuel.

```bash
cargo run -p atlas-cli -- ratio-realish examples/p53_strict.atlas \
  --workload all \
  --policy all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --neighborhood-radius 3 \
  --export-dir artifacts/p64/realish_standard_policy_compare \
  --format json
```

Les exports P64 vivent sous `artifacts/p64/` et restent ignores par Git. Voir
[docs/validation/astra-p64-address-local-realish-workloads.md](docs/validation/astra-p64-address-local-realish-workloads.md)
et le rapport d'analyse
[docs/analysis/ASTRA-P64-address-local-realish-ratio-analysis.md](docs/analysis/ASTRA-P64-address-local-realish-ratio-analysis.md).

## Validation P65 acteurs locaux

P65 ajoute `ratio-actors` pour tester des acteurs locaux deterministes a budget
sur les workloads realish P64. Les acteurs ne sont pas une intelligence
autonome: cache, index, journal, queue, audit et coordination sont tous comptes
comme cout reel.

```bash
cargo run -p atlas-cli -- ratio-actors examples/p53_strict.atlas \
  --workload all \
  --actor-strategy all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --neighborhood-radius 3 \
  --budget-bytes 1048576 \
  --export-dir artifacts/p65/actors_standard \
  --format json
```

Les exports P65 vivent sous `artifacts/p65/` et restent ignores par Git. Voir
[docs/validation/astra-p65-address-local-actor-runtime.md](docs/validation/astra-p65-address-local-actor-runtime.md)
et le rapport d'analyse
[docs/analysis/ASTRA-P65-address-local-actor-runtime-analysis.md](docs/analysis/ASTRA-P65-address-local-actor-runtime-analysis.md).

## Validation P65-2 calibration overhead acteur

P65-2 ajoute `ratio-actors-calibrate` pour explorer une grille locale de
parametres `single_local_actor`: rayon, budget, cache, journal et localite des
requetes. Le but est d'identifier une zone Pareto sans promouvoir trop tot
l'architecture finale.

```bash
cargo run -p atlas-cli -- ratio-actors-calibrate examples/p53_strict.atlas \
  --workload all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --radius-grid 1,2,3,5 \
  --budget-grid 262144,524288,1048576,2097152 \
  --cache-grid off,on \
  --journal-grid lazy,compact \
  --query-locality-grid clustered,random,mixed \
  --export-dir artifacts/p65/calibration_standard \
  --format json
```

Les exports restent sous `artifacts/p65/` et sont ignores par Git. Voir
[docs/validation/astra-p65-2-local-actor-overhead-calibration.md](docs/validation/astra-p65-2-local-actor-overhead-calibration.md)
et le rapport d'analyse
[docs/analysis/ASTRA-P65-2-local-actor-overhead-calibration-analysis.md](docs/analysis/ASTRA-P65-2-local-actor-overhead-calibration-analysis.md).

## Validation P66 address-fiber

P66 ajoute `ratio-fibers` pour formaliser l'espace virtuel comme base
d'adresses avec fibres locales. Les strategies comparees sont `point-fiber`,
`neighborhood-fiber`, `actor-fiber` et `actor-neighborhood-fiber`.

```bash
cargo run -p atlas-cli -- ratio-fibers examples/p53_strict.atlas \
  --workload all \
  --fiber-strategy all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --neighborhood-radius 3 \
  --budget-bytes 2097152 \
  --cache on \
  --journal compact \
  --export-dir artifacts/p66/fibers_standard \
  --format json
```

Les exports restent sous `artifacts/p66/` et sont ignores par Git. Voir
[docs/validation/astra-p66-address-fiber-local-actor-runtime.md](docs/validation/astra-p66-address-fiber-local-actor-runtime.md)
et le rapport d'analyse
[docs/analysis/ASTRA-P66-address-fiber-local-actor-runtime-analysis.md](docs/analysis/ASTRA-P66-address-fiber-local-actor-runtime-analysis.md).

## Validation P67 address-fiber overhead calibration

P67 ajoute `ratio-fibers-calibrate` pour calibrer l'overhead
`actor_managed_fiber` sans masquer les couts de cache, journal, audit,
metadata ou compaction.

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

Les exports restent sous `artifacts/p67/` et sont ignores par Git. Voir
[docs/validation/astra-p67-address-fiber-overhead-calibration.md](docs/validation/astra-p67-address-fiber-overhead-calibration.md)
et le rapport d'analyse
[docs/analysis/ASTRA-P67-address-fiber-overhead-calibration-analysis.md](docs/analysis/ASTRA-P67-address-fiber-overhead-calibration-analysis.md).

## Validation P68 address-fiber promotion gate

P68 ajoute `ratio-fibers-promote`, un evaluateur de promotion pairant les
candidats P67 standard et ambitious. Il produit ablations, stress cible, phase
map et manifeste d'architecture pour P69.

```bash
cargo run -p atlas-cli -- ratio-fibers-promote examples/p53_strict.atlas \
  --run-ablations \
  --run-stress \
  --phase-map \
  --export-dir artifacts/p68/promotion_gate \
  --format json
```

Les exports restent sous `artifacts/p68/` et sont ignores par Git. Voir
[docs/validation/astra-p68-address-fiber-promotion-gate.md](docs/validation/astra-p68-address-fiber-promotion-gate.md),
le manifeste compact
[docs/validation/astra-p68-address-fiber-architecture-manifest.md](docs/validation/astra-p68-address-fiber-architecture-manifest.md)
et le rapport d'analyse
[docs/analysis/ASTRA-P68-address-fiber-promotion-gate-analysis.md](docs/analysis/ASTRA-P68-address-fiber-promotion-gate-analysis.md).

## Validation P69 representation contract

P69 ajoute un contrat `.atlas` specialise pour expliciter ce qui est stocke par
l'architecture `address_fiber_actor_managed_v1`: generateur, parametres,
dictionnaire/ROM, index, residus, journal, cache, etat acteur, audit metadata,
manifest et safety metadata.

```bash
cargo run -p atlas-cli -- contract-check examples/valid/p69_address_fiber_contract.atlas --format json
cargo run -p atlas-cli -- contract-run examples/valid/p69_address_fiber_contract.atlas \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --export-dir artifacts/p69/contract_standard \
  --format json
```

Les exports restent sous `artifacts/p69/` et sont ignores par Git. Voir
[docs/validation/astra-p69-address-fiber-representation-contract.md](docs/validation/astra-p69-address-fiber-representation-contract.md),
la syntaxe contractuelle
[docs/validation/astra-p69-atlas-contract-syntax.md](docs/validation/astra-p69-atlas-contract-syntax.md)
et le rapport d'analyse
[docs/analysis/ASTRA-P69-address-fiber-representation-contract-analysis.md](docs/analysis/ASTRA-P69-address-fiber-representation-contract-analysis.md).

## Validation P70 contract replay et test stack

P70 ajoute `contract-replay` pour rejouer le contrat P69 sur plusieurs fixtures
address-fiber et comparer bytes declares vs bytes mesures par replay. P70 ajoute
aussi la regle d'audit de la stack de tests Rust a chaque jalon repo-first.

```bash
cargo run -p atlas-cli -- contract-replay examples/valid/p69_address_fiber_contract.atlas \
  --fixtures all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --tolerance-percent 5.0 \
  --export-dir artifacts/p70/contract_replay_standard \
  --format json
```

Les exports restent sous `artifacts/p70/` et sont ignores par Git. Voir
[docs/validation/astra-p70-contract-replay-test-stack.md](docs/validation/astra-p70-contract-replay-test-stack.md),
l'audit de tests
[docs/analysis/ASTRA-P70-test-stack-audit.md](docs/analysis/ASTRA-P70-test-stack-audit.md)
et le rapport d'analyse
[docs/analysis/ASTRA-P70-contract-replay-test-stack-analysis.md](docs/analysis/ASTRA-P70-contract-replay-test-stack-analysis.md).

## Validation P71 filesystem fiber store

P71 ajoute `fiber-store-bench` pour construire un Fiber Store reel sur
filesystem local, mesurer les bytes payes, decoder des fibres par adresse,
executer des queries deterministes, et verifier qu'un corpus incompressible ne
produit pas de faux gain.

```bash
cargo run -p atlas-cli -- fiber-store-bench \
  --corpus all \
  --budget-bytes 10485760 \
  --runs 30 \
  --queries 1000 \
  --export-dir artifacts/p71/fiber_store_standard \
  --format json
```

Les exports restent sous `artifacts/p71/` et sont ignores par Git. Voir
[docs/validation/astra-p71-filesystem-fiber-store-real-data.md](docs/validation/astra-p71-filesystem-fiber-store-real-data.md),
l'audit de tests
[docs/analysis/ASTRA-P71-test-stack-audit.md](docs/analysis/ASTRA-P71-test-stack-audit.md)
et le rapport d'analyse
[docs/analysis/ASTRA-P71-filesystem-fiber-store-real-data-analysis.md](docs/analysis/ASTRA-P71-filesystem-fiber-store-real-data-analysis.md).

## Validation P72 living procedural fiber store

P72 ajoute `living-store-bench` pour tester un store ASTRA vivant: etat froid
persistant, etat runtime temporaire, actions read/query/update/delete/audit,
compaction, close, reopen et replay du journal. Le cache runtime n'est pas
requis pour la correction; seule l'equivalence logique observable apres reopen
est gatee.

```bash
cargo run -p atlas-cli -- living-store-bench \
  --corpus all \
  --budget-bytes 10485760 \
  --runs 30 \
  --queries 1000 \
  --updates 100 \
  --deletes 20 \
  --compact threshold \
  --adaptive on \
  --export-dir artifacts/p72/living_store_standard \
  --format json
```

Les exports restent sous `artifacts/p72/` et sont ignores par Git. Voir
[docs/validation/astra-p72-living-procedural-fiber-store.md](docs/validation/astra-p72-living-procedural-fiber-store.md),
l'audit de tests
[docs/analysis/ASTRA-P72-test-stack-audit.md](docs/analysis/ASTRA-P72-test-stack-audit.md)
et le rapport d'analyse
[docs/analysis/ASTRA-P72-living-procedural-fiber-store-analysis.md](docs/analysis/ASTRA-P72-living-procedural-fiber-store-analysis.md).

Resultat local P72 standard: `reopen_equivalence=true`,
`ratio_living=2.366879`, `guard_decision=NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`,
`drift_status=HARD_DRIFT`, decision
`RECALIBRATE_P72_LIVING_COST_MODEL`.

## Validation P73 cubical fiber living store

P73 ajoute `cubical-store-bench` pour tester une topologie cubique du store
vivant: cellules, six faces, contraintes de recollement, audit de gluing,
compaction par faces, corruption/recovery controlee, et comparaison au baseline
P72.

```bash
cargo run -p atlas-cli -- cubical-store-bench \
  --corpus all \
  --budget-bytes 10485760 \
  --cycles 10 \
  --queries 5000 \
  --updates 1000 \
  --deletes 100 \
  --corruptions 3 \
  --compact threshold \
  --adaptive on \
  --compare-p72 baseline \
  --export-dir artifacts/p73/cubical_store_standard \
  --format json
```

Les exports restent sous `artifacts/p73/` et sont ignores par Git. Voir
[docs/validation/astra-p73-cubical-fiber-living-store.md](docs/validation/astra-p73-cubical-fiber-living-store.md),
l'audit de tests
[docs/analysis/ASTRA-P73-test-stack-audit.md](docs/analysis/ASTRA-P73-test-stack-audit.md)
et le rapport d'analyse
[docs/analysis/ASTRA-P73-cubical-fiber-living-store-analysis.md](docs/analysis/ASTRA-P73-cubical-fiber-living-store-analysis.md).

Resultat local P73: standard ameliore `ratio_living` vs P72
(`cubical_gain_vs_p72=1.131893`), mais ambitious retombe sous P72
(`0.904937`) sous pression de gluing/journaux de faces. Decision:
`RECALIBRATE_P73_CUBICAL_FIBER_TOPOLOGY`.

## Validation P74 living fiber topology search

P74 ajoute `topology-living-bench` pour comparer plusieurs topologies de fibres
en mode living-memory avec environ 10 MiB de donnees sources deterministes:
linear, cubical, trie prefix, graph adjacency, hypergraph tag et hierarchical
tile.

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

Les exports restent sous `artifacts/p74/` et sont ignores par Git. Voir
[docs/validation/astra-p74-living-fiber-topology-search.md](docs/validation/astra-p74-living-fiber-topology-search.md),
l'audit de tests
[docs/analysis/ASTRA-P74-test-stack-audit.md](docs/analysis/ASTRA-P74-test-stack-audit.md)
et le rapport d'analyse
[docs/analysis/ASTRA-P74-living-fiber-topology-search-analysis.md](docs/analysis/ASTRA-P74-living-fiber-topology-search-analysis.md).

Resultat local P74: `target_source_bytes=10485760`,
`actual_source_bytes=10485760`, meilleur ratio observe avec
`hierarchical_tile_fiber` sur sparse CSV (`ratio_living=4.742439` standard),
mais les meilleurs choix restent dependants du corpus. Decision:
`RECALIBRATE_P74_FIBER_TOPOLOGY_SEARCH`.

## ASTRA Results LaTeX/PDF

Les rapports Results figes vivent sous [reports/](reports/). Le rapport
Markdown d'analyse reste la trace vivante; le `.tex` et le `.pdf` Results sont
generes localement apres validation. Tectonic est le compilateur prioritaire,
avec fallback `latexmk` puis `pdflatex`:

```bash
bash scripts/build_report.sh reports/P63/RPA_ASTRA-P63-Results_measured-ratio_v1.0_2026-05-01.tex
```

La CI reste minimale et ne compile pas les rapports lourds.

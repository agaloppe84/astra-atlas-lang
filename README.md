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

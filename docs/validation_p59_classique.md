# ASTRA-P59 Validation Classique

## Objet de P59

ASTRA-P59 consolide les acquis ASTRA-SYS-P58 comme invariants ASTRA classiques.
P59 ne crée pas un nouveau runtime, ne modifie pas la grammaire `.atlas`, et ne
change pas les commandes CLI. Son rôle est d'interpréter la surface P58 déjà
validée: workloads multi-familles, modes runtime, rapports stables, corpus
invalide, compatibilité P57.

## P59 Est ASTRA Classique

P58 était un sprint système: il a ajouté des modes runtime déterministes, des
rapports JSON/Markdown stables, des goldens, un script de validation locale et
des checks CI. P59 est classique parce qu'il ne produit pas de mécanisme
nouveau. Il relie les résultats P58 au protocole ASTRA classique: décider si les
programmes `.atlas` strict P53 gardent leurs invariants lorsqu'ils sont évalués
par la surface runtime P58.

## Interprétation Des Modes P58

- `smoke`: couverture partielle, rapide, CI-safe. Son rapport P58 peut décider
  `RECALIBRATE` de manière volontaire, car il ne couvre pas toutes les familles
  actives non-guard.
- `standard`: référence classique raisonnable. Il couvre toutes les familles
  actives non-guard de `examples/p53_strict.atlas` et doit produire la décision
  `VALIDATE` lorsque les invariants stricts restent vrais.
- `ambitious`: validation locale/manuelle plus large. Il reste déterministe,
  mais il n'est pas exigé en CI et ne doit pas devenir une condition bloquante
  de validation distante.

## Reports P58 Comme Surface Classique

Les rapports P58 sont la surface classique de validation P59:

- `tests/golden/p58_report_smoke.json` fixe la sémantique du mode `smoke`:
  couverture partielle, guard non encodé, snapshot/rebuild disponibles,
  décision `RECALIBRATE`.
- `tests/golden/p58_report_standard.json` fixe la sémantique du mode
  `standard`: couverture des 11 familles actives non-guard, guard refusé,
  `snapshot_full` refusé, décision `VALIDATE`.
- `tests/golden/p57_report.json` reste la preuve de compatibilité P57:
  le report P57 classique n'est pas remplacé par P58.

P59 interprète donc les rapports P58 comme des observations structurées et
versionnées des invariants classiques, sans transformer `.atlas` en langage
généraliste.

## Gates P59

| Gate | Interprétation | Source repo |
| --- | --- | --- |
| `P59_G0_p58_local_ci_validated` | P58 a été validée localement et par GitHub Actions selon le contexte utilisateur. | Trace utilisateur / CI externe |
| `P59_G1_standard_mode_classical_reference` | `standard` est la référence classique raisonnable. | `p58_report_standard.json`, `p58_tests.rs` |
| `P59_G2_smoke_mode_recalibrate_interpreted` | `smoke` est partiel et son `RECALIBRATE` est attendu. | `p58_report_smoke.json`, `p58_tests.rs` |
| `P59_G3_p58_reports_stable_as_classical_surface` | Les reports P58 sont stables et goldenisés. | `tests/golden/p58_report_*.json` |
| `P59_G4_invalid_corpus_preserved_20_20` | Les 20 invalides restent refusés. | `examples/invalid/*.atlas`, `atlas_tests.rs`, CI |
| `P59_G5_p57_compatibility_preserved` | Le report P57 reste inchangé. | `tests/golden/p57_report.json`, `p57_tests.rs` |
| `P59_G6_no_atlas_grammar_change` | P59 n'ajoute aucune syntaxe `.atlas`. | absence de changement parser/grammar |
| `P59_G7_ambitious_local_manual_status_clarified` | `ambitious` reste local/manual, non requis en CI. | `docs/validation_p58.md`, CI |
| `P59_G8_classical_decision_clear` | Décision classique claire: `standard` valide, `smoke` recalibre. | ce document, reports P58 |

Décision P59 recommandée après validation locale et CI:

`VALIDATE_REINTEGRATION_CLASSIQUE_P59`

Décision alternative si une preuve locale/CI manque ou si un golden diverge:

`RECALIBRATE_P59`

## Commandes Locales Recommandées

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
bash scripts/validate_p58_local.sh
cargo run -p atlas-cli -- report examples/p53_strict.atlas --mode smoke --format json
cargo run -p atlas-cli -- report examples/p53_strict.atlas --mode standard --format json
cargo run -p atlas-cli -- report examples/p53_strict.atlas --format json
```

Ces commandes doivent être exécutées localement puis confirmées par GitHub
Actions. Ce document ne constitue pas une preuve d'exécution.

## Limites Héritées De P58

- `smoke` reste volontairement partiel.
- `ambitious` reste local/manual et n'est pas une gate CI.
- Les métriques P58 sont structurelles et déterministes; elles ne sont pas des
  benchmarks de performance réalistes.
- Le warning Cargo indiquant que `src/main.rs` est partagé par `atlas-cli` et
  `atlasc` est connu et non bloquant.
- Le corpus invalide est strict, mais les diagnostics layout/index inconnus
  passent encore par `E_LAYOUT_INDEX_MISMATCH`.

## Après P59

Recommandation: ouvrir ASTRA-SYS-P60 pour un cleanup ciblé:

- séparer proprement les binaires `atlas-cli` et `atlasc`;
- clarifier certains diagnostics spécialisés;
- définir un vrai protocole de benchmark réaliste, distinct des métriques
  structurelles P58;
- conserver la compatibilité P57/P58 pendant ce nettoyage.

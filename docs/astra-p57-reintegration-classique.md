# ASTRA-P57 Reintegration Classique

ASTRA-P57 est une iteration ASTRA classique, pas un sprint ASTRA-SYS.
Elle n'ajoute pas de mecanisme systeme, de nouveau runtime, ni de langage
generaliste. Son role est de relier les invariants classiques issus de
P51/P53 aux capacites deja validees dans P55/P55.1/P56.

## Positionnement

P55/P55.1 ont fige le format `.atlas` strict:

- `strict_p53=true` reste obligatoire pour le corpus courant.
- `guard` reste une famille sentinelle refuse-only.
- `snapshot=full` reste refuse en strict P53.
- Les familles, actions, safety policies, layouts, indexes et thresholds restent
  contraints par la table stricte.
- L'export JSON canonique reste deterministe.

P56 a ajoute un runtime smoke minimal:

- instanciation depuis un programme `.atlas` valide;
- encode, read, update;
- snapshot incremental;
- rebuild;
- export de metriques JSON deterministes.

P57 reintegre ces acquis dans le protocole ASTRA classique: le repo devient le
support concret qui montre que les invariants classiques peuvent etre exprimes,
refuses quand ils sont invalides, puis observes via le runtime smoke P56.

## Commande de rapport

La commande P57 produit un rapport JSON synthetique:

```bash
cargo run -p atlas-cli -- report examples/p53_strict.atlas --format json
```

Le rapport contient les champs de reintegration principaux:

- iteration ASTRA;
- version `.atlas`;
- activation de `strict_p53`;
- nombre de familles actives/refusees;
- refus de `guard`;
- politique de snapshot;
- refus de `snapshot=full`;
- disponibilite du chemin runtime smoke;
- presence des policies de seuil, safety, layout et index;
- stabilite de l'export JSON;
- decision P57.

La decision est volontairement conservatrice. Le rapport de la commande ne
pretend pas que `cargo test` ou GitHub Actions ont deja ete executes. Tant que
ces preuves externes ne sont pas renseignees par Codespaces/GitHub Actions, la
decision reste `RECALIBRATE_P57` plutot que
`VALIDATE_REINTEGRATION_CLASSIQUE_RUNTIME`.

## Source de verite

Le repo reste la surface de validation:

- exemples valides dans `examples/` et `examples/valid/`;
- mutants invalides dans `examples/invalid/`;
- golden files dans `tests/golden/`;
- tests Rust dans `tests/`;
- CI dans `.github/workflows/rust.yml`.

La validation locale dans un environnement Rust isole sous `/Users/work/Astra`
est acceptee pendant le developpement quand elle utilise un `CARGO_TARGET_DIR`
hors repo et conserve seulement des traces synthetiques. GitHub Actions reste
la source de verite distante finale.

Le rapport P57 ne remplace pas:

- `cargo fmt --all -- --check`;
- `cargo build --workspace`;
- `cargo test --workspace`;
- les checks valides/invalides;
- le runtime smoke;
- le statut CI GitHub Actions.

## Traces legeres

P57 conserve uniquement des traces structurees et synthetiques. Les traces
doivent etre petites, versionnees et faciles a transmettre dans une future
conversation ChatGPT.

Fichiers prevus:

- `docs/validation/astra-p57-validation-summary.md`
- `artifacts/p57/astra-p57-validation-summary.template.json`

Ne pas versionner:

- `target/`;
- artefacts de build;
- logs bruts volumineux;
- dumps de benchmarks;
- fichiers temporaires `/tmp`;
- sorties stdout completes si elles ne sont pas courtes et essentielles.

Si un log brut aide au diagnostic, il doit etre resume dans le Markdown ou le
JSON synthetique, puis conserve hors repo.

## Decision P57

La decision `VALIDATE_REINTEGRATION_CLASSIQUE_RUNTIME` n'est admissible que si:

- `strict_p53` est preserve;
- `guard` reste refuse;
- `snapshot=full` reste refuse;
- les exemples valides passent;
- les mutants invalides echouent;
- le chemin runtime smoke existe;
- le JSON report/export est stable;
- les tests existants ne regressent pas;
- Codespaces et GitHub Actions confirment les resultats.

Sinon:

- `RECALIBRATE_P57` indique que les invariants repo-internes tiennent mais que
  les preuves externes ou certains controles restent a renseigner.
- `NO_GO_P57` indique qu'un invariant central est casse.

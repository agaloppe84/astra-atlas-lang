# ASTRA-SYS-P60 Cleanup Validation

## Objet

ASTRA-SYS-P60 est un sprint systeme cible. Il nettoie la surface CLI, rend les
diagnostics invalides plus precis, et ajoute une trace de benchmark/reporting
plus realiste sans transformer `.atlas` en langage generaliste.

P60 ne change pas la grammaire `.atlas`, ne modifie pas `strict_p53`, ne retire
aucun invalide, et ne remplace pas les goldens P57/P58.

## Cleanup CLI

Le warning Cargo historique venait du fait que `atlas-cli` et `atlasc`
pointaient tous les deux vers `src/main.rs`.

P60 separe les points d'entree:

- `src/cli.rs`: logique CLI partagee;
- `src/main.rs`: wrapper `atlas-cli`;
- `src/bin/atlasc.rs`: wrapper `atlasc`;
- `Cargo.toml`: `atlasc` pointe vers `src/bin/atlasc.rs`.

Les noms de binaires restent `atlas-cli` et `atlasc`. Le comportement utilisateur
doit rester identique.

## Diagnostics Types

P60 specialise des diagnostics qui etaient volontairement agreges:

- `E_UNKNOWN_LAYOUT` pour layout inconnu;
- `E_UNKNOWN_INDEX` pour index inconnu;
- `E_THRESHOLD_MALFORMED` pour seuil non parsable;
- `E_THRESHOLD_OUT_OF_RANGE` pour seuil hors plage;
- `E_MISSING_LAYOUT` pour layout requis absent;
- `E_DUPLICATE_KEY` pour cle dupliquee dans une ligne `.atlas`.

`E_LAYOUT_INDEX_MISMATCH` reste disponible pour les combinaisons connues mais
incompatibles. `E_THRESHOLD_INVALID` reste disponible pour compatibilite des
explications, mais les mutants P60 cibles utilisent les codes plus precis.

Le corpus invalide est etendu avec `examples/invalid/duplicate_key.atlas`.

## Benchmark Et Reporting

P60 ajoute une sortie stable:

```bash
cargo run -p atlas-cli -- bench --mode standard --format json
```

Cette sortie est une trace structurelle deterministe, pas une mesure de
performance industrielle:

- `benchmark_kind` vaut `deterministic_structural_proxy`;
- `elapsed_ms` vaut `null`;
- `p50_proxy_cost_units`, `p95_proxy_cost_units`, et `p99_proxy_cost_units`
  sont des couts proxy deterministes;
- `smoke` reste CI-safe et partiel;
- `standard` reste la reference raisonnable;
- `ambitious` reste local/manual.

Le mode texte existant de `bench --mode smoke|standard|ambitious` reste conserve.

## Limites

- P60 ne mesure pas de latence wall-clock.
- Les couts proxy ne doivent pas etre presentes comme benchmark industriel.
- `ambitious` ne doit pas devenir obligatoire en CI.
- Les goldens P57/P58 restent la surface de compatibilite stable existante.
- La validation distante reste GitHub Actions; ce document ne remplace pas la CI.

## Commandes De Validation

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

git status --short
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
bash scripts/validate_p58_local.sh

cargo run -p atlas-cli -- check examples/p53_strict.atlas
cargo run -p atlas-cli -- export examples/p53_strict.atlas --format json > /tmp/p53_strict.json
diff -u tests/golden/p53_strict.json /tmp/p53_strict.json
cargo run -p atlas-cli -- bench --mode smoke
cargo run -p atlas-cli -- bench --mode standard
cargo run -p atlas-cli -- bench --mode standard --format json
cargo run -p atlas-cli -- report examples/p53_strict.atlas --mode standard --format json
cargo run -p atlas-cli -- report examples/p53_strict.atlas --format json

for f in examples/invalid/*.atlas; do
  echo "checking invalid: $f"
  if cargo run -p atlas-cli -- check "$f"; then
    echo "ERROR: invalid program unexpectedly passed: $f"
    exit 1
  else
    echo "OK: invalid rejected: $f"
  fi
done
```

## Trace Locale A Completer

```text
ASTRA-SYS-P60 local validation trace
branch/commit:
files changed:
cli cleanup:
cargo warning:
diagnostics:
benchmark/reporting:
fmt:
build:
test:
validate_p58_local:
valid CLI commands:
invalid corpus:
goldens:
CI status:
known limits:
recommendation:
```

### Exemple De Remplissage Sans Resultats Inventes

```text
ASTRA-SYS-P60 local validation trace
branch/commit: <branch> / <commit>
files changed: <git status --short summary>
cli cleanup: NOT_RUN / PASS / FAIL
cargo warning: NOT_CHECKED / ABSENT / PRESENT
diagnostics: NOT_RUN / PASS / FAIL
benchmark/reporting: NOT_RUN / PASS / FAIL
fmt: NOT_RUN / PASS / FAIL
build: NOT_RUN / PASS / FAIL
test: NOT_RUN / PASS / FAIL
validate_p58_local: NOT_RUN / PASS / FAIL
valid CLI commands: NOT_RUN / PASS / FAIL
invalid corpus: NOT_RUN / <rejected>/<checked>
goldens: NOT_RUN / PASS / FAIL
CI status: NOT_RUN / PASS / FAIL / PASS_USER_REPORTED
known limits: <remaining limits, if any>
recommendation: RECALIBRATE_P60 / VALIDATE_ASTRA_SYS_P60
```

Ne pas committer de logs bruts, `target/`, dumps de benchmark volumineux, ou
sorties temporaires. Si une trace doit etre versionnee, preferer un petit resume
Markdown/JSON.

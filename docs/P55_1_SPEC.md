# ASTRA-SYS-P55.1 — backend Rust strict et équivalence Python/Rust

## Objectif

P55.1 durcit astra-atlas-lang après P55.

Le but n'est pas d'ajouter un nouveau concept ASTRA, mais de transformer le backend Rust en backend strict:
parser, typechecker, diagnostics, export JSON canonique, corpus mutants, CLI et CI.

## Livrables repo

- diagnostics typés stables
- corpus .atlas valide et invalide élargi
- tests Rust d'équivalence
- export JSON canonique
- commandes atlasc:
  - check
  - explain
  - export --format json
  - bench --mode smoke
- CI GitHub Actions durcie

## Gates P55.1

G0_repo_clean:
cargo fmt, cargo test, CI OK.

G1_valid_equivalence:
Tous les programmes valides du corpus sont acceptés.

G2_invalid_equivalence:
Tous les mutants invalides sont refusés.

G3_diagnostic_stability:
Chaque refus possède un code stable.

Codes minimaux:
- E_VERSION_UNKNOWN
- E_GUARD_ACTIVE
- E_SNAPSHOT_FULL_STRICT
- E_ACTION_UNKNOWN
- E_SAFETY_UNKNOWN
- E_LAYOUT_INDEX_MISMATCH
- E_THRESHOLD_INVALID
- E_MISSING_FAMILIES

G4_json_canonical_equivalence:
Export JSON déterministe.

G5_strict_p53_preserved:
snapshot_full reste refusé, guard actif reste refusé, strict_p53 reste actif.

G6_no_p55_regression:
Aucune régression sur parse_ok, typecheck_ok, invalid rejection, snapshot_full rejection,
strict_p53 equivalence, guard refusal, dangerous encoded refusal.

G7_ci_source_of_truth:
GitHub Actions passe et ses logs sont fournis.

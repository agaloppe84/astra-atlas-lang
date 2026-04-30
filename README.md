# ASTRA Atlas Lang

Prototype P55 d'un mini-langage `.atlas` pour programmes mémoire procéduraux ASTRA.

## Commandes

```bash
cargo test
cargo run -- examples/p53_strict.atlas
```

## Codespaces

1. Créer un repo GitHub `astra-atlas-lang`.
2. Copier ce dossier à la racine du repo.
3. Ouvrir `Code -> Codespaces -> Create codespace on main`.
4. Le devcontainer Rust installe l'environnement et lance `cargo test`.

## Objectif P55

Le but n'est pas de créer un langage généraliste, mais de figer un format testable :
parser, typechecker, runtime minimal, refus des mutants, et équivalence des invariants P53/P54.1.

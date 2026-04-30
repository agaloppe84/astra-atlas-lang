# ASTRA Atlas Lang

Prototype P55 d'un mini-langage `.atlas` pour programmes mémoire procéduraux ASTRA.

## Commandes

```bash
cargo test --workspace
cargo run -p atlas-cli -- check examples/p53_strict.atlas
cargo run -p atlas-cli -- explain E_GUARD_ACTIVE
cargo run -p atlas-cli -- export examples/p53_strict.atlas --format json
cargo run -p atlas-cli -- run examples/p53_strict.atlas --mode smoke
cargo run --bin atlas-cli -- metrics examples/p53_strict.atlas --format json
cargo run -p atlas-cli -- bench --mode smoke
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

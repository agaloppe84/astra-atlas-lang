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

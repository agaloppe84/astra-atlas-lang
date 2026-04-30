# ASTRA agent instructions

Repository: astra-atlas-lang
Sprint: ASTRA-SYS-P55.1 — backend Rust strict and Python/Rust equivalence.

Non-negotiable rules:
- Do not turn .atlas into a general-purpose language.
- Do not weaken strict_p53.
- Do not accept guard as active.
- Do not allow snapshot=full in strict_p53.
- Do not remove invalid mutants to make tests pass.
- Do not make CI permissive.
- Do not invent benchmark results.
- Do not claim success without cargo logs and CI status.

Primary objective:
Durcir le backend Rust atlasc pour garantir une équivalence stricte avec le miroir Python:
same valid programs accepted, same invalid mutants refused, stable typed diagnostics,
canonical JSON export, and no regression on P55/P53/P54.1 invariants.

Required validation commands:
cargo fmt --all -- --check
cargo test --workspace
cargo run -p atlas-cli -- check examples/p53_strict.atlas
cargo run -p atlas-cli -- check examples/invalid/snapshot_full.atlas
cargo run -p atlas-cli -- export examples/p53_strict.atlas --format json

Expected P55.1 gates:
- valid Python/Rust equivalence
- invalid Python/Rust equivalence
- stable diagnostic codes
- canonical JSON export
- strict_p53 preserved
- guard refused
- snapshot_full refused
- cargo test OK
- GitHub Actions CI OK

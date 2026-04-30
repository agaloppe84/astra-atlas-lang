# ASTRA-SYS-P58 Validation

ASTRA-SYS-P58 moves the repo from smoke-only runtime validation to deterministic
multi-family runtime workloads and stable reports. The `.atlas` grammar remains
unchanged: P58 modes are runtime validation profiles, not language syntax.

## Workload Modes

- `smoke`: fast compatibility path inherited from P56/P57. It intentionally
  covers only a small representative subset, so its P58 decision can be
  `RECALIBRATE`.
- `standard`: deterministic multi-family workload covering every active
  non-guard family in `examples/p53_strict.atlas`. This is the main P58
  representative validation profile and should report `VALIDATE`.
- `ambitious`: deterministic larger local/manual workload. It may report
  `VALIDATE` with a local-only warning, but it is not required by CI.

The guard family remains a refuse-only sentinel and is not encoded as a normal
workload. `strict_p53` and `snapshot=incremental_manifest` remain required.

## Local-First Validation

Run the compact local validator:

```bash
bash scripts/validate_p58_local.sh
```

The script runs format, build, tests, P53 export, smoke and standard runtime
paths, P58 JSON/Markdown reports, P57 compatibility, and the full invalid corpus
loop. It writes temporary output under the system temp directory and avoids raw
logs or build artifacts in the repo.

To write a small local summary artifact, opt in explicitly:

```bash
P58_WRITE_ARTIFACTS=1 bash scripts/validate_p58_local.sh
```

This creates `artifacts/p58/astra-p58-local-validation-summary.json`.

## Manual Commands

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo run -p atlas-cli -- bench --mode smoke
cargo run -p atlas-cli -- bench --mode standard
cargo run -p atlas-cli -- run examples/p53_strict.atlas --mode smoke
cargo run -p atlas-cli -- run examples/p53_strict.atlas --mode standard
cargo run -p atlas-cli -- metrics examples/p53_strict.atlas --mode standard --format json
cargo run -p atlas-cli -- report examples/p53_strict.atlas --mode standard --format json
cargo run -p atlas-cli -- report examples/p53_strict.atlas --mode standard --format markdown
```

## CI Scope

GitHub Actions keeps CI strict but lightweight:

- format, build, and full workspace tests;
- P55/P57 compatibility goldens;
- full invalid corpus rejection checks;
- P56 smoke metrics;
- P58 smoke and standard report goldens.

CI does not run `ambitious` mode. That mode is reserved for local/manual checks.

## Known Warning

Cargo currently warns that `src/main.rs` is used by both the `atlas-cli` and
`atlasc` binary targets. This is a compatibility alias issue, not a validation
failure. A future cleanup can move the CLI dispatcher into a shared module and
give `atlasc` a small wrapper binary, but P58 leaves the binaries unchanged.

# ASTRA-P76 Routing Oracle and Virtual Space Validation

P76 adds `routing-oracle-bench` and `virtual-space-estimate`.

## Routing Oracle Bench

```bash
cargo run -p atlas-cli -- routing-oracle-bench \
  --corpus all \
  --target-source-bytes 10485760 \
  --cycles 10 \
  --queries 10000 \
  --updates 1000 \
  --deletes 100 \
  --locality all \
  --update-pressure all \
  --compare oracle,mixed,hierarchical,linear,cubical,trie,graph,hypergraph \
  --export-dir artifacts/p76/routing_oracle_standard \
  --format json
```

Exports stay under `artifacts/p76/` and are ignored by Git.

## Virtual Space Estimate

```bash
cargo run -p atlas-cli -- virtual-space-estimate \
  --topology mixed \
  --target-source-bytes 10485760 \
  --cells 10000 \
  --fibers-per-cell 4 \
  --hierarchy-depth 5 \
  --format json
```

The fields `virtual_declared_bytes_equivalent` and
`virtual_effective_bytes_equivalent` are materialization equivalents, not stored
bytes.

## Local Validation

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p76_tests
cargo test --test p75_tests
bash scripts/validate_p58_local.sh
```

P76 invalid contracts live under `examples/invalid/p76_*.atlas`.

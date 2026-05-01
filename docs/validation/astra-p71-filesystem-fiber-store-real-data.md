# ASTRA-P71 — Filesystem Fiber Store Real Data Validation

P71 tests whether an address-fiber procedural store can write real local files,
decode exact fibers by address, retrieve useful records, and measure the paid
filesystem bytes under a hard 10 MiB budget.

## Scope

The P71 store is local-first and deterministic. It uses no network and no
external dataset. The corpus is built from:

- `real_code_corpus`: repository Rust, `.atlas`, validation Markdown, and
  analysis Markdown files;
- `realish_logs_corpus`: deterministic structured logs;
- `realish_json_records`: deterministic nested JSON records;
- `sparse_csv_table`: deterministic sparse tabular rows;
- `incompressible_guard_blob`: deterministic pseudorandom guard bytes.

The incompressible guard must not produce false gain. It must be refused,
stored as explicit raw fallback, or classified no-go.

## CLI

```bash
cargo run -p atlas-cli -- fiber-store-bench \
  --corpus all \
  --budget-bytes 10485760 \
  --runs 30 \
  --queries 1000 \
  --export-dir artifacts/p71/fiber_store_standard \
  --format json
```

## Exports

The generated artifacts stay under `artifacts/p71/` and are ignored by Git:

- `p71_fiber_store_report.json`
- `p71_fiber_store_summary.md`
- `p71_store_cost_breakdown.csv`
- `p71_fiber_records.jsonl`
- `p71_decode_report.json`
- `p71_query_report.json`
- `store/manifest.json`
- `store/contract.json`
- `store/address_index.json`
- `store/dictionaries/`
- `store/generators/`
- `store/residuals/`
- `store/journals/`
- `store/checksums/`
- `store/audit/`
- `store/raw_fallback/`

## Metrics

P71 reports:

- `source_dataset_bytes`
- `total_store_bytes`
- `exact_recoverable_bytes`
- `useful_retrieved_bytes`
- `exact_bytes_per_store_byte`
- `useful_retrieved_bytes_per_store_byte`
- `procedural_store_gain_vs_raw`
- `budget_used_percent`
- `declared_vs_measured`
- roundtrip success and checksum pass rates
- retrieval precision/recall/exact match rate
- guard decision and `guard_no_false_gain`

## Decision policy

Possible P71 decisions are:

- `VALIDATE_P71_FILESYSTEM_FIBER_STORE`
- `RECALIBRATE_P71_ENCODING_MODEL`
- `RECALIBRATE_P71_RETRIEVAL_MODEL`
- `RECALIBRATE_P71_CONTRACT_COST_MODEL`
- `NO_GO_P71_REAL_DATA_FIBER_STORE`

The default decision remains conservative. P71 should only validate if exact
roundtrip, retrieval, budget, guard behavior, invalid corpus refusal, and
declared-vs-measured drift all pass with no hidden storage.

## Local validation

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p71_tests
cargo test --test p70_tests
bash scripts/validate_p58_local.sh
```

The CI remains sanity-only and does not run long P71 campaigns.

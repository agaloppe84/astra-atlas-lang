# ASTRA-P69 — Address-Fiber Representation Contract

P69 defines the stored form behind the P68 promoted
`address_fiber_actor_managed_v1` architecture.

The central rule is simple: the virtual space is not stored globally. What is
stored is a finite procedural contract: address space, fiber schema, generator,
quantized parameters, dictionary/ROM, index, residuals, journal, actor policy,
budgets, safety gates and audit metadata.

## Commands

```bash
cargo run -p atlas-cli -- check examples/valid/p69_address_fiber_contract.atlas
cargo run -p atlas-cli -- contract-check examples/valid/p69_address_fiber_contract.atlas --format json
cargo run -p atlas-cli -- contract-run examples/valid/p69_address_fiber_contract.atlas \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --export-dir artifacts/p69/contract_standard \
  --format json
```

## Contract gates

The typechecker requires:

- known architecture: `address_fiber_actor_managed_v1`;
- known cost model: `measured_or_declared_contract_v1`;
- existing references for address space, fiber schema, generator and actor
  policy;
- `budget_bytes > 0`;
- known projection, journal, audit, compaction and cache policies;
- required storage fields marked `accounted`;
- `all_storage_counted = true`;
- zero conflicts and stale reads;
- low budget refusal rate;
- overhead and net-gain gates compatible with the P68 promoted envelope.

## Cost categories

The report must expose:

- `generator_code_bytes`;
- `parameter_bytes`;
- `dictionary_or_rom_bytes`;
- `index_bytes`;
- `residual_bytes`;
- `journal_bytes`;
- `cache_bytes`;
- `actor_state_bytes`;
- `audit_metadata_bytes`;
- `manifest_bytes`;
- `safety_metadata_bytes`;
- `total_contract_bytes`.

No cache, journal, index, actor state or audit metadata is allowed to be hidden.

## Invalid fixtures

P69 adds invalid contracts for:

- missing fiber schema;
- unknown generator reference;
- unaccounted actor state;
- missing `all_storage_counted` gate;
- zero actor budget;
- unknown projection;
- unknown representation reference.

These examples must remain refused.

## Decision

P69 decisions:

- `PROMOTE_P69_ADDRESS_FIBER_CONTRACT_RUNTIME`;
- `RECALIBRATE_P69_REPRESENTATION_CONTRACT`;
- `NO_GO_P69_CONTRACT_DRIFT`.

Promotion means repo/runtime contract promotion only. It does not remove the
need for P70 contract replay, external fixtures or multi-machine checks.

## Local validation

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p69_tests
cargo test --test p68_tests
bash scripts/validate_p58_local.sh
```

The CI remains a minimal sanity layer; P69 validation is local-first.

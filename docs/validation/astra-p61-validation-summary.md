# ASTRA-P61 Validation Summary

## Purpose

ASTRA-P61 introduces the first Virtual Ratio Lab for addressable procedural
representations. It measures an effective virtual ratio, not a declared-space
headline ratio.

## Central Metric

```text
ratio_effective = virtual_effective / real_total_cost_units
```

`ratio_declared` is reported as context only. It must not drive the P61 decision.

## Current Deterministic Results

- smoke `ratio_effective`: `28.361345`
- standard `ratio_effective`: `72.695035`
- smoke decision: `RECALIBRATE_P61_RATIO_COST_MODEL`
- standard decision: `RECALIBRATE_P61_RATIO_COST_MODEL`

These values come from `deterministic_proxy_v1`. They are useful for repo
regression analysis, but they are not scientific validation and they are not
industrial performance claims.

## Timing Status

Timing fields currently remain `null`:

- `read_p50_us`
- `read_p95_us`
- `read_p99_us`
- `update_p50_us`
- `update_p95_us`
- `update_p99_us`

They are null because no real machine timing is measured in P61.

## Stable Artifacts

- Smoke golden: `tests/golden/p61_ratio_smoke.json`
- CLI smoke command:

```bash
cargo run -p atlas-cli -- ratio examples/p53_strict.atlas --mode smoke --format json
```

## CI Scope

GitHub Actions runs only the P61 smoke ratio report and compares it with the
smoke golden. `standard` remains available for local validation but is not added
as a required CI ratio step in P61.

## Recommended Local Commands

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
bash scripts/validate_p58_local.sh
cargo run -p atlas-cli -- ratio examples/p53_strict.atlas --mode smoke --format json
cargo run -p atlas-cli -- ratio examples/p53_strict.atlas --mode standard --format json
```

## Next Steps

- calibrate the cost model;
- add real timing once measurement is justified and stable;
- strengthen CRUD semantics beyond proxy counts;
- link P61 gates explicitly to P51/P53 non-regression;
- keep `ratio_effective` as the decision-driving metric.

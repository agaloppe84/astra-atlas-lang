# ASTRA-P62 Real Measurement Plan

## Purpose

ASTRA-P62 starts the measured calibration path for the P61 Virtual Ratio Lab.
P61 keeps `deterministic_proxy_v1`; P62 adds a separate measured path with
`measured_real_v1`.

P62 does not change the `.atlas` grammar, does not weaken `strict_p53`, and does
not change the P61 smoke golden.

## Command

```bash
cargo run -p atlas-cli -- ratio-real examples/p53_strict.atlas --mode smoke --format json
```

`standard` is available locally:

```bash
cargo run -p atlas-cli -- ratio-real examples/p53_strict.atlas --mode standard --format json
```

The `ratio-real` command is intentionally separate from `ratio` so P61 remains
stable and proxy-based.

## Measurement Model

`ratio-real` uses:

- `std::time::Instant` for wall-clock timings;
- real Rust runtime operations for create/read/update/delete/snapshot/rebuild;
- a real audit pass over the measured snapshot/rebuild result;
- temporary files for snapshot, manifest, journal, index, payload, and audit;
- filesystem metadata for persisted byte counts.

Temporary files are removed after each run. No real timing golden is stored
because timing values are machine-local and variable.

## Report Scope

The P62 report includes:

- `astra_iteration = "ASTRA-P62"`;
- `cost_model = "measured_real_v1"`;
- `measurement_kind = "real_wall_clock_and_filesystem"`;
- operation p50/p95/p99 timings in microseconds;
- persisted file byte counts;
- P61 virtual metrics reused as the numerator surface;
- measured ratios such as `ratio_effective_per_byte`;
- CRUD and system operation counts;
- safety and roundtrip gates;
- conservative decision status.

## Interpretation

P62 is not a simulation, but it is still a controlled deterministic workload.
It must not be confused with external production datasets or scientific
validation. Scientific validation requires repeated runs, richer machine
metadata, calibrated thresholds, and external workloads.

The expected first decision is:

```text
RECALIBRATE_P62_MEASUREMENT_MODEL
```

This is deliberate. P62 proves the measurement path exists before claiming
validation quality.

## Local Validation

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
bash scripts/validate_p58_local.sh
cargo run -p atlas-cli -- ratio examples/p53_strict.atlas --mode smoke --format json
cargo run -p atlas-cli -- ratio-real examples/p53_strict.atlas --mode smoke --format json
```

## Next Steps

- repeat measurements across runs and machines;
- add optional machine/run metadata;
- calibrate thresholds for measured ratios;
- strengthen update/delete semantics;
- decide later whether any P62 smoke check belongs in CI.

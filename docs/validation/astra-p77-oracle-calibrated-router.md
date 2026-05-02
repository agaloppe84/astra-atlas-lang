# ASTRA-P77 — Oracle-Calibrated Mixed Router

P77 calibrates the deterministic mixed-topology router introduced in P75 and
measured against the P76 oracle. The calibration is not machine learning: it is
a bounded grid of thresholds and topology biases.

## CLI

```bash
cargo run -p atlas-cli -- routing-oracle-calibrate \
  --corpus all \
  --target-source-bytes 10485760 \
  --cycles 10 \
  --queries 10000 \
  --updates 1000 \
  --deletes 100 \
  --locality all \
  --update-pressure all \
  --grid standard \
  --export-dir artifacts/p77/router_calibration_standard \
  --format json
```

The command writes compact local artifacts under `artifacts/p77/`, which is
ignored by Git.

## Exports

- `p77_router_calibration_report.json`
- `p77_calibration_grid.csv`
- `p77_wrong_routes.jsonl`
- `p77_wrong_route_summary.csv`
- `p77_calibrated_policy.json`
- `p77_virtual_space_metrics.json`
- `p77_summary.md`

## Gates

Promotion requires `router/oracle >= 0.985`, routing accuracy at least `0.96`,
wrong-route cost reduction at least `50%`, no guard false gain, reopen
equivalence, no hard drift, retrieval success and explicit virtual-space
metrics. If any strict gate remains short, the scientific decision stays
`RECALIBRATE_P77_ROUTER_THRESHOLDS`.

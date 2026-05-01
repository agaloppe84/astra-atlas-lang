# ASTRA-P63 Measured Ratio Calibration

## Objective

ASTRA-P63 prepares calibration for the measured ratio path introduced by P62.
It does not replace P61 or P62. It turns repeated measured runs into an analysis
surface that can later support calibrated decisions.

## Motivation

P61 exposed the central virtual ratio with `deterministic_proxy_v1`. P62 added
`ratio-real` with `measured_real_v1`, real `Instant` timings, real temporary
files, filesystem byte counts, `--runs N`, repeated run summaries and decision
reasons.

The remaining problem is scientific calibration. P62 proves that measurement
exists, but the default decision remains:

```text
RECALIBRATE_P62_MEASUREMENT_MODEL
```

## Limits Inherited From P62

- Workloads are deterministic internal workloads.
- Thresholds are not calibrated.
- Machine metadata is sparse.
- Campaign exports are not yet structured as durable analysis artifacts.
- Real timings are machine-local and must not be goldenized.
- P62 does not claim industrial performance or scientific validation.

## Campaign Export Command

Prompt Codex 2 adds the first compact campaign export surface:

```bash
cargo run -p atlas-cli -- ratio-real examples/p53_strict.atlas \
  --mode smoke \
  --format json \
  --runs 5 \
  --export-dir artifacts/p63/smoke \
  --threshold-profile p63
```

Without `--threshold-profile p63`, `ratio-real` keeps the P62 JSON shape. With
`--threshold-profile p63`, stdout becomes a P63 campaign report and
`--export-dir` writes compact analysis files.

`p63` is an alias for the versioned threshold profile
`p63_conservative_v1`.

## Threshold Profiles

The initial profile is intentionally conservative:

- `profile_id = p63_conservative_v1`
- `alias = p63`
- `min_runs_required = 10`
- `max_ratio_cv = 0.05`
- `max_bytes_cv = 0.05`
- `max_timing_cv = 0.50`
- `min_median_ratio_effective_per_byte = null`
- `require_machine_metadata = true`
- `require_campaign_exports = true`
- `require_realish_workloads = false`
- `allow_validate = false`

`allow_validate = false` prevents
`VALIDATE_P63_MEASURED_RATIO_CALIBRATION` even when an internal campaign looks
stable. This keeps P63 honest until thresholds and workloads are calibrated.

## Metrics To Add Later

Future P63 work should consider:

- measured ratio stability across repeated runs;
- persisted byte min/median/max;
- timing p99 min/median/max by operation;
- run-to-run variability;
- robust outlier notes;
- calibrated threshold profile;
- explicit machine metadata completeness.

## Campaign Exports

When `--export-dir` is provided with `--threshold-profile p63`, the command
writes:

- `campaign_report.json`: complete structured P63 campaign report;
- `runs.jsonl`: one compact JSON object per measured run;
- `runs.csv`: compact tabular run summary;
- `summary.md`: human-readable campaign summary.

Generated exports under `artifacts/p63/` are local campaign outputs and should
not be committed by default.

Campaign exports stay compact and versionable in shape. Expected fields include:

- ASTRA iteration;
- branch and commit;
- mode and repeat count;
- machine metadata;
- workload profile;
- summary statistics;
- decision;
- decision reasons;
- warnings and limitations.

Large raw logs, temporary files and timing dumps should not be committed.

## Machine Metadata

The P63 campaign report records best-effort metadata:

- OS;
- architecture;
- CPU count when available;
- Rust and Cargo versions, or `unknown`;
- Git commit, or `unknown`;
- Git dirty status, or `null`;
- UTC timestamp;
- build profile.

## Expected Robust Statistics

P63 prefers robust summaries over fragile exact timings. The first campaign
summary exposes, for key metrics:

- min / median / max;
- mean;
- standard deviation;
- coefficient of variation.

The first metrics covered are:

- `ratio_effective_per_byte`;
- `total_persisted_bytes`;
- `operation_count`;
- `read_p99_us`;
- `update_p99_us`;
- `snapshot_p99_us`;
- `rebuild_p99_us`;
- `audit_p99_us`.

Exact timing goldens are forbidden because measured timings vary by machine and
run.

## Stability Status

Campaign reports expose:

- `ratio_stability_status`;
- `bytes_stability_status`;
- `timing_stability_status`;
- `campaign_stability_status`;
- `stability_reasons`.

Allowed statuses:

- `STABLE`
- `WARN`
- `UNSTABLE`
- `NOT_ENOUGH_RUNS`
- `NOT_AVAILABLE`

`repeat_count < min_runs_required` yields `NOT_ENOUGH_RUNS`. Timing stability
is evaluated softly because timings are machine-dependent.

## P63 Decisions

P63 decision vocabulary:

- `VALIDATE_P63_MEASURED_RATIO_CALIBRATION`
- `RECALIBRATE_P63_THRESHOLDS`
- `RECALIBRATE_P63_WORKLOADS`
- `NO_GO_P63_MEASURED_RATIO_STABILITY`

The current implementation remains conservative. With `--threshold-profile p63`,
the expected decision is:

```text
RECALIBRATE_P63_THRESHOLDS
```

Validation requires calibrated thresholds, stable campaign exports, sufficient
machine metadata and clear workload scope. This prompt does not return
`VALIDATE_P63_MEASURED_RATIO_CALIBRATION`.

## Campaign Comparison

Prompt Codex 3 adds a lightweight comparison command:

```bash
cargo run -p atlas-cli -- ratio-campaign-compare \
  artifacts/p63/smoke/campaign_report.json \
  artifacts/p63/standard/campaign_report.json \
  --format json
```

The comparison reports median ratio and byte shifts, profile compatibility,
mode compatibility, stability summaries and an informational comparison
decision. Different modes are reported as `DIFFERENT_MODES`; deltas are still
emitted, but they are not regression claims.

## Analysis Reports

P63 introduces durable analysis reports under:

```text
docs/analysis/
```

After important local validation steps pass, the corresponding analysis report
should be completed and committed with summarized results, decisions, limits and
the next recommendation.

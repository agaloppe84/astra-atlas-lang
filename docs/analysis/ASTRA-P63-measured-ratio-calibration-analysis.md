# ASTRA-P63 — Measured Ratio Calibration Analysis

## 1. Executive summary

ASTRA-P63 prepares the scientific calibration layer for measured virtual ratio
experiments. It follows:

- P61, which introduced the Virtual Ratio Lab with `deterministic_proxy_v1`;
- P62, which introduced `ratio-real` with `measured_real_v1`, real
  `std::time::Instant` timings, temporary persistence files, filesystem byte
  measurements, `--runs N`, `summary`, `runs`, and `decision_reasons`.

This report was initialized before P63 calibration implementation. Prompt Codex
2 adds the first compact campaign export surface, but it does not claim
scientific ratio validation.

Current P63 status: `PARTIAL_IMPLEMENTATION_CAMPAIGN_EXPORTS`.

Prompt Codex 3 adds versioned threshold profiles, explicit campaign stability
status, enriched compact run exports, and campaign comparison. It still keeps
the scientific decision conservative.

## 2. Position in ASTRA

P63 belongs to the measured ratio path of ASTRA / Représentations Procedurales
Adressables. The central question is the effective relation between:

- virtual space that is reachable, readable, updatable, safe and auditable;
- real paid system cost measured by runtime work and persisted bytes.

The central ratio remains measured/effective, not declared. Declared virtual
space can be reported, but it must not drive the calibration decision.

## 3. Objective of P63

P63 must turn P62 repeated measurements into a calibrated analysis surface. The
first objective is not to validate a scientific claim, but to install the
durable reporting discipline and define the missing calibration components.

Expected future additions:

- richer machine metadata;
- external workload fixtures;
- calibrated threshold profiles;
- calibrated decisions;
- comparison against P61 proxy ratios and P62 measured ratios.

## 4. Repository state

- P61 proxy command: `cargo run -p atlas-cli -- ratio ...`.
- P62 measured command: `cargo run -p atlas-cli -- ratio-real ... --runs N`.
- P62 current scientific decision: `RECALIBRATE_P62_MEASUREMENT_MODEL`.
- P63 campaign export code: `PARTIAL_IMPLEMENTATION_CAMPAIGN_EXPORTS`.
- P63 threshold profile: `p63`, conservative.
- CI status for this P63 report step: `TODO_AFTER_CI`.

## 5. Files changed

Initial P63 analysis/reporting step:

- `docs/analysis/README.md`
- `docs/analysis/ASTRA-P63-measured-ratio-calibration-analysis.md`
- `docs/validation/astra-p63-measured-ratio-calibration.md`
- `README.md`

Runtime, grammar, invalid examples and goldens are not expected to change in
this first P63 documentation step.

Prompt Codex 2 campaign export step:

- `src/p63.rs`
- `src/cli.rs`
- `src/lib.rs`
- `tests/p63_tests.rs`
- `.gitignore`

Prompt Codex 3 calibration layer step:

- `src/p63.rs`
- `src/cli.rs`
- `tests/p63_tests.rs`
- `docs/validation/astra-p63-measured-ratio-calibration.md`
- `docs/analysis/ASTRA-P63-measured-ratio-calibration-analysis.md`
- `README.md`

Runtime semantics, grammar, invalid examples and timing goldens are not expected
to change in this P63 calibration layer step.

## 6. Commands executed

Executed locally for this P63 documentation/reporting step:

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
bash scripts/validate_p58_local.sh

cargo run -p atlas-cli -- ratio examples/p53_strict.atlas --mode smoke --format json
cargo run -p atlas-cli -- ratio-real examples/p53_strict.atlas --mode smoke --format json --runs 3
cargo run -p atlas-cli -- ratio-real examples/p53_strict.atlas --mode standard --format json --runs 5
cargo run -p atlas-cli -- ratio-real examples/p53_strict.atlas --mode smoke --format json --runs 5 --export-dir artifacts/p63/smoke --threshold-profile p63
cargo run -p atlas-cli -- ratio-real examples/p53_strict.atlas --mode standard --format json --runs 10 --export-dir artifacts/p63/standard --threshold-profile p63
cargo run -p atlas-cli -- ratio-campaign-compare artifacts/p63/smoke/campaign_report.json artifacts/p63/standard/campaign_report.json --format json

git status --short
git diff --check
```

Execution status: `LOCAL_VALIDATION_PASS_FOR_PROMPT_3`.

## 7. Validation results

- `cargo fmt --all -- --check`: `PASS`
- `cargo build --workspace`: `PASS`
- `cargo test --workspace`: `PASS`
- `scripts/validate_p58_local.sh`: `PASS`
- P61 proxy smoke report: `PASS`
  - `ratio_effective = 28.361345`
  - decision: `RECALIBRATE_P61_RATIO_COST_MODEL`
- P62 measured smoke sanity command: `PASS`
  - command: `ratio-real ... --mode smoke --runs 3`
  - `repeat_count = 3`
  - `run_count = 3`
  - `total_persisted_bytes = 32765`
  - `ratio_effective_per_byte = 3.296200`
  - `all_runs_passed = true`
  - decision: `RECALIBRATE_P62_MEASUREMENT_MODEL`
- P62 measured standard sanity command: `PASS`
  - command: `ratio-real ... --mode standard --runs 5`
  - `repeat_count = 5`
  - `run_count = 5`
  - `total_persisted_bytes = 383085`
  - `ratio_effective_per_byte = 6.421551`
  - `all_runs_passed = true`
  - decision: `RECALIBRATE_P62_MEASUREMENT_MODEL`
- P63 smoke campaign export: `PASS`
  - command: `ratio-real ... --mode smoke --runs 5 --export-dir artifacts/p63/smoke --threshold-profile p63`
  - `repeat_count = 5`
  - `run_count = 5`
  - median `ratio_effective_per_byte = 3.296200`
  - median `total_persisted_bytes = 32765`
  - `all_runs_passed = true`
  - decision: `RECALIBRATE_P63_THRESHOLDS`
- P63 standard campaign export: `PASS`
  - command: `ratio-real ... --mode standard --runs 10 --export-dir artifacts/p63/standard --threshold-profile p63`
  - `repeat_count = 10`
  - `run_count = 10`
  - median `ratio_effective_per_byte = 6.421551`
  - median `total_persisted_bytes = 383085`
  - `all_runs_passed = true`
  - decision: `RECALIBRATE_P63_THRESHOLDS`
- P63 threshold profile `p63`: `PASS`
  - resolves to `p63_conservative_v1`
  - `allow_validate = false`
- P63 smoke campaign stability:
  - `campaign_stability_status = NOT_ENOUGH_RUNS`
  - reason: `run_count 5 is below min_runs_required 10`
- P63 standard campaign stability:
  - `campaign_stability_status = WARN`
  - ratio/bytes stability: `STABLE`
  - timing stability: `WARN`
- P63 campaign comparison command: `PASS`
  - `compatibility_status = DIFFERENT_MODES`
  - `comparison_decision = COMPARE_P63_DIFFERENT_MODES_INFORMATIONAL`
  - `ratio_shift_percent = 94.816789`
  - `bytes_shift_percent = 1069.189684`
- `git diff --check`: `PASS`
- GitHub Actions before Prompt Codex 3: `PASS_USER_REPORTED`
- GitHub Actions after Prompt Codex 3: `TODO_AFTER_CI`

The results validate the repository/documentation step, inherited P61/P62
commands and the first compact P63 campaign export surface. They are not a
calibrated P63 scientific campaign result.

## 8. P61 proxy vs P62/P63 measured comparison

Known P61 proxy baseline from the Virtual Ratio Lab:

- smoke `ratio_effective`: approximately `28.361345`;
- standard `ratio_effective`: approximately `72.695035`;
- decision: `RECALIBRATE_P61_RATIO_COST_MODEL`.

P62 replaces proxy cost with measured `ratio_effective_per_byte` and real timing
fields, but remains conservative because workloads are deterministic internal
workloads and thresholds are not calibrated.

P63 comparison table: `PENDING_P63_IMPLEMENTATION`.

## 9. Campaign results

Measured P63 campaign export: `LOCAL_EXPORT_PASS`.

Prompt Codex 2 introduces:

- `campaign_report.json`;
- `runs.jsonl`;
- `runs.csv`;
- `summary.md`.

The values in Section 7 should still be treated as local command summaries, not
calibrated campaign evidence.

Prompt Codex 3 adds compact enriched run exports with:

- `campaign_id`;
- `mode`;
- `threshold_profile`;
- `operation_count`;
- `decision`;
- `timestamp_utc`.

Expected future campaign fields:

- campaign id;
- mode;
- repeat count;
- machine metadata;
- commit hash;
- persisted byte summaries;
- p50/p95/p99 timing summaries;
- `ratio_effective_per_byte` min/median/max;
- decision and decision reasons.

## 10. Robust statistics

Robust statistics are expected to summarize repeated runs without pretending
machine-local timings are universal.

Implemented first-pass statistics:

- min / median / max;
- mean;
- standard deviation;
- coefficient of variation.

Status: `PARTIAL_IMPLEMENTATION_CAMPAIGN_EXPORTS`.

Prompt Codex 3 adds explicit stability statuses:

- `ratio_stability_status`;
- `bytes_stability_status`;
- `timing_stability_status`;
- `campaign_stability_status`;
- `stability_reasons`.

The first versioned profile is `p63_conservative_v1`.

## 11. Threshold profile and decisions

P63 decisions to document and later implement:

- `VALIDATE_P63_MEASURED_RATIO_CALIBRATION`
- `RECALIBRATE_P63_THRESHOLDS`
- `RECALIBRATE_P63_WORKLOADS`
- `NO_GO_P63_MEASURED_RATIO_STABILITY`

The first implementation prefers recalibration unless thresholds, workloads,
metadata and repeated-run stability are all justified.

Threshold profile: `p63`.

Expected Prompt Codex 2 decision: `RECALIBRATE_P63_THRESHOLDS`.

Prompt Codex 3 profile:

- `profile_id = p63_conservative_v1`
- `alias = p63`
- `min_runs_required = 10`
- `max_ratio_cv = 0.05`
- `max_bytes_cv = 0.05`
- `max_timing_cv = 0.50`
- `require_machine_metadata = true`
- `require_campaign_exports = true`
- `require_realish_workloads = false`
- `allow_validate = false`

Expected Prompt Codex 3 decision remains:

```text
RECALIBRATE_P63_THRESHOLDS
```

`VALIDATE_P63_MEASURED_RATIO_CALIBRATION` remains blocked by
`allow_validate = false`.

## 12. Workloads / fixtures

Current measured workloads are inherited from P61/P62 and remain deterministic
internal workloads. They are useful for regression and calibration scaffolding,
but are not external scientific datasets.

Future P63 fixture work:

- name stable workload fixtures;
- document inclusion/exclusion criteria;
- separate smoke CI workloads from local standard campaigns;
- avoid heavy or timing-variable fixtures in CI.

Status: `PENDING_P63_IMPLEMENTATION`.

## 13. Gates P63

Proposed gates:

- `P63_G0_p61_proxy_baseline_preserved`
- `P63_G1_p62_measured_runs_available`
- `P63_G2_campaign_export_structured`
- `P63_G3_machine_metadata_present`
- `P63_G4_robust_statistics_present`
- `P63_G5_threshold_profile_declared`
- `P63_G6_decision_reasons_complete`
- `P63_G7_no_timing_golden`
- `P63_G8_no_atlas_grammar_change`
- `P63_G9_strict_p53_preserved`

Gate status: `PENDING_P63_IMPLEMENTATION`.

Prompt Codex 2 observed gate status:

- `P63_G0_p61_proxy_baseline_preserved`: `PASS`
- `P63_G1_p62_measured_runs_available`: `PASS`
- `P63_G2_campaign_export_structured`: `PASS`
- `P63_G3_machine_metadata_present`: `PASS_BEST_EFFORT`
- `P63_G4_robust_statistics_present`: `PASS`
- `P63_G5_threshold_profile_declared`: `PASS`
- `P63_G6_decision_reasons_complete`: `PASS`
- `P63_G7_no_timing_golden`: `PASS`
- `P63_G8_no_atlas_grammar_change`: `PASS`
- `P63_G9_strict_p53_preserved`: `PASS`

Final scientific gate status remains `RECALIBRATE_P63_THRESHOLDS`.

## Campaign comparison

Prompt Codex 3 adds:

```bash
cargo run -p atlas-cli -- ratio-campaign-compare \
  artifacts/p63/smoke/campaign_report.json \
  artifacts/p63/standard/campaign_report.json \
  --format json
```

Smoke and standard are different modes. A comparison between them should report
`DIFFERENT_MODES` while still emitting informative ratio and byte deltas.

## 14. Scientific interpretation

P63 should make the measured ratio analysis more scientific, but it must remain
honest about scope. Repeated deterministic workloads can calibrate the internal
measurement path; they do not prove external generality.

No industrial latency claim, universal compression claim, or scientific
validation claim is made by this initialized report.

## 15. Limitations

- P62 measurements are machine-local.
- P62 machine metadata is still sparse.
- Workloads are internal and deterministic.
- No external datasets are included yet.
- Thresholds are not calibrated.
- Timing values must not be goldenized.
- Large raw logs must not be committed.

## 16. Recommendation for P64

Recommended P64 direction after P63:

- strengthen external workload/fixture discipline;
- add calibrated threshold profiles only after enough campaign evidence;
- decide whether richer campaign exports are needed beyond the compact P63
  JSON/JSONL/CSV/Markdown surface;
- prepare a Results-oriented synthesis only after repo reports are complete.

## 17. Reproducibility notes

Local validation should be run in the isolated ASTRA environment:

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang
```

Reports should record branch, commit, machine context, commands and summarized
results. Raw logs should stay outside the repo unless a short excerpt is
essential.

## 18. Journal

- P63 report initialized: `PENDING_P63_IMPLEMENTATION`.
- Local validation for documentation step: `LOCAL_VALIDATION_PASS`.
- Prompt Codex 2 campaign export implementation:
  `PARTIAL_IMPLEMENTATION_CAMPAIGN_EXPORTS`.
- Prompt Codex 2 local validation: `LOCAL_VALIDATION_PASS_FOR_PROMPT_2`.
- Prompt Codex 3 calibration layer implementation:
  `LOCAL_VALIDATION_PASS_FOR_PROMPT_3`.
- CI validation: `TODO_AFTER_CI`.

# ASTRA-P77 — Test Stack Hygiene Audit

## Summary

P77 adds deterministic router calibration against the P76 routing oracle. No
Rust test file was deleted. The stack keeps earlier tests as non-regression
because P77 changes the router calibration layer, not the strict P53 parser,
old goldens, or historical runtime contracts.

## Test File Audit

| Test file | Role | Status | P77 action |
|---|---|---|---|
| `tests/atlas_tests.rs` | strict parser invalid corpus | UPDATE | Added eight P77 invalid contracts and raised invalid count to 84. |
| `tests/runtime_tests.rs` | runtime P53 smoke and invalid refusal | KEEP | Still protects strict runtime basics. |
| `tests/p57_tests.rs` | P57 report compatibility | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p58_tests.rs` | P58 local validation baseline | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p60_tests.rs` | bench report compatibility | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p61_tests.rs` | virtual ratio lab | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p62_tests.rs` | measured real ratio | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p63_tests.rs` | measured campaigns | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p64_tests.rs` | address-local realish workloads | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p65_tests.rs` | local actors and calibration | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p66_tests.rs` | address-fiber runtime | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p67_tests.rs` | address-fiber overhead calibration | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p68_tests.rs` | promotion gate | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p69_tests.rs` | representation contract | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p70_tests.rs` | contract replay and drift | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p71_tests.rs` | filesystem fiber store | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p72_tests.rs` | living procedural store | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p73_tests.rs` | cubical living store | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p74_tests.rs` | living topology search | HISTORICAL_NON_REGRESSION | Kept unchanged. |
| `tests/p75_tests.rs` | mixed-topology router | HISTORICAL_NON_REGRESSION | Kept unchanged and called explicitly. |
| `tests/p76_tests.rs` | routing oracle and virtual-space estimator | HISTORICAL_NON_REGRESSION | Kept unchanged and called explicitly. |
| `tests/p77_tests.rs` | oracle-calibrated router thresholds | KEEP | Added in P77. |

## P77-Specific Additions

`tests/p77_tests.rs` covers:

- calibration grid construction;
- P77 router threshold policy parsing;
- eight invalid P77 contracts;
- wrong-route grouping by corpus and feature;
- calibrated score and safety gates;
- promotion refusal when router/oracle, accuracy, or wrong-route cost gates fail;
- synthetic promotion gate success;
- export of calibrated policy and virtual-space metrics;
- CLI `routing-oracle-calibrate`;
- P76/P75/P74 non-regression by keeping the earlier test stack intact.

## Deleted Or Merged Tests

None.

## Decision Hygiene

P77 unit tests protect implementation invariants only. The P77 R&D decision is
based on living-memory routing-oracle calibration runs with source data near
10 MiB, reopen equivalence, guard refusal, measured cold/runtime cost and
explicit virtual-space equivalent metrics.

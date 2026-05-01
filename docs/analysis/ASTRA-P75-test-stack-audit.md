# ASTRA-P75 — Test Stack Hygiene Audit

## Summary

P75 adds a mixed-topology router while preserving the historical P61-P74
non-regression stack. No test file was deleted.

## Test file audit

| Test file | Role | Status | P75 action |
|---|---|---|---|
| `tests/atlas_tests.rs` | strict parser invalid corpus | UPDATE | Added 8 P75 invalid router contracts and raised invalid count to 68. |
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
| `tests/p74_tests.rs` | living topology search | HISTORICAL_NON_REGRESSION | Kept unchanged and called explicitly. |
| `tests/p75_tests.rs` | mixed-topology router | KEEP | Added in P75. |

## P75-specific additions

`tests/p75_tests.rs` covers:

- router policy enumeration;
- valid P75 `.atlas` parsing and typechecking;
- eight invalid P75 contracts;
- route selection for code, logs, JSON, CSV and guard;
- guard refusal without false gain;
- living benchmark ratio and baseline comparison;
- phase map GREEN/YELLOW/RED;
- reopen equivalence and `NO_DRIFT`;
- compact exports;
- CLI `mixed-topology-bench`;
- P74 non-regression path.

## Deleted or merged tests

None.

## Decision hygiene

The P75 unit tests protect implementation invariants only. The P75 decision is
based on local living-memory campaigns with 10,485,760 source bytes, not on unit
tests alone.

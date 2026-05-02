# ASTRA-P76 — Test Stack Hygiene Audit

## Summary

P76 adds a routing oracle, virtual-space metrics and a spec snapshot while
preserving the P61-P75 non-regression stack. No test file was deleted.

## Test File Audit

| Test file | Role | Status | P76 action |
|---|---|---|---|
| `tests/atlas_tests.rs` | strict parser invalid corpus | UPDATE | Added eight P76 invalid process contracts and raised invalid count to 76. |
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
| `tests/p75_tests.rs` | mixed-topology router | HISTORICAL_NON_REGRESSION | Kept unchanged and called explicitly. |
| `tests/p76_tests.rs` | routing oracle and virtual-space estimator | KEEP | Added in P76. |

## P76-Specific Additions

`tests/p76_tests.rs` covers:

- P76 routing oracle target enumeration;
- valid P76 `.atlas` parsing and typechecking;
- eight invalid P76 contracts;
- virtual-space metrics and equivalent-byte labeling;
- mixed-router versus oracle regret;
- CRUD/addressing metrics;
- phase map GREEN/YELLOW/RED;
- compact exports and CLI commands;
- P75 non-regression path;
- spec snapshot files.

## Deleted Or Merged Tests

None.

## Decision Hygiene

P76 unit tests protect implementation invariants only. The P76 decision is
based on living-memory routing oracle campaigns, not on non-living tests.

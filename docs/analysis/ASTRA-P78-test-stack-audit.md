# ASTRA-P78 — Test Stack Audit

P78 keeps the test stack hygiene rule active. No Rust test file was deleted in this milestone.

| Test file | Role | Status | Action |
|---|---|---|---|
| `tests/atlas_tests.rs` | strict parser invalid corpus and historical P53/P55 gates | UPDATE | Added P78 invalid fixtures to the historical invalid corpus count. |
| `tests/runtime_tests.rs` | runtime baseline behavior | KEEP | Still protects core local runtime behavior. |
| `tests/p57_tests.rs` | P57 report and diagnostic continuity | HISTORICAL_NON_REGRESSION | Kept as historical coverage. |
| `tests/p58_tests.rs` | invalid corpus validation | KEEP | Still protects refuse behavior. |
| `tests/p60_tests.rs` | P60 validation layer | HISTORICAL_NON_REGRESSION | Kept. |
| `tests/p61_tests.rs` | P61 virtual ratio layer | HISTORICAL_NON_REGRESSION | Kept. |
| `tests/p62_tests.rs` | P62 real ratio layer | HISTORICAL_NON_REGRESSION | Kept. |
| `tests/p63_tests.rs` | P63 measured ratio campaigns | HISTORICAL_NON_REGRESSION | Kept. |
| `tests/p64_tests.rs` | address-local generation | HISTORICAL_NON_REGRESSION | Kept. |
| `tests/p65_tests.rs` | local actor runtime | HISTORICAL_NON_REGRESSION | Kept. |
| `tests/p66_tests.rs` | address-fiber model | HISTORICAL_NON_REGRESSION | Kept. |
| `tests/p67_tests.rs` | address-fiber overhead calibration | HISTORICAL_NON_REGRESSION | Kept. |
| `tests/p68_tests.rs` | promotion evaluator | HISTORICAL_NON_REGRESSION | Kept. |
| `tests/p69_tests.rs` | representation contract | KEEP | Still protects contract syntax and cost accounting. |
| `tests/p70_tests.rs` | contract replay and drift detector | KEEP | Still protects declared-vs-measured logic. |
| `tests/p71_tests.rs` | filesystem fiber store | KEEP | Still protects real store roundtrip/retrieval/guard. |
| `tests/p72_tests.rs` | living procedural store | KEEP | Still protects close/reopen equivalence. |
| `tests/p73_tests.rs` | cubical living store | KEEP | Kept as topology regression coverage. |
| `tests/p74_tests.rs` | living topology search | KEEP | Kept as non-regression for topology candidates. |
| `tests/p75_tests.rs` | mixed topology router | KEEP | Still protects router behavior. |
| `tests/p76_tests.rs` | routing oracle and virtual metrics | KEEP | Still protects virtual bytes as equivalents. |
| `tests/p77_tests.rs` | oracle-calibrated router | KEEP | Still protects calibrated policy and wrong-route accounting. |
| `tests/p78_tests.rs` | level-1 virtual space and universal file store | ADD | New tests for Level1AddressSpace, estimator, universal fallback, CLI, invalids, and living metrics. |

No test was removed. The only existing test file modified is `tests/atlas_tests.rs`, because the invalid corpus grew from 84 to 92 files with the eight P78 invalid contracts.

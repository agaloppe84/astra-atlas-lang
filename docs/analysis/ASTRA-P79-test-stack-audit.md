# ASTRA-P79 - Test Stack Audit

P79 keeps the test stack hygiene rule active. No Rust test file was deleted.

| Test file | Role | Status | Action |
|---|---|---|---|
| `tests/atlas_tests.rs` | strict parser invalid corpus and historical P53/P55 gates | UPDATE | Added eight P79 invalid fixtures; invalid corpus coverage grows from 92 to 100 files. |
| `tests/runtime_tests.rs` | runtime baseline behavior | KEEP | Still protects core local runtime behavior. |
| `tests/p57_tests.rs` | P57 report and diagnostic continuity | HISTORICAL_NON_REGRESSION | Kept as historical coverage. |
| `tests/p58_tests.rs` | invalid corpus validation | KEEP | Still protects refuse behavior through the local invalid corpus script. |
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
| `tests/p75_tests.rs` | mixed topology router | KEEP | Still protects fiber topology routing behavior. |
| `tests/p76_tests.rs` | routing oracle and virtual metrics | KEEP | Still protects virtual bytes as equivalents. |
| `tests/p77_tests.rs` | oracle-calibrated fiber router | KEEP | Still protects calibrated policy and wrong-route accounting. |
| `tests/p78_tests.rs` | level-1 virtual space and universal file store | KEEP | Still protects Level1AddressSpace, universal fallback, and local-on-address metrics. |
| `tests/p79_tests.rs` | level-1 address router | ADD | New tests for routing rules, oracle wrong-route accounting, phase map, CLI, invalids, and P78/P77/P76 compatibility. |

No test was removed or weakened. P79 modifies only the historical invalid corpus test to include the new invalid files and adds `tests/p79_tests.rs`.


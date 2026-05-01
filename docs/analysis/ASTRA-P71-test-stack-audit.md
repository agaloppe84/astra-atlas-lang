# ASTRA-P71 — Test Stack Audit

## Scope

P71 keeps the P70 rule: every repo-first milestone audits the local Rust test
stack. The goal is not to delete old tests because they are old. The goal is to
keep tests that protect real invariants, update tests that track new invalid
corpus files, and document any deletion, merge, or recalibration.

## Summary

| status | count |
|---|---:|
| KEEP | 4 |
| UPDATE | 1 |
| MERGE | 0 |
| DELETE | 0 |
| HISTORICAL_NON_REGRESSION | 11 |

No Rust test file was deleted in P71.

## Audit table

| file | role | invariants protected | status | P71 action |
|---|---|---|---|---|
| tests/atlas_tests.rs | strict parser/export corpus | strict_p53, invalid corpus, canonical JSON | UPDATE | Added P71 invalid corpus entries. |
| tests/runtime_tests.rs | runtime smoke invariants | deterministic runtime execution | KEEP | No change. |
| tests/p57_tests.rs | historical report layer | P57 report non-regression | HISTORICAL_NON_REGRESSION | No change. |
| tests/p58_tests.rs | workload reports | P58 smoke/standard/invalid gates | HISTORICAL_NON_REGRESSION | No change. |
| tests/p60_tests.rs | CLI and diagnostics cleanup | P60 parser/CLI hygiene | HISTORICAL_NON_REGRESSION | No change. |
| tests/p61_tests.rs | virtual ratio proxy | P61 deterministic proxy ratio | HISTORICAL_NON_REGRESSION | No change. |
| tests/p62_tests.rs | measured ratio skeleton | P62 repeated measured runs without timing goldens | HISTORICAL_NON_REGRESSION | No change. |
| tests/p63_tests.rs | campaign exports/calibration | P63 profiles, summaries, registry, sets | HISTORICAL_NON_REGRESSION | No change. |
| tests/p64_tests.rs | realish address-local | P64 workloads and policy comparison | HISTORICAL_NON_REGRESSION | No change. |
| tests/p65_tests.rs | local actors | P65 actor overhead/calibration | HISTORICAL_NON_REGRESSION | No change. |
| tests/p66_tests.rs | address-fiber model | P66 fiber strategies and metrics | HISTORICAL_NON_REGRESSION | No change. |
| tests/p67_tests.rs | fiber overhead calibration | P67 Pareto/promotability gates | HISTORICAL_NON_REGRESSION | No change. |
| tests/p68_tests.rs | promotion gate | P68 paired evaluator/manifest | HISTORICAL_NON_REGRESSION | No change. |
| tests/p69_tests.rs | representation contract | P69 .atlas contract parse/typecheck/cost accounting | KEEP | No change; P71 reuses it. |
| tests/p70_tests.rs | contract replay | P70 declared vs measured drift and hygiene rule | KEEP | No change; P71 keeps replay compatibility. |
| tests/p71_tests.rs | filesystem fiber store | P71 manifest/store/decode/query/guard/cost ratios | KEEP | Added. |

## Changes justified

- `tests/atlas_tests.rs` was updated because the invalid corpus grew from 33 to
  38 files after adding five P71 invalid contract drift cases. This is not a
  semantic relaxation; it keeps the invariant that every invalid example is
  tracked by the strict invalid corpus test.
- `tests/p71_tests.rs` was added to cover the filesystem fiber store, measured
  filesystem cost breakdown, exact roundtrip metrics, retrieval metrics,
  incompressible guard behavior, declared-vs-measured drift, and CLI exports.

## Deletions

None.

## Recommendation

Keep P71 tests local and deterministic. Do not add long campaign runs to CI. In
P72, audit whether `tests/p71_tests.rs` should split store construction,
retrieval, and guard behavior if the store grows beyond this compact first
implementation.

# ASTRA-P72 — Test Stack Audit

## Scope

P72 keeps the repo-first test stack hygiene rule introduced in P70. The audit
checks whether local Rust tests still protect real invariants after P71 and
before the living procedural fiber store becomes a new validated surface.

No test was deleted in P72.

## Summary

| status | count |
|---|---:|
| KEEP | 5 |
| UPDATE | 1 |
| MERGE | 0 |
| DELETE | 0 |
| HISTORICAL_NON_REGRESSION | 11 |

## Audit table

| file | role | invariants protected | status | P72 action |
|---|---|---|---|---|
| `tests/atlas_tests.rs` | strict parser/export corpus | strict_p53, invalid corpus, canonical JSON, invalid examples | UPDATE | Added six P72 invalid lifecycle examples to the tracked invalid corpus. |
| `tests/runtime_tests.rs` | runtime smoke invariants | deterministic runtime execution | KEEP | No change. |
| `tests/p57_tests.rs` | historical report layer | P57 report non-regression | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p58_tests.rs` | workload reports | P58 smoke/standard/invalid gates | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p60_tests.rs` | CLI and diagnostics cleanup | P60 parser/CLI hygiene | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p61_tests.rs` | virtual ratio proxy | P61 deterministic proxy ratio | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p62_tests.rs` | measured ratio skeleton | P62 repeated measured runs without timing goldens | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p63_tests.rs` | campaign exports/calibration | P63 profiles, summaries, registry, sets | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p64_tests.rs` | realish address-local | P64 workloads and policy comparison | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p65_tests.rs` | local actors | P65 actor overhead/calibration | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p66_tests.rs` | address-fiber model | P66 fiber strategies and metrics | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p67_tests.rs` | fiber overhead calibration | P67 Pareto/promotability gates | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p68_tests.rs` | promotion gate | P68 paired evaluator/manifest | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p69_tests.rs` | representation contract | P69 .atlas contract parse/typecheck/cost accounting | KEEP | No change. |
| `tests/p70_tests.rs` | contract replay | P70 declared vs measured drift and hygiene rule | KEEP | No change. |
| `tests/p71_tests.rs` | filesystem fiber store | P71 manifest/store/decode/query/guard/cost ratios | KEEP | No change; P72 reuses P71 as seed store. |
| `tests/p72_tests.rs` | living procedural fiber store | lifecycle contract, cold/runtime split, close/reopen, compaction, adaptive encoding, guard, drift | KEEP | Added. |

## Changes justified

- `tests/atlas_tests.rs` was updated because the invalid corpus grew from 38 to
  44 files after adding six P72 lifecycle invalids. This preserves the invariant
  that new invalid examples are tracked by the strict invalid corpus test.
- `tests/p72_tests.rs` was added to cover the living store lifecycle, real
  filesystem cold/runtime layout, journal replay, reopen equivalence, compaction
  accounting, adaptive living fiber metrics, incompressible guard behavior, and
  CLI exports.

## Deletions

None.

## Recommendation

Keep P72 tests deterministic and local. Do not add living-store benchmark runs
to CI. In P73, reassess whether living store tests should split lifecycle,
filesystem cost, and adaptive encoding into separate files if the implementation
continues to grow.

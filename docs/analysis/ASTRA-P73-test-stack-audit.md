# ASTRA-P73 — Test Stack Audit

## Scope

P73 keeps the repo-first test stack hygiene rule. The audit checks whether the
local Rust tests still protect live invariants after P72 and before the cubical
fiber living store becomes a new experimental surface.

No Rust test file was deleted in P73.

## Summary

| status | count |
|---|---:|
| KEEP | 6 |
| UPDATE | 1 |
| MERGE | 0 |
| DELETE | 0 |
| HISTORICAL_NON_REGRESSION | 11 |

## Audit table

| file | role | invariants protected | status | P73 action |
|---|---|---|---|---|
| `tests/atlas_tests.rs` | strict parser/export corpus | strict_p53, invalid corpus, canonical JSON, invalid examples | UPDATE | Added eight P73 invalid cubical lifecycle examples to the tracked invalid corpus. |
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
| `tests/p69_tests.rs` | representation contract | P69 `.atlas` contract parse/typecheck/cost accounting | KEEP | No change. |
| `tests/p70_tests.rs` | contract replay | P70 declared vs measured drift and hygiene rule | KEEP | No change. |
| `tests/p71_tests.rs` | filesystem fiber store | P71 manifest/store/decode/query/guard/cost ratios | KEEP | No change. |
| `tests/p72_tests.rs` | living procedural fiber store | P72 lifecycle, reopen, compaction, adaptive encoding | KEEP | No change; P73 compares against this baseline. |
| `tests/p73_tests.rs` | cubical fiber living store | cells/faces/gluing, CRUD, reopen, corruption recovery, guard, P72 comparison | KEEP | Added. |

## Changes justified

- `tests/atlas_tests.rs` was updated because the invalid corpus grew from 44 to
  52 files after adding eight P73 cubical invalids. This keeps strict invalid
  corpus tracking intact.
- `tests/p73_tests.rs` was added to cover the cubical contract, six-face cell
  topology, gluing constraints, bounded face updates, face reads, compaction,
  reopen equivalence, corruption/recovery, guard behavior, P72 comparison, and
  CLI exports.

## Deletions

None.

## Recommendation

Keep P73 tests deterministic and local. Do not add long cubical campaigns to CI.
In P74, revisit whether corruption/recovery tests should split from topology
tests if recovery semantics become richer.

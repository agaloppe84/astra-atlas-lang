# ASTRA-P70 — Rust Test Stack Audit

## Purpose

P70 introduces test stack hygiene as a required local-first step. The goal is
not to delete old tests because they are old, but to identify which tests still
protect live project invariants, which ones are historical non-regression
guards, and which ones should be updated, merged or removed.

## Summary

| Status | Count |
|---|---:|
| KEEP | 4 |
| UPDATE | 1 |
| MERGE | 0 |
| DELETE | 0 |
| HISTORICAL_NON_REGRESSION | 10 |

No tests were deleted in P70. The existing stack still protects meaningful
historical invariants from P53 through P69. The only update is the invalid
corpus test table in `tests/atlas_tests.rs`, extended for the new P70 invalid
contract drift examples. P70 adds `tests/p70_tests.rs`.

## Test files

| Test file | Role actuel | Invariants proteges | Status | Action |
|---|---|---|---|---|
| `tests/atlas_tests.rs` | Base parser/typechecker and invalid corpus | `strict_p53`, guard refused, snapshot full refused, canonical export, invalid corpus | UPDATE | Extended invalid table from 28 to 33 cases with P70 drift mutants. |
| `tests/runtime_tests.rs` | Runtime smoke and snapshot/rebuild non-regression | runtime instantiation, metrics, invalid runtime refusal | KEEP | Keep unchanged. |
| `tests/p57_tests.rs` | Historical P57 report/golden guard | P57 JSON shape, guard refusal, golden stability | HISTORICAL_NON_REGRESSION | Keep unchanged. |
| `tests/p58_tests.rs` | Historical P58 workload/report guard | smoke/standard/ambitious workloads, P58 goldens, conservative decisions | HISTORICAL_NON_REGRESSION | Keep unchanged. |
| `tests/p60_tests.rs` | P60 benchmark JSON guard | bench JSON stability and mode decisions | HISTORICAL_NON_REGRESSION | Keep unchanged. |
| `tests/p61_tests.rs` | P61 proxy ratio lab guard | virtual ordering, cost ratios, refusal semantics, smoke golden | HISTORICAL_NON_REGRESSION | Keep unchanged. |
| `tests/p62_tests.rs` | P62 measured ratio guard | real timings non-null, persisted byte sums, repeated runs | HISTORICAL_NON_REGRESSION | Keep unchanged. |
| `tests/p63_tests.rs` | P63 campaign/export/registry guard | campaign exports, robust summary, threshold profile, set summary | HISTORICAL_NON_REGRESSION | Keep unchanged. |
| `tests/p64_tests.rs` | P64 address-local guard | realish workloads, policies, locality selectivity, exports | HISTORICAL_NON_REGRESSION | Keep unchanged. |
| `tests/p65_tests.rs` | P65 actors and calibration guard | actor strategies, overhead, cache, Pareto calibration | HISTORICAL_NON_REGRESSION | Keep unchanged. |
| `tests/p66_tests.rs` | P66 address-fiber guard | fiber strategies, actor-fiber overhead, CRUD/audit/compaction metrics | HISTORICAL_NON_REGRESSION | Keep unchanged. |
| `tests/p67_tests.rs` | P67 overhead calibration guard | calibration grid, Pareto front, safety factor, promotion condition | HISTORICAL_NON_REGRESSION | Keep unchanged. |
| `tests/p68_tests.rs` | P68 promotion evaluator guard | paired promotion gates, ablations, stress, phase map, manifest | HISTORICAL_NON_REGRESSION | Keep unchanged. |
| `tests/p69_tests.rs` | P69 contract guard | contract parser/typechecker, invalid contracts, cost breakdown, CLI | KEEP | Keep unchanged. |
| `tests/p70_tests.rs` | P70 replay and drift guard | contract replay, declared-vs-measured bytes, drift detector, exports | KEEP | Added in P70. |

## Obsolete tests

No obsolete test was found in this pass.

## Redundant tests

Some historical tests overlap in broad smoke coverage, but they protect
different iteration-level contracts and goldens. They are intentionally kept as
historical non-regression tests.

## Coupling risks

- P57/P58/P61 golden tests are intentionally coupled to stable JSON outputs.
- P62+ timing tests avoid exact timing assertions.
- P69/P70 contract tests are intentionally coupled to the promoted
  address-fiber contract syntax.

## Actions realized in P70

- Added `tests/p70_tests.rs`.
- Updated `tests/atlas_tests.rs` to include P70 invalid contract drift fixtures.
- No tests deleted.
- No tests merged.
- No CI-heavy tests added.

## Recommendation

Keep the current stack. For P71, add a lightweight matrix documenting which
tests are local-only, which are suitable for CI sanity, and which are historical
golden gates.

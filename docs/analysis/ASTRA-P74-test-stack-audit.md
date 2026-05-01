# ASTRA-P74 — Test Stack Hygiene Audit

Date: 2026-05-02

## Process rule

Starting with P70, every repo-first milestone audits the local Rust test stack.
Starting with P74, architectural ratio decisions must come from living-memory
campaigns around 10 MiB of source data. Smaller Rust tests remain unit,
parser/typechecker and non-regression tests.

## Summary

No useful test was deleted.

| Status | Count |
|---|---:|
| KEEP | 6 |
| UPDATE | 1 |
| MERGE | 0 |
| DELETE | 0 |
| HISTORICAL_NON_REGRESSION | 12 |

## Files

| Test file | Role | Invariants | Status | Action |
|---|---|---|---|---|
| `tests/atlas_tests.rs` | strict parser invalid corpus | strict_p53, guard refusal, snapshot refusal, invalid corpus | UPDATE | Added 8 P74 invalid contracts and raised invalid count to 60. |
| `tests/runtime_tests.rs` | runtime smoke | runtime/check/export baseline | KEEP | No change. |
| `tests/p57_tests.rs` | historical report non-regression | P57 JSON and guard/snapshot invariants | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p58_tests.rs` | workload/report non-regression | P58 workload modes and goldens | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p60_tests.rs` | benchmark report | P60 bench schema | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p61_tests.rs` | virtual ratio lab | deterministic proxy and guard/adversarial refusal | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p62_tests.rs` | measured real ratio | measured bytes and repeated runs | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p63_tests.rs` | campaign registry/set | campaign reports, registry, campaign sets | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p64_tests.rs` | address-local realish policy | policy comparison and address-local safety | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p65_tests.rs` | local actors/calibration | actor overhead and calibration Pareto | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p66_tests.rs` | address-fiber runtime | fiber strategies and actor-managed fiber | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p67_tests.rs` | address-fiber overhead calibration | promotion candidate remains conditional | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p68_tests.rs` | promotion evaluator | paired standard/ambitious gates | HISTORICAL_NON_REGRESSION | No change. |
| `tests/p69_tests.rs` | representation contract | .atlas contract and cost breakdown | KEEP | No change. |
| `tests/p70_tests.rs` | contract replay | drift detector and invalid contracts | KEEP | No change. |
| `tests/p71_tests.rs` | filesystem fiber store | real filesystem bytes, roundtrip/retrieval/guard | KEEP | No change. |
| `tests/p72_tests.rs` | living procedural store | close/reopen, journal replay, runtime/cold split | KEEP | No change. |
| `tests/p73_tests.rs` | cubical living store | cells/faces/gluing/recovery/P72 comparison | KEEP | No change. |
| `tests/p74_tests.rs` | living topology search | six topology kinds, 10 MiB-style target, phase map, guard, rankings | KEEP | Added in P74. |

## P74-specific additions

`tests/p74_tests.rs` covers:

- six topology kinds;
- specialized P74 `.atlas` parsing/typechecking;
- eight invalid P74 contracts;
- living benchmark source-byte target;
- `ratio_living`, `reopen_equivalence`, guard no false gain;
- phase map GREEN/YELLOW/RED production;
- topology ranking;
- topology overhead accounting;
- corpus-specific topology relevance;
- CLI `topology-living-bench`;
- P73 non-regression path.

## Deletions or merges

None.

## Rationale

The existing P61-P73 tests still protect real historical invariants. P74 adds
new living-memory topology coverage without deleting older non-regression tests.
The new tests do not justify the P74 decision by themselves; the decision is
based on local `topology-living-bench` campaigns with approximately 10 MiB of
source data.

# ASTRA-P76 — Routing Oracle, Virtual Space Metrics and ASTRA Spec Freeze Analysis

## 1. Executive summary

P76 adds an oracle for the mixed-topology router, explicit virtual-space
metrics, addressing/CRUD metrics and a working ASTRA/Atlas spec snapshot. The
decision remains conservative: `FREEZE_P76_ASTRA_CORE_SPEC_AND_RECALIBRATE_ROUTER`.

## 2. Position after P75

P75 showed that the mixed router keeps almost all hierarchical ratio while
reducing update/audit cost. It lacked a per-fiber oracle for route quality.

## 3. Process principles: living memory, max ratio, local virtual space

P76 keeps three process rules central: decisions come from living-memory runs,
`ratio_living` remains the main ratio, and virtual space is constructed locally
on address rather than globally materialized.

## 4. Routing oracle model

The oracle compares the router-selected topology against the best observed
topology for each corpus, feature, locality and update-pressure slice.

## 5. Wrong-route cost and regret

Regret is measured as ratio loss and extra update/audit cost. P76 reports
wrong-route count, wrong-route rate, worst wrong route and route accuracy.

## 6. Wide-spectrum living benchmark

The benchmark covers code, logs, JSON, CSV and guard corpora across clustered,
random, mixed and hotspot locality plus low, medium and high update pressure.
The run includes encode, open, read/query, update, delete, audit, compact,
close and reopen.

## 7. VirtualSpaceEstimator

The estimator calculates address, cell, fiber, face, edge and hyperedge counts,
then reports virtual declared/reachable/readable/updatable/safe/effective units.

## 8. Virtual space metrics

`virtual_declared_bytes_equivalent` and `virtual_effective_bytes_equivalent`
are equivalent materialization sizes. They are not stored bytes.

## 9. Addressing and CRUD metrics

P76 reports address lookup count, success rate, mean and p95 steps, bytes read,
CRUD counts, success rates, journal replay steps and compaction savings.
Machine-dependent timings are not goldenized.

## 10. Phase map

The phase map spans router policy, corpus, topology, locality and update
pressure. Cells are GREEN, YELLOW, RED or GREY.

## 11. Historical comparison P63-P76

The trajectory is: measured ratio layer, address-local, local actor,
address-fiber, contract, replay, filesystem store, living store, cubical
topology, topology search, mixed router and now routing oracle.

## 12. ASTRA core spec freeze

`docs/specs/ASTRA_CORE_SPEC_P76.md` freezes a working P77 snapshot of living
memory, local-on-address virtual space, cost accounting, guard rules and
decision gates.

## 13. Atlas language spec snapshot

`docs/specs/ATLAS_LANGUAGE_SPEC_P76.md` documents specialized P76 blocks and
forbids general-purpose control flow.

## 14. Patterns catalog

`docs/specs/ASTRA_PATTERNS_CATALOG_P76.md` catalogs promoted, candidate and
recalibrated patterns.

## 15. Standard campaign results

P76 routing oracle view
target source bytes                  : 10,485,760
actual source bytes                  : 10,485,760
virtual cell count                   : 10,000
virtual fiber count                  : 40,000
virtual effective bytes equivalent   : 63,532,744
cold persisted bytes                 : 1,019,318
runtime peak bytes                   : 512,520
ratio_living mixed router            : 4.955068
ratio_living oracle                  : 5.076273
router/oracle ratio                  : 0.976123
wrong route count                    : 570
wrong route cost                     : 2,034
routing accuracy                     : 0.931159
address lookup p95 steps             : 10.250
crud success rate                    : 1.000000
phase map green/yellow/red           : 148 / 196 / 136
recommended architecture             : mixed_router_with_oracle_feedback_and_hierarchical_default_for_P77
decision                             : FREEZE_P76_ASTRA_CORE_SPEC_AND_RECALIBRATE_ROUTER

The standard campaign completed locally in `real 0.42s`. The guard decision was
`NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`, `reopen_equivalence=true`, and
`drift_status=NO_DRIFT`.

## 16. Ambitious campaign results if executed

The ambitious campaign used the same wide-spectrum locality/update grid with
`cycles=25`, `queries=50000`, `updates=5000`, and `deletes=500`. It completed
locally in `real 0.44s`.

Ambitious summary:

- `ratio_living mixed router`: 4.955068
- `ratio_living oracle`: 5.076273
- `router/oracle ratio`: 0.976123
- `wrong_route_count`: 570
- `wrong_route_cost`: 2,034
- `routing_accuracy`: 0.931159
- `reopen_equivalence`: true
- `drift_status`: `NO_DRIFT`

The values match the standard ratio/regret view because the standard P76
wide-spectrum run already includes `update_pressure=high` and all locality
profiles. The ambitious run raises CRUD/replay volume and confirms stability,
not a different topology choice.

## 17. Router vs oracle

Promotion requires mixed/oracle ratio at least 0.95 and controlled wrong-route
cost. P76 intentionally does not promote if wrong routes remain material.

## 18. Decision

Decision after local campaign:
`FREEZE_P76_ASTRA_CORE_SPEC_AND_RECALIBRATE_ROUTER`.

Promotion is withheld because wrong routes remain non-zero even though the
mixed/oracle ratio is above the 0.95 gate.

## 19. Limitations

The oracle is deterministic and protocol-local. It is useful for route
calibration but is not a proof that the mixed router is globally optimal.

## 20. Recommendation for P77

Use P76 oracle regret to recalibrate deterministic router thresholds and
measure whether wrong-route cost can be reduced without sacrificing
`ratio_living`.

## 21. Reproducibility notes

Commands are recorded in the validation document and Results report. Exports
are under `artifacts/p76/` and ignored by Git.

## 22. Journal

- Added P76 module, CLI commands, `.atlas` syntax and invalid contracts.
- Added tests, specs, validation doc, analysis report and Results skeleton.

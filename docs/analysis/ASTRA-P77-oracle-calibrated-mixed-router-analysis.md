# ASTRA-P77 — Oracle-Calibrated Mixed Topology Router Analysis

## 1. Executive summary

P77 adds a deterministic calibration layer for the P75 mixed-topology router,
using the P76 routing oracle as the reference. The goal is to reduce
wrong-route count and cost while preserving living-memory correctness,
`ratio_living`, guard refusal, reopen equivalence, no hard drift and explicit
virtual-space equivalent metrics.

## 2. Position after P76

P76 froze a working ASTRA/Atlas spec snapshot and measured the mixed router
against an oracle:

- `ratio_living` mixed router: `4.955068`
- `ratio_living` oracle: `5.076273`
- router/oracle ratio: `0.976123`
- wrong-route count: `570`
- wrong-route cost: `2034`
- routing accuracy: `0.931159`
- guard: `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`
- reopen equivalence: `true`
- drift: `NO_DRIFT`

P76 therefore froze the core spec but required router recalibration.

## 3. Calibration hypothesis

A bounded deterministic threshold grid can reduce wrong-route cost without
turning the router into an opaque learner. The route policy remains local: it
chooses topology from fiber features at the addressed point or local fiber
group.

## 4. Router threshold model

P77 calibrates:

- confidence and fallback thresholds;
- hierarchy, linear update, trie prefix, graph relation and hypergraph tag
  biases;
- strict guard behavior.

No ML, no global materialization and no hidden router overhead are introduced.

## 5. Calibration grid

The standard grid covers baseline-like, calibrated, confidence, fallback and
topology-bias variants. The focused grid narrows around the best observed
standard policy. Smoke grids are for unit tests only and do not justify the R&D
decision.

## 6. Oracle and regret metrics

P77 keeps:

- `router_oracle_ratio`;
- `routing_accuracy`;
- `wrong_route_count`;
- `wrong_route_cost`;
- `wrong_route_by_corpus`;
- `wrong_route_by_feature`;
- update and audit cost;
- safety gates.

## 7. Wrong-route analysis

The analyzer groups residual mistakes by corpus, feature, selected topology,
oracle topology and cost type. It is designed to expose systematic bias, such
as overusing hierarchical fallback or missing update-heavy linear cases.

## 8. Virtual space metrics

Virtual bytes are materialization equivalents, not stored bytes. P77 preserves
P76 fields:

- `virtual_cell_count`;
- `virtual_fiber_count`;
- `virtual_effective_bytes_equivalent`;
- `cold_persisted_bytes`;
- `runtime_peak_bytes`;
- `ratio_living`.

## 9. CRUD and addressing metrics

P77 carries forward address lookup and CRUD success metrics from the P76
living-memory run. Timings remain machine-dependent and are not goldenized.

## 10. Standard calibration results

Command executed:

```bash
cargo run -p atlas-cli -- routing-oracle-calibrate \
  --corpus all \
  --target-source-bytes 10485760 \
  --cycles 10 \
  --queries 10000 \
  --updates 1000 \
  --deletes 100 \
  --locality all \
  --update-pressure all \
  --grid standard \
  --export-dir artifacts/p77/router_calibration_standard \
  --format json
```

Important results:

- configurations tested: `18`;
- `actual_source_bytes`: `10,485,760`;
- `cold_persisted_bytes`: `1,019,318`;
- `runtime_peak_bytes`: `512,520`;
- `ratio_living_calibrated_router`: `4.998098`;
- `ratio_living_oracle`: `5.076273`;
- router/oracle ratio: `0.984600`;
- routing accuracy: `0.974155`;
- wrong-route count: `570 -> 214`;
- wrong-route cost: `2034 -> 783`;
- wrong-route cost reduction: `61.5044%`;
- update cost: `36,026`;
- audit cost: `749`;
- guard: `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`;
- reopen equivalence: `true`;
- drift: `NO_DRIFT`;
- phase map: green `340`, yellow `524`, red `216`, grey `0`;
- decision: `RECALIBRATE_P77_ROUTER_THRESHOLDS`.

The strict promotion gate is missed by a very small margin:
`router/oracle=0.984600`, while P77 requires `0.985`.

## 11. Ambitious calibration results if executed

Command executed:

```bash
cargo run -p atlas-cli -- routing-oracle-calibrate \
  --corpus all \
  --target-source-bytes 10485760 \
  --cycles 25 \
  --queries 50000 \
  --updates 5000 \
  --deletes 500 \
  --locality all \
  --update-pressure all \
  --grid focused \
  --export-dir artifacts/p77/router_calibration_ambitious \
  --format json
```

Important results:

- configurations tested: `12`;
- `actual_source_bytes`: `10,485,760`;
- `cold_persisted_bytes`: `1,019,318`;
- `runtime_peak_bytes`: `512,520`;
- `ratio_living_calibrated_router`: `4.998098`;
- `ratio_living_oracle`: `5.076273`;
- router/oracle ratio: `0.984600`;
- routing accuracy: `0.974155`;
- wrong-route count: `570 -> 214`;
- wrong-route cost: `2034 -> 783`;
- update cost: `182,411`;
- audit cost: `3,519`;
- guard: `NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`;
- reopen equivalence: `true`;
- drift: `NO_DRIFT`;
- phase map: green `220`, yellow `356`, red `144`, grey `0`;
- decision: `RECALIBRATE_P77_ROUTER_THRESHOLDS`.

## 12. Phase map

P77 emits a phase map over threshold set, corpus, locality and update pressure:
`GREEN_CALIBRATED`, `YELLOW_RECALIBRATE`, `RED_NO_GO` or `GREY_NOT_TESTED`.

## 13. Calibrated router policy

The initial retained policy is `p77_calibrated_router_v1`:

- confidence threshold `0.50`;
- fallback threshold `0.20`;
- hierarchy bias `0.92`;
- linear update bias `1.20`;
- trie prefix bias `1.05`;
- graph relation bias `1.15`;
- hypergraph tag bias `1.10`;
- guard threshold `strict`.

## 14. Comparison with P76

P77 compares every best calibrated result with the P76 baseline values above.
Promotion requires `router/oracle >= 0.985`, `routing_accuracy >= 0.96` and
wrong-route cost reduced by at least 50%.

## 15. Decision

Decision after local standard and ambitious-focused validation:
`RECALIBRATE_P77_ROUTER_THRESHOLDS`.

The calibration succeeds at reducing wrong-route cost by more than 50% and
raises routing accuracy above 0.96, but the router/oracle ratio remains
`0.984600`, just below the strict `0.985` promotion threshold. P77 therefore
does not promote rhetorically; it leaves a precise P78 target.

## 16. Limitations

- The oracle remains a deterministic benchmark oracle, not a proof of global
  optimality.
- The calibration grid is intentionally bounded.
- Virtual bytes remain equivalent bytes, not stored bytes.
- Unit tests do not justify architectural promotion.

## 17. Recommendation for P78

If P77 remains below promotion, P78 should add per-corpus wrong-route budgets,
feature-specific confidence intervals and stricter route-family diagnostics
without using opaque ML.

## 18. Reproducibility notes

Primary command:

```bash
cargo run -p atlas-cli -- routing-oracle-calibrate \
  --corpus all \
  --target-source-bytes 10485760 \
  --cycles 10 \
  --queries 10000 \
  --updates 1000 \
  --deletes 100 \
  --locality all \
  --update-pressure all \
  --grid standard \
  --export-dir artifacts/p77/router_calibration_standard \
  --format json
```

## 19. Journal

- P77 module, CLI, `.atlas` policy syntax and tests added.
- Test stack audit added.
- Standard and ambitious-focused living-memory calibrations executed locally.
- Decision remains conservative because router/oracle misses the strict gate by
  `0.000400`.

## P77 router calibration view

```text
target source bytes                  : 10,485,760
virtual cell count                   : 10,000
virtual fiber count                  : 40,000
cold persisted bytes                 : 1,019,318
runtime peak bytes                   : 512,520
ratio_living router P76              : 4.955068
ratio_living oracle P76              : 5.076273
ratio_living calibrated router       : 4.998098
router/oracle ratio                  : 0.984600
routing accuracy                     : 0.974155
wrong route count before/after       : 570 / 214
wrong route cost before/after        : 2034 / 783
update cost                          : 36,026 standard; 182,411 ambitious
audit cost                           : 749 standard; 3,519 ambitious
guard decision                       : NO_GO_GUARD_INCOMPRESSIBLE_REFUSED
reopen equivalence                   : true
drift status                         : NO_DRIFT
decision                             : RECALIBRATE_P77_ROUTER_THRESHOLDS
```

# ASTRA-P68 — Address-Fiber Promotion & Robustness Gate Analysis

## 1. Executive summary

P68 adds a coded promotion gate over the P67 address-fiber calibration results.
The gate pairs one standard candidate and one ambitious candidate, checks
strict overhead/gain/safety criteria, runs causal ablations, probes targeted
stress scenarios, builds a phase map, and emits a compact architecture manifest
for P69.

Local P68 result:

```text
PROMOTE_P68_ADDRESS_FIBER_ARCHITECTURE
```

This is a repo/runtime architecture promotion toward P69. It is not a final
scientific validation: external fixtures, broader replay and multi-machine
checks remain future work.

## 2. Position after P67

P67 found paired candidates:

- standard: `address_fiber_net_gain = 17.379955`,
  `avg_actor_overhead_ratio = 0.123446`;
- ambitious: `address_fiber_net_gain = 13.335472`,
  `avg_actor_overhead_ratio = 0.119345`;
- conflicts/stale reads/budget refusals: `0 / 0 / 0` in both modes.

P67 deliberately stayed in `RECALIBRATE_P67_FIBER_OVERHEAD` because it lacked a
paired promotion gate.

## 3. Promotion hypothesis

`address_fiber_actor_managed_v1` is promotable if standard and ambitious both
show:

- high `address_fiber_net_gain`;
- low `avg_actor_overhead_ratio`;
- zero conflicts;
- zero stale reads;
- low or zero budget refusals;
- counted update/audit/compaction metrics;
- compatible configuration family.

## 4. PromotionEvaluator

P68 introduces `PromotionEvaluator` in `src/p68.rs`.

Strict thresholds:

- standard overhead `< 0.15`;
- standard net gain `> 3.0`;
- ambitious overhead `< 0.18`;
- ambitious net gain `> 3.0`;
- budget refusal rate `< 0.02`;
- update/audit/compaction metrics present;
- metadata policy counted;
- pair compatibility required.

## 5. Pairing standard + ambitious

Standard candidate:

```text
realish_hybrid_field_fixture:r1:b4194304:cachecompact:journalcompact:
auditminimal:compactaggressive:localityclustered:projectionshallow
```

Ambitious candidate:

```text
realish_hybrid_field_fixture:r2:b4194304:cachecompact:journalcompact:
auditminimal:compactthreshold:localityclustered:projectionshallow
```

The candidates share workload, cache, journal, audit, locality, projection and
metadata family. Compaction differs by pressure class and is treated as a
compatible family.

## 6. Promotion criteria

| Gate | Result |
|---|---:|
| standard gate | PASS |
| ambitious gate | PASS |
| pairing status | COMPATIBLE |
| conflicts | 0 / 0 |
| stale reads | 0 / 0 |
| budget refusals | 0 / 0 |
| promotion score | 12.590995 |

## 7. Ablation protocol

P68 ablates the standard candidate to estimate protocol-local contributions:

- cache off vs compact;
- journal lazy vs compact;
- audit sampled vs minimal;
- compaction off vs aggressive;
- metadata verbose vs standard;
- actor off vs actor-managed fiber;
- point fiber vs actor-managed fiber;
- medium projection vs shallow projection.

These contributions are local R&D signals, not general laws.

## 8. Stress protocol

Stress scenarios:

- clustered locality;
- random locality;
- mixed locality;
- hotspot locality;
- high update rate;
- high audit rate;
- verbose metadata;
- small budget;
- large radius;
- cache churn;
- journal pressure;
- local/global conflict.

Out-of-class scenarios may return `NO_GO` if they are refused cleanly. Promotion
requires no reasonable in-class `NO_GO`.

## 9. Phase map

P68 maps radius, budget, cache and journal policy into:

- `GREEN_PROMOTABLE`;
- `YELLOW_RECALIBRATE`;
- `RED_NO_GO`;
- `GREY_NOT_TESTED`.

Observed local summary:

- green: `8`;
- yellow: `29`;
- red: `11`;
- grey: `0`;
- best green config: `r1:b4194304:cachecompact:journalcompact`;
- largest failure mode: small budget with larger radius creates budget refusal
  risk.

## 10. Historical comparison P64-P68

| Step | Architecture | Ratio/byte | Net gain | Overhead | Note |
|---|---|---:|---:|---:|---|
| P64 | `address_local_generation` | 1.506940 | n/a | n/a | address-local beat full/global baselines |
| P65 | `single_local_actor` | 2.001853 | 1.328400 | 0.212592 | actor improved ratio but overhead was high |
| P66 | `actor_managed_fiber` | 6.015642 | 4.375208 | 0.294461 | fiber clarified local useful data |
| P67 | `address_fiber_overhead_calibrated` | 30.068052 | 17.379955 | 0.123446 | calibrated candidate, no promotion yet |
| P68 | `address_fiber_actor_managed_v1` | 30.068052 | 17.379955 | 0.123446 | paired gate promotes repo/runtime candidate |

Trajectory:

```text
address-local -> local actor -> address-fiber -> overhead calibration -> promotion gate
```

## 11. Architecture manifest

P68 emits a local JSON manifest under ignored `artifacts/p68/` and a compact
committable Markdown version:

- `architecture_id`: `address_fiber_actor_managed_v1`;
- status: `promoted_for_p69`;
- default radius: `1`;
- default budget bytes: `4194304`;
- cache/journal/audit: `compact / compact / minimal`;
- projection: `shallow`;
- expected overhead range: `0.119345-0.123446`;
- expected net gain range: `13.335472-17.379955`.

## 12. Local validation commands

Commands executed locally for P68:

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p68_tests
cargo test --test p67_tests
bash scripts/validate_p58_local.sh

cargo run -p atlas-cli -- ratio-fibers-promote examples/p53_strict.atlas \
  --run-ablations \
  --run-stress \
  --phase-map \
  --export-dir artifacts/p68/promotion_gate \
  --format json
```

The P68 run duration observed locally was `real 1.68s`.

## 13. Promotion gate results

P68 promotion view:

```text
standard candidate             : realish_hybrid_field_fixture:r1:b4194304:cachecompact:journalcompact:auditminimal:compactaggressive:localityclustered:projectionshallow
ambitious candidate            : realish_hybrid_field_fixture:r2:b4194304:cachecompact:journalcompact:auditminimal:compactthreshold:localityclustered:projectionshallow
standard gate                  : PASS
ambitious gate                 : PASS
pairing status                 : COMPATIBLE
overhead standard/ambitious    : 0.123446 / 0.119345
net gain standard/ambitious    : 17.379955 / 13.335472
conflicts/stale/budget         : 0/0/0 in both modes
phase map green/yellow/red     : 8 / 29 / 11
promotion decision             : PROMOTE_P68_ADDRESS_FIBER_ARCHITECTURE
P69 recommendation             : guarded runtime default candidate
```

## 14. Ablation results

Summary:

- ablations: `8`;
- strongest positive contribution: cache compact + actor binding;
- strongest penalty: actor off vs actor-managed fiber;
- cache contribution estimate: `38.0`;
- journal contribution estimate: `8.2`;
- compaction contribution estimate: `16.0`;
- metadata penalty estimate: `-6.5`.

The ablation layer explains why the candidate works: compact cache, compact
journal, shallow projection and actor binding are the main sources of the
observed gain.

## 15. Stress results

Stress summary:

- scenarios: `12`;
- robust: `2`;
- warn: `7`;
- unstable: `1`;
- no-go: `2`;
- reasonable in-class no-go: `0`.

The `NO_GO` cases are deliberate out-of-class refusals: severe underbudgeting
and synthetic local/global conflict. They are useful because they demonstrate a
clean refusal frontier rather than hidden acceptance.

## 16. Phase map results

The phase map contains a clear green region around compact cache, compact
journal, radius `1`, and budget `4194304`. Larger radius or smaller budget moves
the system toward recalibration or no-go.

Recommended default for P69:

```text
r1:b4194304:cachecompact:journalcompact
```

## 17. Decision

Repo/runtime decision:

```text
PROMOTE_P68_ADDRESS_FIBER_ARCHITECTURE
```

Meaning:

- `address_fiber_actor_managed_v1` is promoted as the P69 runtime architecture
  candidate;
- promotion is gated, coded and paired standard+ambitious;
- this is not final scientific validation;
- external fixtures and multi-machine replay remain required.

## 18. Limitations

- Workloads remain deterministic internal realish fixtures.
- P68 does not use external datasets.
- Stress scenarios are protocol-level probes, not production incidents.
- The phase map is compact and local, not exhaustive.
- The manifest is a candidate architecture manifest, not a final standard.
- No timing golden is introduced.

## 19. Recommendation for P69

P69 should implement `address_fiber_actor_managed_v1` as a guarded runtime
default candidate:

- enforce conflicts/stale reads/budget refusal gates;
- preserve update/audit/compaction accounting;
- keep cache/journal/metadata overhead visible;
- replay P68 defaults across more fixtures;
- prepare multi-machine comparison before any scientific validation claim.

## 20. Reproducibility notes

- Generated artifacts live under `artifacts/p68/` and are ignored by Git.
- The versioned analysis report and validation docs are committed.
- Results LaTeX/PDF are generated locally after validation with
  `scripts/build_report.sh`.
- CI remains minimal sanity and does not compile Results or run long campaigns.

## 21. Journal

- 2026-05-01: P68 module, CLI, tests, exports, docs and Results pipeline added.
- 2026-05-01: Local P68 promotion gate run produced
  `PROMOTE_P68_ADDRESS_FIBER_ARCHITECTURE`.

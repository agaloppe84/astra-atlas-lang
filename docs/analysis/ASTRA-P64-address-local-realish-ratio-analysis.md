# ASTRA-P64 — Address-Local Realish Ratio Analysis

## 1. Executive summary

ASTRA-P64 tests the next system hypothesis after P63: in an addressable
procedural representation, paid cost should not come from generating the whole
virtual space. The runtime should generate only a local neighborhood around a
requested address.

P64 introduces a lightweight `ratio-realish` command, four deterministic
realish workloads, three generation policies, compact local exports, and a
policy comparison surface. The observed local campaigns favor
`address_local_generation` on all four workloads, but the scientific decision
remains conservative:

```text
RECALIBRATE_P64_ADDRESS_LOCAL_RATIO_MODEL
```

No `.atlas` grammar change was made. `strict_p53` remains preserved. Invalid
examples and P61/P62/P63 goldens remain untouched.

## 2. Position after P63

P63 measured repeated local campaigns for `ratio-real` and introduced
campaign registries, campaign sets, robust summaries, and core virtual/real
ratio views. P63-v6 reported:

| metric | value |
|---|---:|
| virtual_declared_units | 34,320,000 |
| virtual_effective_units | 2,460,000 |
| total_persisted_bytes | 777,971 |
| ratio_effective_per_byte | 3.162072 |
| gain_vs_materialized | 352.918039 |
| effective_gain_vs_materialized | 25.296573 |
| total_runs | 150 |
| scientific decision | RECALIBRATE_P63_THRESHOLDS |

P64 does not replace that result. It asks whether the cost model improves when
the system pays for address-local generation instead of broader generation or
full materialization.

## 3. Central hypothesis: local generation around address

P64 explicitly tests the hypothesis:

> Dans un système procédural, on limite les coûts de calcul en générant
> seulement un endroit local autour d'une adresse précise dans l'espace
> virtuel.

For an address `x` in a virtual space `Omega`, the runtime generates only a
neighborhood `N(x, r)`, with lightweight accounting for cache, local index,
journal, audit, and metadata.

## 4. Workloads realish

P64 adds four deterministic, local-only fixtures. They are intentionally small
enough for local development and CI-safe tests, but they model more concrete
address shapes than the earlier proxy workloads.

| workload | address model | local generation |
|---|---|---|
| `realish_log_events` | timestamp bucket + service + request_id | time window around timestamp plus service prefix |
| `realish_sparse_csv` | row_id + column_group | row window plus active column groups |
| `realish_json_records` | record_id + projection path | record plus neighboring projected fields |
| `realish_hybrid_field_fixture` | tile / point address | local patch around tile with global/local proxy terms |

These are not external datasets and do not claim scientific representativeness
yet.

## 5. Generation policies

P64 compares three policies:

| policy | role |
|---|---|
| `full_materialization` | baseline that accounts for generating the whole declared space |
| `global_indexed_generation` | keeps broader global/indexed generated state |
| `address_local_generation` | generates only neighborhoods around requested addresses |

The CLI spelling is:

```bash
cargo run -p atlas-cli -- ratio-realish examples/p53_strict.atlas \
  --workload all \
  --policy all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --neighborhood-radius 3 \
  --export-dir artifacts/p64/realish_standard_policy_compare \
  --format json
```

## 6. Metrics

Each workload/policy entry reports:

- virtual space: declared, reachable, readable, updatable, safe, effective,
  generated, generated per query, locality selectivity;
- real cost: payload, index, journal, manifest, audit, metadata, total persisted
  bytes;
- ratios: declared/effective/generated per byte, gain against materialization,
  local generation gain against full materialization;
- locality: queries, unique addresses touched, radius, cache hit rate,
  materialized units per query;
- safety: refused queries, guard refusals, unsafe local generation count,
  read/update/audit success rates;
- local timing observation in nanoseconds, without timing goldens.

## 7. Local validation commands

Executed locally in the ASTRA isolated Rust environment:

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p63_tests
bash scripts/validate_p58_local.sh

cargo run -p atlas-cli -- ratio-realish examples/p53_strict.atlas \
  --workload all \
  --policy all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --neighborhood-radius 3 \
  --export-dir artifacts/p64/realish_standard_policy_compare \
  --format json

cargo run -p atlas-cli -- ratio-realish examples/p53_strict.atlas \
  --workload all \
  --policy all \
  --mode ambitious \
  --runs 50 \
  --queries 5000 \
  --neighborhood-radius 5 \
  --export-dir artifacts/p64/realish_ambitious_policy_compare \
  --format json
```

Validation summary:

- `cargo fmt --all -- --check`: PASS
- `cargo build --workspace`: PASS
- `cargo test --workspace`: PASS
- `cargo test --test p63_tests`: PASS, 18 tests
- `scripts/validate_p58_local.sh`: PASS, invalid corpus refused 21/21
- `tests/p64_tests.rs`: PASS, 10 tests in the workspace run

Campaign durations observed with `/usr/bin/time -p`:

- standard campaign: `real 0.26s`
- ambitious campaign: `real 0.31s`

The timings are local observations only and are not scientific latency claims.

## 8. Results standard campaign

Standard campaign parameters:

- mode: `standard`
- runs: `30`
- query_count: `1000`
- neighborhood_radius: `3`
- workload_policy_metrics: `12`
- policy comparisons: `4`
- decision: `RECALIBRATE_P64_ADDRESS_LOCAL_RATIO_MODEL`

Address-local aggregate view across the four workloads:

| campaign_set | mode | campaigns | total_runs | virtual_declared | virtual_generated_local | virtual_effective | real_bytes | ratio_effective_per_byte | gain_vs_materialized | effective_gain_vs_materialized | decision |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| realish_standard_policy_compare | standard | 1 | 30 | 132,000,000 | 2,296,000 | 22,600,000 | 14,997,283 | 1.506940 | 70.412754 | 12.055517 | RECALIBRATE_P64_ADDRESS_LOCAL_RATIO_MODEL |

Address-local view:

```text
virtual declared        : 132,000,000 units
virtual generated local :   2,296,000 units
virtual effective       :  22,600,000 units
real persisted bytes    :  14,997,283 bytes
locality selectivity    : 0.017394
effective gain          : 12.055517
best policy             : address_local_generation on 4/4 workloads
```

Policy comparison:

| workload | full ratio | global ratio | address-local ratio | address-local selectivity | address-local gain vs full | policy decision |
|---|---:|---:|---:|---:|---:|---|
| realish_log_events | 0.036443 | 0.099752 | 1.193000 | 0.028000 | 32.735739 | ADDRESS_LOCAL_STRONG |
| realish_sparse_csv | 0.018494 | 0.140576 | 1.948657 | 0.014000 | 105.366786 | ADDRESS_LOCAL_STRONG |
| realish_json_records | 0.033082 | 0.056021 | 0.678860 | 0.035000 | 20.520760 | ADDRESS_LOCAL_STRONG |
| realish_hybrid_field_fixture | 0.018517 | 0.127806 | 1.903213 | 0.015750 | 102.781363 | ADDRESS_LOCAL_STRONG |

## 9. Results ambitious campaign if executed

Ambitious campaign parameters:

- mode: `ambitious`
- runs: `50`
- query_count: `5000`
- neighborhood_radius: `5`
- workload_policy_metrics: `12`
- policy comparisons: `4`
- decision: `RECALIBRATE_P64_ADDRESS_LOCAL_RATIO_MODEL`

Address-local aggregate view across the four workloads:

| campaign_set | mode | campaigns | total_runs | virtual_declared | virtual_generated_local | virtual_effective | real_bytes | ratio_effective_per_byte | gain_vs_materialized | effective_gain_vs_materialized | decision |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| realish_ambitious_policy_compare | ambitious | 1 | 50 | 132,000,000 | 18,040,000 | 22,600,000 | 119,212,435 | 0.189578 | 8.858136 | 1.516620 | RECALIBRATE_P64_ADDRESS_LOCAL_RATIO_MODEL |

Address-local view:

```text
virtual declared        : 132,000,000 units
virtual generated local :  18,040,000 units
virtual effective       :  22,600,000 units
real persisted bytes    : 119,212,435 bytes
locality selectivity    : 0.136667
effective gain          : 1.516620
best policy             : address_local_generation on 4/4 workloads
```

Policy comparison:

| workload | full ratio | global ratio | address-local ratio | address-local selectivity | address-local gain vs full | policy decision |
|---|---:|---:|---:|---:|---:|---|
| realish_log_events | 0.031879 | 0.022722 | 0.149664 | 0.220000 | 4.694771 | ADDRESS_LOCAL_PROMISING |
| realish_sparse_csv | 0.017846 | 0.019862 | 0.245105 | 0.110000 | 13.734700 | ADDRESS_LOCAL_STRONG |
| realish_json_records | 0.027280 | 0.015532 | 0.085248 | 0.275000 | 3.124974 | ADDRESS_LOCAL_PROMISING |
| realish_hybrid_field_fixture | 0.018025 | 0.017886 | 0.240142 | 0.123750 | 13.322457 | ADDRESS_LOCAL_STRONG |

## 10. Policy comparison

The observed best policy is `address_local_generation` for every P64 workload
in both standard and ambitious local campaigns.

This is a system observation, not a final scientific validation. It is
consistent with the P64 hypothesis because address-local generation pays for a
small generated neighborhood instead of full declared space. The result is
strongest in the standard campaign where locality selectivity remains below
4 percent on every workload.

## 11. Ratio/gain view

The standard campaign is the clearest P64-v1 signal:

```text
Virtual/real compression view
[declared  : ##########] 132,000,000 units
[generated : ----------]   2,296,000 units
[effective : ##--------]  22,600,000 units
[real cost : #---------]  14,997,283 bytes
ratio_effective_per_byte = 1.506940
gain_vs_materialized     = 70.412754
effective_gain           = 12.055517
```

The ambitious campaign touches more local space and therefore pays more real
bytes:

```text
Virtual/real compression view
[declared  : ##########] 132,000,000 units
[generated : #---------]  18,040,000 units
[effective : ##--------]  22,600,000 units
[real cost : ###-------] 119,212,435 bytes
ratio_effective_per_byte = 0.189578
gain_vs_materialized     = 8.858136
effective_gain           = 1.516620
```

## 12. Conceptual interpretation

P64 supports the idea that procedural addressability should be evaluated by
how much effective virtual space remains addressable after local generation
costs are paid. The declared space alone is not sufficient; the meaningful
quantity is the effective space that remains reachable, readable, updatable,
safe, auditable, and locally generable.

The standard P64 campaign shows a large difference between declared space and
generated local space. This is exactly the intended distinction: a procedural
system can declare a broad address space while paying only for local access
paths.

## 13. Technical/system interpretation

The implementation is intentionally compact:

- `src/p64.rs` contains realish workload specs, generation policies, policy
  metrics, comparisons, JSON/Markdown serialization, and compact exports.
- `ratio-realish` validates the input `.atlas` program through the existing
  parser/typechecker path before running P64 measurements.
- `artifacts/p64/` receives generated campaign files and is ignored by Git.
- CI is not extended with long campaigns.

P64 does not implement external datasets, a persistent P64 registry, or a
multi-machine protocol. Those remain future work.

## 14. Decisions

Repository/system implementation decision:

```text
PROMOTE_P64_ADDRESS_LOCAL_FOR_P65_CANDIDATE
```

This means the implementation is strong enough to motivate P65 design work.
It does not mean the scientific claim is validated.

Scientific decision:

```text
RECALIBRATE_P64_ADDRESS_LOCAL_RATIO_MODEL
```

Reasons:

- fixtures are deterministic and local-only;
- only one local machine was used;
- no external dataset was added;
- timings are local observations and not goldenized;
- no calibrated P64 threshold profile exists yet;
- address-local dominates these fixtures, but the result must be reproduced
  across richer workloads and machines.

## 15. Limitations

- No external or production dataset.
- No multi-machine campaign.
- No calibrated P64 threshold profile.
- No final validation decision.
- P64 campaign set is represented by `address_local_summary` inside each
  comparative campaign over all workloads and policies; a true multi-campaign
  P64 registry is left for P64-2.
- Timing values are local and machine-dependent.
- The generated/persisted byte model is structural and deterministic, not a
  replacement for full production storage accounting.

## 16. Recommendation for P65

P65 should promote address-local generation into a more explicit calibration
track:

- add a P64/P65 campaign registry for multiple realish campaign files;
- add external or semi-external fixtures without bloating the repo;
- calibrate P64 threshold profiles;
- test multi-machine reproducibility;
- strengthen update and audit semantics around local neighborhoods;
- compare address-local with a real persisted cache/index implementation.

## 17. Reproducibility notes

Generated campaign outputs are local artifacts:

```text
artifacts/p64/realish_standard_policy_compare/
artifacts/p64/realish_ambitious_policy_compare/
```

They are ignored by Git. The durable repo artifact is this Markdown analysis,
which summarizes commands, decisions, limits, and compact results.

To replay locally:

```bash
source ~/Astra/activate_astra.sh
cd ~/Astra/astra-atlas-lang

cargo run -p atlas-cli -- ratio-realish examples/p53_strict.atlas \
  --workload all \
  --policy all \
  --mode standard \
  --runs 30 \
  --queries 1000 \
  --neighborhood-radius 3 \
  --export-dir artifacts/p64/realish_standard_policy_compare \
  --format json
```

## 18. Journal

- P64-1: added `ratio-realish`, four realish workloads, three generation
  policies, P64 JSON/Markdown reports, compact exports, tests, and local
  standard/ambitious campaigns.
- Local validation: `cargo fmt`, `cargo build`, `cargo test`, targeted
  `p63_tests`, and `validate_p58_local.sh` passed.
- Decision remains `RECALIBRATE_P64_ADDRESS_LOCAL_RATIO_MODEL`.

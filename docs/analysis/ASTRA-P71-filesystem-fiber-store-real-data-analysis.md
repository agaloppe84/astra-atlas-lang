# ASTRA-P71 — Filesystem Fiber Store and Real Data Encode/Decode Analysis

## 1. Executive summary

P71 introduces a local filesystem Fiber Store for address-fiber procedural
storage. The store writes real files, measures real filesystem bytes, decodes
addressed fibers, runs deterministic retrieval queries, and checks an
incompressible guard corpus under a hard 10 MiB budget.

Local validation status: PASS.

Decision from the standard campaign:
`RECALIBRATE_P71_CONTRACT_COST_MODEL`.

## 2. Position after P70

P69 promoted the `.atlas` address-fiber representation contract. P70 replayed
that contract across address-fiber fixtures and found no hard drift:

- declared contract bytes: `176128`;
- measured median bytes: `180224`;
- max delta percent: `3.488372`;
- drift status: `NO_DRIFT`;
- invalides P70: `5/5` refused;
- decision: `RECALIBRATE_P70_CONTRACT_DRIFT`.

P71 moves from contract replay to a real local store on filesystem.

## 3. Central question: what can 10 MiB of procedural fiber store hold?

P71 asks how much local data can be made addressable, decodable, verifiable and
useful with a budget of `10,485,760` bytes. The answer must count contract,
generator, parameters, dictionaries, index, residuals, journal, checksums,
audit metadata, actor state, and raw fallback bytes.

## 4. FiberStore model

The P71 `FiberStore` contains:

- store id;
- hard byte budget;
- P69 contract id;
- address space and fiber schema;
- codec id;
- manifest;
- address index;
- dictionaries;
- generators;
- residuals;
- journals;
- checksums;
- audit metadata;
- actor policy;
- measured filesystem cost breakdown.

## 5. Real data corpora

| corpus | source kind | expected behavior |
|---|---|---|
| `real_code_corpus` | repo filesystem | exact roundtrip required |
| `realish_logs_corpus` | deterministic realish | template/delta exact recovery |
| `realish_json_records` | deterministic realish | dictionary/residual exact recovery |
| `sparse_csv_table` | deterministic realish | sparse projection exact recovery |
| `incompressible_guard_blob` | deterministic pseudorandom guard | refused or no false gain |

Measured corpus sizes in the standard run:

| corpus | source bytes | records |
|---|---:|---:|
| `real_code_corpus` | 1,248,734 | 100 |
| `realish_logs_corpus` | 10,912 | 96 |
| `realish_json_records` | 6,222 | 64 |
| `sparse_csv_table` | 1,102 | 72 |
| `incompressible_guard_blob` | 65,536 | 1 |

## 6. Encoding policies

P71 exposes these policies:

- `raw_fiber`;
- `dictionary_fiber`;
- `template_delta_fiber`;
- `grammar_token_fiber`;
- `sparse_projection_fiber`;
- `generated_plus_residual_fiber`;
- `refused_fiber`.

The first P71 implementation stays conservative: exact local data is counted as
residual or policy-specific stored bytes. The incompressible guard is refused.

## 7. Decoding path

Decoding validates exact recoverability through fiber records and checksums. No
timing golden is used.

## 8. Retrieval path

Retrieval uses deterministic known-answer queries over useful records. It
reports precision, recall, exact match rate, decoded fibers per query, and
bytes read per query.

## 9. Filesystem cost breakdown

The store measures filesystem bytes with metadata for:

- manifest;
- contract;
- generators;
- parameters if available;
- dictionaries;
- index;
- residuals;
- journal;
- checksums;
- audit metadata;
- actor state;
- raw fallback.

Reports generated beside the store are not counted as useful storage cost.

## 10. Budget 10 MiB

Budget: `10,485,760` bytes.

Budget result:

- `budget_bytes = 10,485,760`;
- `filesystem_store_bytes = 747,457`;
- `budget_used_percent = 7.128305`;
- `budget_pass = true`.

## 11. Exact roundtrip results

Roundtrip results:

- `sample_count = 332`;
- `exact_roundtrip_count = 332`;
- `missing_fiber_count = 0`;
- `corrupted_fiber_count = 0`;
- `checksum_pass_rate = 1.000000`;
- `roundtrip_success_rate = 1.000000`;
- `exact_recoverable_bytes = 1,266,970`.

## 12. Retrieval results

Retrieval results:

- `query_count = 82`;
- `successful_queries = 82`;
- `precision = 1.000000`;
- `recall = 1.000000`;
- `exact_match_rate = 1.000000`;
- `useful_retrieved_bytes = 79,972`;
- `query_success_rate = 1.000000`.

## 13. Guard incompressible results

Guard result:

- `guard_source_bytes = 65,536`;
- `guard_store_bytes = 0`;
- `guard_decision = NO_GO_GUARD_INCOMPRESSIBLE_REFUSED`;
- `guard_refused = true`;
- `guard_no_false_gain = true`.

## 14. Contract replay integration

P71 reuses `examples/valid/p69_address_fiber_contract.atlas`.

Contract result:

- `contract_check_pass = true`;
- `all_storage_counted = true`;
- `hidden_storage_risk = low`;
- `declared_total_bytes = 176,128`;
- `measured_total_store_bytes = 747,457`;
- `delta_percent = 324.382835`;
- `drift_status = HARD_DRIFT`.

## 15. Declared vs measured bytes

P71 exposes a hard contract-cost drift: the P69 contract declares `176,128`
bytes, while the real filesystem store pays `747,457` bytes. The difference is
expected for this first real-data store because the contract was calibrated on
compact address-fiber fixtures, not on repository-scale text plus generated
corpora. This prevents validation and forces a P71 cost-model recalibration.

Filesystem cost breakdown:

| category | bytes |
|---|---:|
| manifest | 110 |
| contract | 119 |
| generator | 57 |
| parameters | 0 |
| dictionary | 66 |
| index | 39,543 |
| residuals | 654,127 |
| journal | 21,312 |
| checksums | 16,260 |
| audit metadata | 89 |
| actor state | 15,698 |
| raw fallback | 76 |
| report overhead included in store cost | 0 |
| total store | 747,457 |

## 16. Ratio/gain view

P71 fiber store view

```text
budget bytes                         : 10,485,760
source dataset bytes                  : 1,332,506
filesystem store bytes                : 747,457
exact recoverable bytes               : 1,266,970
useful retrieved bytes                : 79,972
exact bytes / store byte              : 1.695041
useful retrieved bytes / store byte   : 0.106992
procedural store gain vs raw          : 1.782719
guard decision                        : NO_GO_GUARD_INCOMPRESSIBLE_REFUSED
roundtrip success rate                : 1.000000
retrieval success rate                : 1.000000
decision                              : RECALIBRATE_P71_CONTRACT_COST_MODEL
```

The procedural code does not store a global table. It stores a contract,
generators, parameters, dictionaries, indexes, residuals, journals and metadata
that allow the runtime to regenerate useful local fibers.

## 17. Test stack hygiene P71

See `docs/analysis/ASTRA-P71-test-stack-audit.md`.

P71 adds `tests/p71_tests.rs` and updates `tests/atlas_tests.rs` to track five
new invalid P71 contracts. No test file is deleted.

## 18. Local validation commands

```bash
git pull --ff-only
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo test --test p71_tests
cargo test --test p70_tests
bash scripts/validate_p58_local.sh

cargo run -p atlas-cli -- fiber-store-bench \
  --corpus all \
  --budget-bytes 10485760 \
  --runs 30 \
  --queries 1000 \
  --export-dir artifacts/p71/fiber_store_standard \
  --format json

for f in examples/invalid/p71_*.atlas; do
  cargo run -p atlas-cli -- check "$f"
done

bash scripts/build_report.sh reports/P71/RPA_ASTRA-P71-Results_filesystem-fiber-store-real-data_v1.0_2026-05-01.tex
```

Validation results:

| command | result | important output |
|---|---|---|
| `git pull --ff-only` | PASS | already up to date |
| `cargo fmt --all -- --check` | PASS | formatting OK |
| `cargo build --workspace` | PASS | dev build OK |
| `cargo test --workspace` | PASS | full workspace tests OK |
| `cargo test --test p71_tests` | PASS | 14 tests passed |
| `cargo test --test p70_tests` | PASS | 13 tests passed |
| `bash scripts/validate_p58_local.sh` | PASS | 38 invalid examples refused |
| `fiber-store-bench` | PASS | `real 0.31s` |
| P71 invalid loop | PASS | 5/5 invalid P71 contracts refused |
| `build_report.sh` | PASS | Tectonic, PDF 38K |

## 19. Decision

`RECALIBRATE_P71_CONTRACT_COST_MODEL`.

The store passes budget, roundtrip, retrieval and guard behavior, but the
declared-vs-measured byte drift is `HARD_DRIFT`. P71 therefore cannot validate
the filesystem fiber store yet. The next step must recalibrate the
representation contract cost model for real local stores.

## 20. Limitations

- The corpus is local and deterministic; no external dataset is used.
- The first filesystem store is compact and intentionally simple.
- The real code corpus excludes current P71 analysis files to avoid
  self-referential moving measurements; it includes repo source, tests,
  examples, validation docs, and historical analysis docs.
- The declared P69 contract cost is too small for the measured real-data store.
- No timing claim is made.
- High ratio claims are not allowed for the incompressible guard.

## 21. Recommendation for P72

P72 should recalibrate the contract cost model for filesystem stores before
claiming validation: separate contract bytes from dataset-specific residuals,
add a declared store budget model, and compare raw, dictionary, grammar-token,
and generated-plus-residual policies per corpus.

## 22. Reproducibility notes

Generated artifacts live under `artifacts/p71/` and are ignored by Git. The
committable evidence is this Markdown report, the validation documentation, the
tests, and the Results LaTeX/PDF.

## 23. Journal

- P71 prompt: add filesystem Fiber Store, real local corpora, 10 MiB budget,
  roundtrip, retrieval, guard incompressible, test stack audit, and Results
  LaTeX/PDF.
- Local P71 standard campaign: `real 0.31s`, budget pass, roundtrip pass,
  retrieval pass, guard refused, `HARD_DRIFT`, decision
  `RECALIBRATE_P71_CONTRACT_COST_MODEL`.
- Local validation complete: cargo fmt/build/test, P71/P70 tests,
  validate_p58_local, invalides P71, and Results PDF generation passed.

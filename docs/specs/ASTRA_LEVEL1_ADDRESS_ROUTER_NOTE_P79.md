# ASTRA Level-1 Address Router Note P79

P79 extends the P78 level-1 address-space note. It does not replace
`ASTRA_CORE_SPEC_P76.md` or `ASTRA_LEVEL1_ADDRESS_SPACE_NOTE_P78.md`.

## Principle

The level-1 address space is not chosen globally. A Level-1 Address Router may
choose a topology by local feature:

- path trie for file paths, modules, JSON paths and URL-like keys;
- content-addressed DAG for chunks, binary-like files and deduplication;
- product typed space for namespace/type/object/chunk/version addresses;
- graph address space for symbolic relations and cross references;
- hybrid multi-index when multiple access paths justify the index cost;
- grid baselines only for regular geometric projections.

## Guard

Guard data is never routed to a success topology. It is refused or represented
as raw no-gain fallback, with no false gain.

## Virtual Metrics

Fields ending in `_bytes_equivalent` remain materialization equivalents. They
are not stored bytes.

## Cost Accounting

Paid cost includes the level-1 router policy, level-1 indexes, topology metadata,
journals, checksums, audit, actor state, residuals, cold persisted bytes,
runtime peak bytes and reopen/replay bytes.

## P80 Path

P80 should calibrate the level-1 router against the oracle and cross the
`0.97 * hybrid` ratio gate while preserving lookup and index savings.


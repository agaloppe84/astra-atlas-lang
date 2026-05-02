# ASTRA Core Spec P76

## Status

This is a P76 working snapshot for P77/P78. It freezes the core rules observed
through P69-P76 without claiming a final scientific specification.

## Living Memory Principle

ASTRA decisions about virtual/real memory ratio must come from living-memory
runs: encode, open, read/query, update, delete, audit, compact, close and
reopen. Unit tests remain compatibility and invariant checks.

## Procedural Virtual Space

The virtual space is not globally materialized. It is constructed locally when
an address is reached. Reports must distinguish:

- `virtual_declared`;
- `virtual_reachable`;
- `virtual_readable`;
- `virtual_updatable`;
- `virtual_safe`;
- `virtual_effective`;
- `virtual_declared_bytes_equivalent`;
- `virtual_effective_bytes_equivalent`.

The two byte fields are materialization equivalents, not stored bytes.

## Address-Fiber Model

An address selects a local fiber. Runtime evaluation regenerates the requested
fiber or a bounded local neighborhood. Actor/router/topology state is counted as
real overhead.

## Living Store Cycle

The living store cycle is:

`encode -> open -> read/query -> update -> delete -> audit -> compact -> close -> reopen`.

Reopen equivalence is logical equivalence of reads, query answers, tombstones,
checksums, journal sequence and contract hash. Runtime cache bytes need not be
byte-identical after reopen.

## Topology Model

P76 keeps six topology families available: linear, cubical, trie, graph,
hypergraph and hierarchical. No topology creates more information per bit. A
topology may improve factorization, locality and audit scope when the corpus has
matching structure.

## Mixed Topology Router

The mixed router chooses a topology per local fiber feature. It is a candidate,
not a universal law. P76 measures its regret against an oracle.

## Routing Oracle

The P76 oracle compares the router-selected topology with the best observed
topology per feature/corpus/locality/update slice. The oracle reports
wrong-route count, regret in `ratio_living`, update cost, audit cost and routing
accuracy.

## Cost Accounting Rules

The real paid cost includes cold persisted bytes, runtime peak bytes, journal
replay bytes, router/oracle bytes, topology metadata, indexes, residuals,
journals, checksums, audit, actor state and guard fallback. Hidden overhead is a
contract failure.

## Guard Rules

Incompressible guard data must be refused, raw/no-go, or explicitly excluded
from success. It must not create false ratio gain.

## P76 Decision Gates

Promotion requires mixed/oracle `ratio_living >= 0.95`, controlled wrong-route
cost, successful retrieval, reopen equivalence, guard refusal, drift not hard,
virtual metrics present and P61-P75 non-regression.

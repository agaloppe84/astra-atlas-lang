# ASTRA Level-1 Address Space Note P78

P78 extends the P76 spec snapshot with a level-1 address-space note. This file
does not replace `ASTRA_CORE_SPEC_P76.md`; it records the P78 working extension.

## Principle

The level-1 address space selects where a local fiber can be reached. It is not
required to be a 2D grid. It may be a tree, path trie, content-addressed DAG,
graph, typed product space, or hybrid multi-index.

The virtual space is local-on-address. ASTRA must not materialize the full
virtual address space globally.

## Virtual Bytes

Fields named `virtual_declared_bytes_equivalent` and
`virtual_effective_bytes_equivalent` are materialization equivalents. They are
not stored bytes.

## Universal File Support

The universal store must accept any file by either a specialized codec or an
explicit raw fallback. Raw fallback is counted as real storage and must not
create a false gain. Incompressible guard data is refused or no-go.

## Cost Accounting

The paid cost includes level-1 topology metadata, address indexes, fiber router
state, codec metadata, residuals, journals, checksums, audit data, actor state,
runtime peak bytes, and reopen/replay bytes.

## P79 Path

P79 should evaluate a level-1 address-space router that chooses path trie,
content DAG, typed product space, graph overlay, or hybrid multi-index by file
class and local address feature.

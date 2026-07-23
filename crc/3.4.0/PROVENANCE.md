# crc 3.4.0 provenance

**Verification status: strong functional core (partial).**

This source tree is copied from the crate published on crates.io as `crc`
version `3.4.0`. The published archive has SHA-256 checksum
`5eb8a2a1cd12ab0d987a5d5e825195d372001a4094a0376319d5a0ad71c1ba0d`.

The archive records immutable upstream revision
`2c8fd9615d620b5a5f8c9556b79a4ca173d6d401`:

https://github.com/mrhooray/crc-rs/tree/2c8fd9615d620b5a5f8c9556b79a4ca173d6d401

The public runtime API and behavior, upstream tests, and MIT and Apache-2.0
license files are preserved. `creusot-std`, mathematical models, contracts,
invariants, proof annotations, and verification support files are the
repository-owned additions.

## Specification

The public contracts cover all five supported register types (`u8`, `u16`,
`u32`, `u64`, and `u128`) and all three sealed implementation choices
(`NoTable`, `Table<1>`, and `Table<16>`):

- `Crc::new` requires a nonzero width no greater than the machine register and
  establishes the complete table-representation invariant.
- `Crc::checksum` is specified by a recursive, table-independent CRC model
  covering initial alignment, polynomial reflection, one-bit feedback rounds,
  the complete byte fold, output reflection, width alignment, and `xorout`.
- `Crc::digest` and `digest_with_initial` specify the exact initial internal
  register, including the custom-initial override.
- `Digest::update` specifies an exact fold from the old register over the whole
  input slice; `Digest::finalize` specifies the exact final transformation.
- `Crc::table` exposes a logical snapshot of all generated lanes.
- The explicit `Clone` implementations preserve the algorithm,
  implementation data, borrowed CRC, and in-progress register as applicable.

`Table`, every supported `Crc<W, Table<L>>`, and every supported
`Digest<'a, W, Table<L>>` have explicit invariants. `Crc` relates every table
entry to the byte-wise recurrence, including the recurrence between slicing
lanes. `Digest` requires the borrowed `Crc` invariant and exposes its current
unfinalized register through `View`.

`Algorithm`, `Width`, and the named algorithm constants are reexports owned by
the separately published `crc-catalog 2.4.0` dependency; they are not nominal
types defined by this crate. Their fields are nevertheless fully consumed by
the `crc` contracts.

## Proof status

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| Public constructors and orchestration (`checksum`, digest creation/update/finalize, `table`) | yes | yes | no | yes |
| Public `Clone` adapters | yes | no | yes | yes |
| Mathematical recurrence and representation predicates | yes | yes | no | yes |
| Lookup-table generation | yes | no | yes | yes |
| Eight-round bitvector leaf functions | yes | no | yes | yes |
| Initial alignment and final reflection runtime leaves | yes | no | yes | yes |
| Byte-wise and slicing-by-16 optimized update | yes | no | yes | yes |

The trusted boundaries have strong functional contracts and local TODOs. They
are retained because `reverse_bits` currently has no Creusot standard-library
contract and because proving table generation and the slicing-by-16 blocks is
the next separate proof phase. Removing the optimized-update boundary requires
first proving one byte update for `NoTable` and `Table<1>`, then one complete
16-byte block for `Table<16>`, for each register width.

The crate-scoped integrated proof currently contains 89 proof files. Run
`./verify-all.bash` in this directory to clean generated proof artifacts and
reproduce it. Run `cargo test` here to execute the six upstream unit tests, five
integration tests, and two documentation tests.

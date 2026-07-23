# byteorder 1.5.0 provenance and verification scope

**Verification status: strong functional core (partial).**

This source tree is copied from the crate published on crates.io as
`byteorder` version `1.5.0`. The published archive has SHA-256 checksum
`1fd0f2584146f6f2ef48085050886acf353beff7305ebd1ae69500e27c67f64b`.
The archive's `.cargo_vcs_info.json` identifies the immutable upstream source
revision.

The public runtime API, feature gates, licenses, and observable byte-order
behavior are preserved. Four unsafe byte-copy implementations were rewritten
with the corresponding safe `to_be_bytes`/`to_le_bytes` slice copies, the bulk
conversion loops use explicit indices instead of iterator zips, and float
slice conversion uses `to_bits`/`from_bits`; these are behavior-preserving
rewrites made so Creusot can translate the crate.

## Established contracts

The `ByteOrder` contract surface uses recursive mathematical models for
big-endian and little-endian unsigned decoding. It specifies:

- exact unsigned and two's-complement signed results for every scalar integer
  read, including 24-bit, 48-bit, and variable-width operations;
- exact encoded bytes and untouched output suffixes for scalar integer writes;
- exact element-by-element results for integer slice reads and writes;
- input-size and representable-range preconditions that match every panic
  condition in the integer API;
- buffer-size and footprint conditions for floating-point operations.

All 53 public `ReadBytesExt` and `WriteBytesExt` methods have explicit
contracts. `BigEndian` and `LittleEndian`, the only two public nominal types,
have explicit deep models and invariants. Both invariants are `true` because
the marker enums are uninhabited; there is no runtime representation that can
violate an additional condition.

## Explicit trusted boundaries

`ByteOrder` implementations are trusted against the reviewed contracts because
Creusot's current standard-library model has no contracts for primitive
`from_be_bytes`, `from_le_bytes`, `to_be_bytes`, `to_le_bytes`, or float bit
conversion. The six private sign-extension and packed-width helpers are part of
the same primitive bit-operation boundary. Integer contracts nevertheless
expose exact functional behavior to verified callers.

The `ReadBytesExt` and `WriteBytesExt` traits are trusted integration boundaries
because Creusot does not model the stateful `std::io::Read`/`Write` protocols.
Their contracts currently guarantee total result typing only; they do not model
reader/writer state transitions. Their private slice-to-byte reinterpretation
helper is included in that boundary. Float scalar and bulk contracts specify
safety and write footprint, but not an IEEE-754 bit-level relation for the same
missing standard-library-contract reason.

The `Default` implementations for the two uninhabited marker enums are trusted
with `ensures(false)`, exactly recording that both implementations always panic
and never return a value.

Run `./verify-all.bash` in this directory to check both the no-default-features
and all-features configurations. Generated Cargo and Why3 artifacts are not
tracked.

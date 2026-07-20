# crate-proofs

Experiments in verifying published Rust crates with
[Creusot](https://github.com/creusot-rs/creusot).

Each directory under `crates/<name>/<version>` is a complete copy of the
published crate with specifications and proof annotations added in place.
Public APIs and runtime behavior are preserved.

Run proofs with:

```sh
./verify.bash crates/adler2/2.0.0
./verify.bash crates/fnv/1.0.7
./crates/byteorder/1.5.0/verify-all.bash
./crates/hex/0.4.3/verify-all.bash
./crates/percent-encoding/2.3.2/verify-all.bash
```

`creusot-libs` contains the Creusot libraries pinned at commit
`7a48f5a5b1cb15a11c4e744568ca187331a30025` and repository-owned
standard-library specifications used by the proofs.

## Current proofs

### byteorder 1.5.0

`byteorder` 1.5.0 has recursive mathematical models for big-endian and
little-endian decoding. Contracts cover every public `ByteOrder`,
`ReadBytesExt`, and `WriteBytesExt` method; integer scalar and slice contracts
specify exact byte-level behavior, signed two's-complement interpretation,
panic preconditions, and write footprints. Both public marker types have an
explicit deep model and invariant.

Primitive byte-conversion functions, floating-point bit conversion, and the
stateful `std::io` protocol remain explicit trusted boundaries because the
current Creusot standard library does not specify them. The proof matrix covers
no-default-features and all features.

### adler2 2.0.0

`adler2` 2.0.0 is checked for arithmetic and indexing safety, including the
post-update range of the private checksum state. This proof does not yet relate
the optimized checksum loop to a mathematical Adler-32 function.

The `std::io::BufRead` adapter is marked `#[trusted]` because Creusot does not
currently specify the stateful `fill_buf`/`consume` protocol. The checksum core
called by that adapter is verified.

The repository-owned `ChunksExact` external specification is also a trusted
library boundary because libcore keeps the iterator state private. Its contract
models chunk counts, remainder length, and yielded chunk size; it does not claim
element-by-element correspondence with the source slice.

### fnv 1.0.7

`fnv` 1.0.7 is checked for arithmetic and indexing safety. Its 64-bit FNV-1a
step is modeled with bitvector XOR and wrapping multiplication. The public
`FnvHasher` type has an opaque `u64` view and an explicit invariant stating that
every 64-bit state is valid. Contracts specify `Default`, `with_key`, `finish`,
and `write`; in particular, `write` is proved to update the old view to the
recursive FNV-1a fold over the complete input slice.

The proof keeps `FnvHasher`'s tuple field private, preserving the upstream API.
Its logical view is opaque outside the crate, so clients can use the contracts
without depending on that private representation. The upstream FNV test vectors
also exercise the public `write` and `finish` behavior.

### hex 0.4.3

`hex` 0.4.3 has exact lowercase/uppercase encoding and mixed-case decoding
models. Contracts cover slice, alloc, trait, fixed-array, and serde-facing
public APIs across the no-feature, `alloc`, `serde`, and all-feature
configurations. Slice decoding also specifies error precedence, the first
invalid character and index, and the precise partially written prefix and
untouched suffix on failure.

The only trusted portions are explicit generic-adapter boundaries for
`AsRef`/`FromIterator`, serde's unmodeled visitor protocol, and
`core::fmt::Formatter`; the codec loops themselves are proved. The version
audit found no algorithm change from 0.4.2, but did find a feature-level source
incompatibility: alloc-backed APIs that were available with
`default-features = false` in 0.4.2 require the new `alloc` feature in 0.4.3.

### percent-encoding 2.3.2

`percent-encoding` 2.3.2 has exact models for ASCII encode sets and for the
remaining output of its encoder and decoder iterators. Contracts cover set
operations, uppercase `%HH` encoding, WHATWG-style decoding, iterator yields
and size hints, structural equality, `Display`/`Debug`, `Cow` conversions, and
strict and lossy UTF-8 conversion. All three public nominal types have an
explicit view, deep model, and invariant.

The proof matrix covers `no-default-features`, `alloc`, and all features. The
codec models, decoder loop, set operations, constructors, and size hints are
proved. Explicit trusted boundaries remain for the runtime `%HH` lookup table,
the encoder's maximal unchanged-chunk search, iterator composition laws,
formatter behavior, allocation/ownership adapters, and standard-library UTF-8
observers. These boundaries retain meaningful output contracts and are listed
in the crate's `PROVENANCE.md`; exhaustive byte and bridge tests connect the
runtime implementations to those contracts.

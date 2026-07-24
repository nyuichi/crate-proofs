# rust-crate-proofs

Experiments in verifying published Rust crates with
[Creusot](https://github.com/creusot-rs/creusot).

Each `<name>/<version>` directory is a complete copy of the
published crate with specifications and proof annotations added in place.
Public APIs and runtime behavior are preserved.

## Running proofs

Each crate's exact proved surface, remaining boundaries, feature matrix, and
reproduction command are recorded in its `PROVENANCE.md`. Run the proofs with:

```sh
./adler2/2.0.0/verify-all.bash
./fnv/1.0.7/verify-all.bash
./crc/3.4.0/verify-all.bash
./arrayvec/0.7.8/verify-all.bash
./byteorder/1.5.0/verify-all.bash
./hex/0.4.3/verify-all.bash
./percent-encoding/2.3.2/verify-all.bash
./fugit/0.4.0/verify-all.bash
./cobs/0.5.1/verify-all.bash
./rustc-hash/2.1.3/verify-all.bash
./slab/0.4.12/verify-all.bash
./smallvec/1.15.2/verify-all.bash
./bytes/1.11.1/verify-all.bash
./semver/1.0.28/verify-all.bash
./fixedbitset/0.5.7/verify-all.bash
./uuid/1.24.0/verify-all.bash
./bstr/1.13.0/verify-all.bash
./base64/0.22.1/verify-all.bash
./ipnet/2.12.0/verify-all.bash
./heapless/0.9.2/verify-all.bash
./bitflags/2.13.1/verify-all.bash
./itoa/1.0.18/verify-all.bash
./crossbeam-queue/0.3.13/verify-all.bash
./indexmap/2.14.0/verify-all.bash
./utf8parse/0.2.2/verify-all.bash
./unicode-ident/1.0.24/verify-all.bash
```

`creusot-libs` contains the Creusot libraries pinned at commit
`7a48f5a5b1cb15a11c4e744568ca187331a30025` and repository-owned
standard-library specifications used by the proofs.

## Current proofs

### bitflags 2.13.1

`bitflags` 2.13.1 has an exact `u8` bit-vector model for a representative
three-flag macro expansion. Construction, checked/retaining/truncating
conversion, known and unknown bit observations, containment and intersection,
all four set-algebra operations, known-bit complement, mutations, and value and
assignment operator adapters are proved. The matrix covers no-default-features,
default features, and all features.

This is a representative generated-core proof, not a proof of generic macro
expansion for every integer width. Parsing, formatting, iteration, named lookup,
and optional integrations remain outside translation. Full scope and the
generic-model removal condition are recorded in `PROVENANCE.md`.

### itoa 1.0.18

`itoa` 1.0.18 has an exact signed decimal ASCII model for every supported
integer primitive. The proof establishes magnitude and sign conversion,
recursive digit writing, the 40-byte capacity bound, exact start indices, and
the complete result of public `Buffer::format`, including `i128::MIN`. Default
and all-feature integrated runs each prove 67 files.

Ordinary builds retain the published lookup-table and `MaybeUninit` runtime
implementation. The verification build uses an equivalent initialized-buffer
writer so that raw-memory details do not obscure the decimal algorithm. The
only trusted leaf converts an already-proved ASCII suffix to `str`; its exact
byte contract and removal condition are recorded in `PROVENANCE.md`.

### crossbeam-queue 0.3.13

`crossbeam-queue` 0.3.13 has exact element-sequence models for `ArrayQueue` and
`SegQueue`. Empty construction, length and emptiness observations, bounded
capacity/fullness, and the FIFO effects of the exclusive `push_mut` and
`pop_mut` APIs are proved. Full bounded pushes preserve the queue and return
the rejected value; successful pops return and remove the logical head.

This is a single-owner state-machine proof, not a proof of the lock-free
implementation. Atomics, compare-exchange loops, memory ordering,
linearizability, progress, raw slot storage, allocation/reclamation, concurrent
APIs, drops, and iterators remain explicitly excluded. Ordinary builds retain
the complete upstream implementation. The proof matrix covers `no_std +
alloc`, default `std`, and all features; the ordinary all-feature suite passes
23 integration tests and 19 documentation tests. Full boundaries are recorded
in the crate's `PROVENANCE.md`.

### indexmap 2.14.0

`indexmap` 2.14.0 has exact ordered-sequence models for `IndexMap` and
`IndexSet`. Caller-hasher construction, length and emptiness observation,
clearing, exact prefix truncation, ordered popping, shift removal by index,
position exchange, and content-preserving capacity operations are body-proved.
The proof matrix covers `no_std`, default `std`, and all features.

This is a positional order proof, not a hash-table or key-uniqueness proof.
Hash/equality coherence, key lookup and insertion, entry APIs, iterators,
sorting, draining, raw and disjoint access, and optional adapters remain outside
proof translation. Random-state construction is the sole narrow trusted body;
its contract asserts only that the result is empty. Full boundaries and the
removal condition are recorded in `PROVENANCE.md`.

### utf8parse 0.2.2

`utf8parse` 0.2.2 has an exact model of all byte-class transitions and parser
accumulator updates. The proof establishes the complete transition table,
state-specific reachable ranges, preservation of the parser invariant for every
input byte, and the safety of the unchecked Unicode scalar construction. The
integrated no-feature run proves 14 files.

Receiver callbacks remain abstract because the public `Receiver` trait imposes
no semantic model on implementors. Two verification-only equality adapters are
trusted and tied to exact deep models; the transition, accumulator, scalar
safety, and public `advance` bodies are all proved. Full scope and removal
conditions are recorded in the crate's `PROVENANCE.md`.

### unicode-ident 1.0.24

`unicode-ident` 1.0.24 has exact classification models for both XID_Start and
XID_Continue. The ASCII branches, non-ASCII trie/half-chunk offset arithmetic,
leaf bounds, all eight leaf-bit positions, and both public API bodies are
proved. An exhaustive runtime comparison checks every valid Unicode scalar
value against independently generated Unicode 17.0.0 range tables.

The pinned Creusot cannot translate immutable static arrays or `char`-to-
integer casts, so generated table-byte access and scalar-value conversion are
narrow explicit trusted boundaries with exact contracts. The published
compressed tables and optimized runtime implementation are preserved. Full
scope and removal conditions are recorded in `PROVENANCE.md`.

### heapless 0.9.2

`heapless` 0.9.2 has a verified length/cursor state machine for `Deque`.
Capacity and wraparound arithmetic, empty/full observations, and checked and
unchecked front/back push and pop bodies are proved, including exact length
effects. The integrated default-feature run proves 57 files, and the ordinary
Deque suite passes 34 tests.

This is a Deque structural proof, not a whole-heapless or element-sequence
proof. Generic `MaybeUninit<T>` slot moves, construction, clearing, slice and
reference exposure, element-aware iterators, and the crate's other collections
remain trusted or excluded. Full boundaries are recorded in `PROVENANCE.md`.

### semver 1.0.28

`semver` 1.0.28 has an exact model of Cargo's version-requirement evaluation.
Contracts and proved bodies cover all eight comparator operators, partial
major/minor/patch versions, requirement conjunction, and the special rule that
admits prerelease versions only when a comparator names the same numeric
version with a nonempty prerelease. Both public `matches` methods are proved
against these models.

The proof matrix covers `no_std` and all features. There are two trusted
identifier observations: prerelease emptiness and prerelease precedence. They
isolate the upstream pointer-tagged short-string representation; parsing,
formatting, serde, and identifier storage/ordering bodies remain outside proof
translation. The ordinary all-feature upstream suite passes 34 tests. Full
boundary and removal-condition details are recorded in the crate's
`PROVENANCE.md`.

### fixedbitset 0.5.7

`fixedbitset` 0.5.7 has an exact finite Boolean-sequence model for its core
fixed-length state machine. Construction, length/full/clear observation, exact
minimum and maximum, out-of-range membership, clearing, insertion, removal,
put, toggle, set, bit copying, grow-and-insert orchestration, set relations,
all four in-place set algebra operations, and their exact cardinalities are
proved with element-wise contracts. All range counting, mutation, and
all/any-predicate APIs are also proved for `..`, `a..`, `..b`, and `a..b`.
The proof matrix covers
no-default-features, default `std`, and all features.

The upstream aligned SIMD allocation, raw-pointer ownership, lazy set-algebra
iterators, raw block counting, formatting, adapters, and unsafe APIs remain
outside proof translation. Ordinary builds retain the complete upstream
implementation, whose all-feature suite passes 63 unit tests and 7
documentation tests. Full boundary and removal conditions are in
`PROVENANCE.md`.

### uuid 1.24.0

`uuid` 1.24.0 has an exact 16-byte model for `Uuid` and `Builder`. The
no-default-features proof establishes byte-preserving construction and access,
the mixed-endian field permutation, nil/max values, variant and version-bit
extraction, and the exact byte footprints of the consuming builder
variant/version mutations.

This is not a parser, formatter, generator, or randomness proof. Parsing,
actual text encoding, `u128`/field/reference reinterpretation adapters,
timestamps, UUID generation, and the real `NonZero<u128>`-backed `NonNilUuid`
remain explicit trusted boundaries or Creusot-only exclusions. The generated
verification manifest keeps the library target named `uuid` but uses a distinct
package name to avoid Cargo selector ambiguity. Full scope and removal
conditions are recorded in the crate's `PROVENANCE.md`.

### bstr 1.13.0

`bstr` 1.13.0 has exact positional contracts for its scalar non-ASCII scan and
the forward and reverse single-byte inverse searches. The proof-facing loops
establish both the returned differing byte and equality of the complete skipped
prefix or suffix; unsuccessful searches establish equality of the whole input.
The proof matrix covers no-default-features, default, and all features.

The published optimized implementations use raw pointers, unaligned word loads,
and SSE2, so their bodies remain an explicit excluded boundary behind the same
contracts. UTF-8 decoding, Unicode segmentation, public string types and the
remaining search, split, allocation, I/O, and serde APIs are not yet verified.
Full boundary and removal-condition details are in `PROVENANCE.md`.

### base64 0.22.1

`base64` 0.22.1 has exact arithmetic models for padded and unpadded encoded
lengths and conservative decoded buffer lengths. The length bodies, including
`usize` overflow behavior, and the `GeneralPurposeConfig` builders are proved.
The matrix covers `no_std`, `alloc`, and all features.

This is not an RFC 4648 codec proof. Alphabet validation and table generation,
the optimized encode/decode bodies, suffix handling, padding writes, generic
and allocation adapters, formatting, and streaming I/O remain explicit trusted
boundaries or Creusot exclusions. The upstream all-feature suite passes 179
unit tests, 13 integration tests, and 25 documentation tests. Full boundary and
removal-condition details are in `PROVENANCE.md`.

### ipnet 2.12.0

`ipnet` 2.12.0 has a prefix-length model for `Ipv4Net`, `Ipv6Net`, and
`IpNet`, with invariants enforcing the IPv4 and IPv6 limits. Checked and
asserting constructors, prefix observers, maximum-prefix observers, and
family-level orchestration are proved in `no_std`, default `std`, and
all-feature configurations.

This is a structural prefix-state proof, not an address-content proof. The
pinned Creusot library has no logical model or method contracts for
`core::net` address types, so mask arithmetic, containment, network/broadcast
calculation, ranges, subnet iteration, aggregation, parsing, formatting, and
optional adapters remain explicitly excluded from translation. Ordinary builds
retain the upstream implementation and API. Full scope and the removal
condition are recorded in the crate's `PROVENANCE.md`.

### bytes 1.11.1

`bytes` 1.11.1 has a structural length/capacity model for `Bytes` and
`BytesMut`. Empty construction, length/capacity observation, splitting,
truncation, clearing, resizing, freezing, and the core cursor laws are proved
in a Creusot-facing state machine. The matrix covers no-default-features,
default `std`, and all features.

This is not a byte-content or raw-memory proof. The published implementation's
type-erased vtables, raw pointers, atomics, reference counting, uninitialized
storage, complete `Buf`/`BufMut` API, and serde adapters remain outside proof
translation because the pinned Creusot reaches an internal compiler error in a
generic `Vec<T>` adapter. Runtime builds keep the upstream implementation and
API. Two small `std` conversion helpers are additionally trusted because
`usize::try_from(u64)` lacks a pinned-library contract. Full boundary and
removal-condition details are in `PROVENANCE.md`.

### slab 0.4.12

`slab` 0.4.12 has an occupied-count view and a closed representation invariant
that bounds both the count and vacant-list head by the backing vector length.
Empty construction, length and emptiness observation, and clearing are proved.
Contracts specify the occupied-count effects of insertion, removal, retention,
compaction, and draining.

This is a structural verification, not a key-to-value functional proof. Entry
contents, the vacant linked list, raw and disjoint mutable access, iterator
contents, formatting, `FromIterator`, and serde remain explicit trusted
boundaries or `cfg(creusot)` exclusions. The proof matrix covers `no_std`,
default `std`, and all features. Full boundary details are recorded in the
crate's `PROVENANCE.md`.

### smallvec 1.15.2

`smallvec` 1.15.2 has a representation-aware logical length model for the
default enum representation and an explicit nonnegative-length invariant.
Contracts cover empty and slice/vector construction, length and capacity
observers, push/pop, truncation, clearing, insertion/removal, slice extension,
resizing, raw-parts construction, and allocation operations. The `len`,
`is_empty`, `clear`, and `Default` bodies are proved against those contracts.

Raw `MaybeUninit` storage, allocator and pointer operations, element contents,
mutation bodies, drops, iterators, and generic standard-library adapters remain
explicit trusted boundaries or `cfg(creusot)` exclusions. The proof matrix
covers no features, `const_generics`, and `const_new`; it does not claim the
alternate `union` representation or the remaining optional adapters. The
default upstream suite passes 62 unit tests and 13 documentation tests. Full
boundary and removal-condition details are recorded in `PROVENANCE.md`.

### rustc-hash 2.1.3

`rustc-hash` 2.1.3 has exact machine-word transition models for integer input
and a byte-compression model covering little-endian reads, 16-byte bulk blocks,
the overlapping suffix, length mixing, and platform-selected multiplication.
Contracts cover `FxHasher`, `FxBuildHasher`, `FxSeededState`, and the optional
`FxRandomState`; all four public nominal types have explicit invariants.

Integer updates, the multiply-mix primitive on the x86-64 proof target, the
complete byte-compression algorithm, final rotation, seeded builders, and the
public byte-write orchestration are proved. Runtime builds retain the upstream
optimized slice and rotate spellings; the verification build uses proved
equivalent indexed little-endian reads and shifts. The only trusted body is the
optional thread-local random seed constructor, whose RNG protocol is not
modeled and which is outside the deterministic hash algorithm. The proof matrix
covers `no_std`, default `std`, `rand`, and all features including `nightly`.

### crc 3.4.0

`crc` 3.4.0 has table-independent mathematical models for all supported
register widths (`u8` through `u128`). Contracts cover construction, one-shot
checksums, incremental digest creation/update/finalization, table access, and
cloning for `NoTable`, `Table<1>`, and `Table<16>`. The three public nominal
types have explicit invariants; in particular, `Crc` relates the algorithm
width and every generated lookup lane to the byte-wise recurrence.

Public orchestration bodies are proved and the crate-scoped integrated proof
passes. Explicit trusted leaf boundaries remain for lookup-table generation,
eight-round bitvector helpers, runtime bit reversal during initialization and
finalization, the optimized byte/slicing-by-16 update bodies, and generic Clone
adapters. Each boundary has a functional contract and removal condition in
`PROVENANCE.md`.

### arrayvec 0.7.8

`arrayvec` 0.7.8 has length views for `ArrayVec` and `ArrayString`, with an
invariant that the stored length does not exceed the fixed capacity. Exact
contracts cover empty construction and the public length/capacity observers.
The proof matrix passes for default (`std`), `no-default-features` (`no_std`),
and all-feature configurations.

This is a structural verification, not a full element-sequence proof. Raw
`MaybeUninit` storage operations, collection mutation, UTF-8 construction,
iterator contents, drops, formatting, and several generic standard-library
adapters remain explicit trusted boundaries or `cfg(creusot)` exclusions. The
crate's `PROVENANCE.md` records those limits; the serde, borsh, and zeroize
implementations compile in the all-feature configuration but are excluded from
the proof translation.

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

`adler2` 2.0.0 has an exact Adler-32 model built from byte and weighted-byte
sums. Its runtime checksum loop uses the output-equivalent per-byte Adler
recurrence and is proved functionally equivalent to that model for arbitrary
input lengths. Contracts cover every crate-owned checksum API and `Hasher`
integration, including repeated writes and every noncanonical state accepted by
`from_checksum`. This verification-oriented runtime replaces the upstream
four-lane implementation and may have lower throughput.

The disabled optimized source and locally proved four-lane `process_chunk`
component are retained as reference scaffolding, but are not claimed as an
integrated optimized-runtime proof. The sole crate-local trusted body is the
optional `std::io::BufRead` adapter because Creusot does not model the stateful
`fill_buf`/`consume` protocol; the slice checksum core it calls is proved. The
proof matrix covers `no_std` and all features.

### fnv 1.0.7

`fnv` 1.0.7 is a complete functional proof of the crate-owned FNV-1a
implementation. Its 64-bit step is modeled with bitvector XOR and wrapping
multiplication. The public `FnvHasher` type has an opaque `u64` view and an
explicit invariant stating that every 64-bit state is valid. The proved bodies
cover every crate-owned executable API: `Default`, `with_key`, `finish`, and
`write`; in particular, `write` updates the old view to the recursive FNV-1a
fold over the complete input slice.

The proof contains no `#[trusted]` declaration. The remaining public map/set and
builder names are standard-library type aliases, not additional crate-owned
algorithm bodies. The proof matrix covers `no_std` and default `std`; the
upstream FNV test vectors exercise `write` and `finish` in both configurations.

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

### fugit 0.4.0

`fugit` 0.4.0 models `Duration`, `Instant`, and `Rate` by their exact stored
tick/raw values. Contracts cover every public API and specify floor, ceiling,
and nearest base conversions, checked and saturating arithmetic, reciprocal
period/rate conversion, `u32`/`u64` conversion, and wrap-aware instant
arithmetic. Every public nominal type has an explicit invariant; public aliases
inherit the invariant of their underlying type.

The feature matrix covers no features and all features. Cross-base/GCD
arithmetic, floating-point and `core::time` adapters, formatting, and several
operator adapters remain explicit trusted bodies with reviewed contracts.
Serde/postcard derives and defmt formatting are preserved for normal builds but
excluded from Creusot translation. Standard comparison traits are likewise
excluded because the upstream wrap-aware/overflow-aware semantics do not obey
the total-order and equivalence laws assumed by Creusot; the contracted
`const_*` comparison APIs are retained as the verification interface. Full
boundary details are recorded in the crate's `PROVENANCE.md`.

### cobs 0.5.1

`cobs` 0.5.1 has contracts on every explicit public function and method and an
explicit view and invariant for all eleven public nominal types. Contracts cover
encoding lengths and footprints, canonical-frame round trips, arbitrary
sentinels, streaming-state bounds, reports, resets, and allocation-backed APIs.
The primitive encoder and decoder state transitions and length arithmetic are
proved; buffer orchestration, allocation/heapless adapters, and the opaque exact
COBS sequence model remain documented trusted boundaries. The proof matrix
covers no-default-features, alloc, and all features, and the upstream all-feature
test suite is retained.

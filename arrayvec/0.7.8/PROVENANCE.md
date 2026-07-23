# arrayvec 0.7.8 provenance and verification scope

This source tree was copied from the `arrayvec` 0.7.8 package in the local
Cargo registry. Its `.cargo_vcs_info.json` records upstream revision
`0cb664cf505844348538230479b0040b44f3faf1`. Proof annotations, the
`creusot-std` dependency, `why3find.json`, and the verification script are
repository-local additions. Normal Rust builds retain the upstream behavior;
verification-only branches and exclusions are selected with `cfg(creusot)`.

## Established contracts

The current logical view of both `ArrayVec<T, CAP>` and `ArrayString<CAP>` is
their length as a mathematical integer. Their invariant states that this
length does not exceed `CAP`. Exact contracts cover empty construction,
`len`, `is_empty`, `capacity`, `is_full`, and `remaining_capacity`.
`set_len` has an explicit capacity precondition and exact resulting-length
postcondition, but remains trusted because it changes the initialized prefix
through an unsafe representation operation.

The proof succeeds for both supported core configurations:

- default features (`std`): `Proved (74 files)`;
- `--no-default-features` (`no_std`): `Proved (74 files)`;
- `--all-features`: `Proved (74 files)`, with the optional adapters excluded
  from translation as described below.

These results establish the translated proof obligations under the boundaries
below. They do not establish element-by-element contents, UTF-8 validity, drop
behavior, or a full functional model of the collection operations.

## Explicit trusted boundaries and exclusions

The initialized storage is a `MaybeUninit` array manipulated with raw
pointers. Creusot's current standard-library contracts do not expose enough of
that representation to prove the unsafe implementation compositionally, so
the following areas are explicit trusted boundaries:

- raw slice/string exposure, pointer arithmetic, initialized-prefix length
  changes, element writes, moves, clones, shifts, removals, truncation, and
  drops;
- `ArrayString` UTF-8 encoding and mutation operations;
- iterator `next`/`next_back`, drain state restoration, and panic-safe extend
  guards. The iterator protocol specifications are intentionally weak and do
  not claim yielded element order or correspondence;
- formatting, hashing, I/O, parsing, path conversion, and generic adapter
  implementations whose required standard-library protocols are unavailable;
- comparison, borrowing, conversion, clone/from-iterator, and related trait
  implementations excluded only from the `cfg(creusot)` build. They remain
  present and unchanged in ordinary runtime builds.

The `serde`, `borsh`, and `zeroize` implementations are included in ordinary
all-feature builds but explicitly excluded from the Creusot translation; the
all-feature proof therefore checks feature compatibility without proving those
adapter bodies. The upstream all-feature test suite passes all 64 tests,
including six borsh and six serde adapter tests. Run `./verify-all.bash` in
this directory to reproduce the three checked configurations. Generated Cargo
and Why3 artifacts are not tracked.

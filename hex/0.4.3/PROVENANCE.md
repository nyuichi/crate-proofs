# hex 0.4.3 provenance and verification scope

This source tree is copied from the crate published on crates.io as `hex`
version `0.4.3`. The published archive has SHA-256 checksum
`7f24254aa9a54b5c858eaee2f5bccdb46aaf0e486a595ed5fd8f86ba55232a70`.
Its `.cargo_vcs_info.json` records the immutable upstream revision
`b2b4370b5bf021b98ee7adc92233e8de3f2de792`.

The public runtime API, feature gates, licenses, and observable codec behavior
are preserved. The two core loops were rewritten into verification-friendly
but equivalent forms: decoding combines two proven nibbles as `high * 16 +
low`, and encoding checks exact output length with division and remainder
instead of a potentially overflowing `input.len() * 2` expression.

## Established contracts

The proof gives exact lowercase and uppercase models and proves the following:

- `encode_to_slice` succeeds exactly when the output length is twice the input
  length, writes the complete lowercase encoding on success, and leaves the
  output untouched on a length error;
- `decode_to_slice` distinguishes odd input, output-length mismatch, and the
  first invalid character with its exact character and index; on an invalid
  character, every complete preceding pair is decoded and the remaining
  output suffix is unchanged;
- alloc-backed `encode`, `encode_upper`, and `decode`, the `ToHex` and
  `FromHex` implementations, and every published fixed-array implementation
  use the same exact models;
- `FromHexError` has a discriminant-and-fields view, a structural invariant,
  and equality tied to its deep model. Because all variants and fields are
  public, every structurally valid variant is constructible; facts connecting
  an error to a particular input belong to decoder postconditions, not the type
  invariant.

The source documentation says `encode_to_slice` accepts an output buffer able
to hold "at least" twice the input length. The implementation in both 0.4.2
and 0.4.3 actually requires an exact length. The contract deliberately records
the implementation behavior, and regression tests cover an oversized,
unchanged output buffer.

## Explicit trusted boundaries

No codec loop or error branch is trusted. The remaining trusted declarations
are narrow integration boundaries:

- generic `AsRef<[u8]>` and generic `FromIterator<char>` adapters, because the
  current Creusot standard library does not expose the needed generic trait
  models; their contracts bind the adapters to exact byte/character sequences;
- serde serializer/deserializer protocol relations, because serde has no
  Creusot model. Serialization still exposes the exact string passed across
  the boundary, and successful deserialization is tied to the selected
  `FromHex` implementation's postcondition;
- `Display::fmt` and the verification-only spelling of the derived `Debug`
  implementation, because Creusot does not model mutation through
  `core::fmt::Formatter`. Ordinary builds retain the upstream `Debug` derive,
  and upstream tests check every exact display string.

## 0.4.2 to 0.4.3 compatibility audit

The official `hex` 0.4.2 archive has SHA-256 checksum
`644f9158b2f133fd50f5fb3242878846d9eb792e445c893805ff0e3824006e35`
and records revision `be0c32f9c8938ca0359bbb0d1477e31b07cb3358`.

There is no codec algorithm or error-behavior change between the official
0.4.2 and 0.4.3 sources; the nibble-mask spelling and formatting-only edits are
semantically identical. There is, however, a feature-level source
incompatibility: 0.4.2 exposed alloc-backed `Vec`/`String` APIs even with
`default-features = false`, while 0.4.3 introduced an explicit `alloc` feature
and gates those APIs behind it. Downstream crates using `default-features =
false` must add `features = ["alloc"]`. Likewise, `serde` without `alloc` now
exposes deserialization but not serialization. Default-feature behavior is
unchanged.

Run `./verify-all.bash` in this directory to reproduce the feature-matrix
proof. The latest integrated run passes all four no-feature, `alloc`, `serde`,
and all-feature configurations. The ordinary all-feature suite passes 14 unit,
4 serde integration, 2 version-audit, and 11 documentation tests. Generated
Cargo and Why3 build artifacts are intentionally not tracked.

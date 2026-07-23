# bstr 1.13.0 provenance and verification scope

This source tree is copied from the crate published on crates.io as `bstr`
version `1.13.0`. The published archive has SHA-256 checksum
`1f7dc094d718f2e1c1559ad110e27eeaae14a5465d3d56dd6dbd793079fbd530`.
The archive's `.cargo_vcs_info.json` identifies upstream revision
`63c6578f22bb8eda35a983e5c937d79c16b2a355`. Runtime builds retain the
published implementation and public API.

## Established contracts

The Creusot configuration proves exact positional contracts for three scalar
byte-search routines:

- `first_non_ascii_byte` returns the first byte greater than `0x7f`, or the
  slice length when all bytes are ASCII;
- `inv_memchr` returns the first byte unequal to its needle, or `None` when
  every byte is equal;
- `inv_memrchr` returns the last byte unequal to its needle, or `None` when
  every byte is equal.

Each successful-result contract identifies the differing byte and quantifies
over the complete preceding or following region. Each unsuccessful-result
contract quantifies over the complete input.

## Explicit verification boundary

The published implementations use raw pointers, unaligned word loads and, for
ASCII scanning on x86-64, SSE2 intrinsics. Creusot instead checks safe scalar
loops carrying the same functional contracts in `src/verification.rs`. The
runtime implementations have those contracts attached, but their optimized
bodies and the equivalence between the optimized and scalar implementations
remain unproved. Removing this boundary requires pointer/provenance contracts
for the word-at-a-time code and bit-vector contracts for the SSE2 intrinsics.

The rest of `bstr` is excluded from proof translation. In particular, this
checkpoint does not verify UTF-8 decoding, Unicode segmentation, searching and
splitting iterators, `BStr`/`BString`, allocation behavior, I/O or serde. The
upstream source is compiled and tested normally.

## Proof status

| Component | Contract reviewed | Body proved | Trusted/excluded | Integrated run |
|---|---:|---:|---:|---:|
| scalar non-ASCII search | yes | yes | no | yes |
| scalar forward inverse-byte search | yes | yes | no | yes |
| scalar reverse inverse-byte search | yes | yes | no | yes |
| optimized pointer/SSE2 implementations | contract only | no | excluded | no |
| remaining public API | no | no | excluded | no |

Run `./verify-all.bash` in this directory to check no-default-features,
default, and all-features configurations. Generated Cargo, Creusot and Why3
artifacts are not tracked.

The published source passes its 38 library tests with `std` enabled and
`unicode` disabled, including the inverse-search property tests and exhaustive
UTF-8 decoder test. An all-features library build passes. The published crate
archive excludes the Unicode consortium test-data files, so its all-features
unit-test target cannot be compiled directly from that archive; this is an
upstream packaging limitation rather than a failed test.

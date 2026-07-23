# bytes 1.11.1 provenance and verification scope

This source tree is copied from the crate published on crates.io as `bytes`
version `1.11.1`. The published archive has SHA-256 checksum
`1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33`.
The archive's `.cargo_vcs_info.json` identifies the immutable upstream source
revision. Runtime builds retain the published implementation and public API.

## Established contracts

The Creusot configuration models the visible length of `Bytes` and the
initialized length and capacity of `BytesMut`. Both public nominal types have
explicit invariants. The mutable invariant states that initialized length never
exceeds capacity. Proved contracts cover:

- empty and capacity-aware construction;
- `len`, `is_empty`, and `capacity`;
- the length transitions of `split_off`, `split_to`, `truncate`, and `clear`;
- `BytesMut::resize` and `freeze`;
- the remaining/advance cursor laws for the proof-facing subset of `Buf`;
- reviewed result contracts for the crate's `std`-only integer/`usize` min and
  saturating-subtraction helpers.

These are structural contracts. They do not model byte contents.

## Explicit verification boundary

The upstream `Bytes` and `BytesMut` implementations use type-erased vtables,
raw pointers, atomics, reference-counted ownership, `MaybeUninit`, and generic
standard-library collection adapters. With this repository's pinned Creusot,
translating the unmodified source reaches an internal compiler error while
instantiating a generic `Vec<T>` adapter. The runtime modules are therefore
selected under `cfg(not(creusot))`; `src/verification.rs` supplies the small,
proof-only state machine described above under `cfg(creusot)`.

This boundary is not a proof of allocation safety, reference-count correctness,
byte preservation, zero-copy aliasing, encoding/decoding, or the complete
`Buf`/`BufMut` method surface. Removal requires Creusot to translate the
upstream representation, followed by a representation relation connecting its
pointer range to a byte-sequence view. The state-machine contracts should then
be moved onto the real methods one component at a time.

The two `std` conversion helpers are trusted against their arithmetic contracts
because the pinned Creusot library has no contract for `usize::try_from(u64)`.
Their bodies are small direct matches on that conversion. This boundary can be
removed when the standard-library conversion contract is available.

## Proof status

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| proof-only length/capacity state machine | yes | yes | no | yes |
| `std` integer/`usize` conversion helpers | yes | no | yes | yes |
| upstream raw-pointer representation | no | no | excluded | no |
| byte-content model | no | no | absent | no |
| full `Buf`/`BufMut` API | no | no | absent | no |

Run `./verify-all.bash` in this directory to check no-default-features, default
`std`, and all-features configurations. Generated Cargo, Creusot, and Why3
artifacts are not tracked.

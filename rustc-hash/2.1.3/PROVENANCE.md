# rustc-hash 2.1.3 provenance

This source tree is copied from the crate published on crates.io as
`rustc-hash` version `2.1.3`. The published archive has SHA-256 checksum
`6b1e7f9a428571be2dc5bc0505c13fb6bf936822b894ec87abf8a08a4e51742d`.
The archive records the immutable upstream revision
`c13e7ccca705e6255387a2ebc6dca142d6881621`:

https://github.com/rust-lang/rustc-hash/tree/c13e7ccca705e6255387a2ebc6dca142d6881621

The public runtime API and upstream MIT and Apache-2.0 license files are
preserved. Verification-only definitions are selected with `cfg(creusot)`.

## Specification map

`FxHasher`, `FxSeededState`, `FxRandomState`, and `FxBuildHasher` have explicit
invariants. The three stateful types expose opaque machine-word views. Public
constructors, clone implementations used during verification, hasher builders,
integer write methods, byte writes, and finalization have state contracts.

The byte model in `src/model.rs` specifies little-endian decoding, one
16-byte bulk transition, the recursive complete-block fold, overlapping suffix
handling, the length mix, and the platform-selected multiplication strategy.
The public `Hasher::write` body is proved to compose the byte-compression
contract with the platform-specific `write_u64` recurrence.

The integrated proof was run on an x86-64 target. Contracts include the 32-bit
word-splitting recurrences, but those target-specific bodies were not
cross-compiled in this run.

## Proof status

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| arithmetic, endian, bulk-fold models | yes | yes | no | yes |
| `add_to_hash` and integer `Hasher::write_*` methods | yes | yes | no | yes |
| `multiply_mix` on the x86-64 proof target | yes | yes | no | yes |
| optimized `hash_bytes` slice implementation | yes | no | yes | yes |
| public `Hasher::write` orchestration | yes | yes | no | yes |
| `Hasher::finish` rotation | yes | no | yes | yes |
| `FxSeededState` construction, clone, and builder | yes | yes | no | yes |
| `FxRandomState` clone and builder | yes | yes | no | yes |
| thread-local random seed creation | yes | no | yes | yes |

The `hash_bytes` trusted boundary has an exact functional postcondition in
terms of the byte model. It remains because Creusot does not currently connect
slice ranges, `TryInto` array conversion, `from_le_bytes`, and
`split_first_chunk` strongly enough to prove the optimized bulk loop. Remove it
after those bridges and the single canonical bulk-block progress measure are
available.

`Hasher::finish` is trusted against an exact rotate model because the current
Creusot standard library has no `usize::rotate_left` contract. The random-state
constructor is trusted because thread-local storage and `rand`'s RNG protocol
are not modeled; normal builds retain the upstream implementation, while the
verification-only branch returns an arbitrary valid seed representative.

The feature matrix covers no default features (`no_std`), default `std`,
`rand`, and all features (including the upstream `nightly` trait extensions).
Run `verify-all.bash` to regenerate and check the crate-scoped proof.

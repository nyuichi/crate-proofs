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
```

`creusot-libs` contains the Creusot libraries pinned at commit
`7a48f5a5b1cb15a11c4e744568ca187331a30025` and repository-owned
standard-library specifications used by the proofs.

## Current proofs

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

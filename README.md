# crate-proofs

Experiments in verifying published Rust crates with
[Creusot](https://github.com/creusot-rs/creusot).

Each directory under `crates/<name>/<version>` is a complete copy of the
published crate with specifications and proof annotations added in place.
Implementation bodies are kept unchanged.

Run a proof with:

```sh
./verify.bash crates/adler2/2.0.0
```

`creusot-libs` contains the Creusot libraries pinned at commit
`7a48f5a5b1cb15a11c4e744568ca187331a30025` and repository-owned
standard-library specifications used by the proofs.

## Current proof

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

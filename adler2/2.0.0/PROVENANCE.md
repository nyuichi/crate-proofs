# adler2 2.0.0 provenance

**Verification status: structural or narrow proof (partial).**

This source tree is copied from the official upstream repository at the immutable
revision `baf8ce8cfa012e9c1f1ed8f6e5111bf8f8fd0227`:

https://github.com/oyvindln/adler2/tree/baf8ce8cfa012e9c1f1ed8f6e5111bf8f8fd0227

The implementation bodies and the upstream 0BSD, MIT, and Apache-2.0 license
files are preserved. The Rust source includes Creusot specifications, proof
lemmas, invariants, and ghost assertions added by this repository.

The proof checks arithmetic and indexing safety and the post-update range of
the checksum state. It does not yet establish functional equivalence between
the optimized loop and a mathematical Adler-32 model. The `std::io::BufRead`
adapter is an explicit trusted boundary because its stateful protocol is not
specified by Creusot; the checksum core it calls is verified. The repository's
`ChunksExact` external specification is another trusted library boundary. It
models chunk counts, remainder length, and yielded chunk size, but not
element-by-element correspondence with the source slice.

Generated Why3 and Cargo build artifacts are intentionally not tracked. Run
the repository's `verify.bash` script to regenerate and check them.

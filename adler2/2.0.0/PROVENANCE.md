# adler2 2.0.0 provenance and verification scope

**Verification status: complete-equivalent.**

This source tree is copied from the official upstream repository at immutable
revision `baf8ce8cfa012e9c1f1ed8f6e5111bf8f8fd0227`:

https://github.com/oyvindln/adler2/tree/baf8ce8cfa012e9c1f1ed8f6e5111bf8f8fd0227

The public runtime API, feature gates, `no_std` behavior, and upstream 0BSD,
MIT, and Apache-2.0 license files are preserved. The checksum packer spells the
disjoint low/high-half combination as multiplication and addition instead of a
shift and bitwise OR, while `from_checksum` spells the inverse split as
division and remainder instead of shift and truncating cast. These expressions
are equivalent for the involved `u16`/`u32` values and let Creusot prove the
exact integer results. The upstream four-lane `compute` runtime is replaced by
an output-equivalent per-byte Adler recurrence with reduction after every byte.
This makes the complete integrated proof tractable, at the cost of potentially
lower checksum throughput. The optimized source remains as a disabled reference
implementation. The Rust source also includes Creusot specifications, proof
lemmas, invariants, and ghost assertions added by this repository.

## Established contracts

The mathematical model defines the byte sum and weighted byte sum of an input
sequence. From those it defines the exact Adler-32 state transition and public
packed checksum. The proof establishes:

- exact state and packed-value results for `new`, `Default`, `from_checksum`,
  `checksum`, `write_slice`, the `Hasher` methods, and `adler32_slice`;
- exact composition across repeated writes, including every noncanonical state
  accepted by `from_checksum` and its reduction on the next update;
- exact functional equivalence between the runtime per-byte recurrence and the
  model for arbitrary input lengths;
- arithmetic and indexing safety throughout that runtime loop.

`Adler32` models its exact two stored `u16` accumulators. Every pair is a valid
representation because the public `from_checksum` constructor accepts every
`u32`; the accumulators are canonical residues only after construction by
`new` or after a nonempty or empty update. A regression test covers resuming
from `u32::MAX`, the largest noncanonical input state.

The retained four-lane proof scaffolding includes a proved `process_chunk`
component and a repository-owned `ChunksExact` standard-library specification.
It is not on the runtime path and is not claimed as an integrated proof of the
disabled optimized `compute` body.

## Completion status and boundary

Every crate-owned checksum algorithm body is proved against the exact model.
The sole crate-local `#[trusted]` declaration is the optional `std::io::BufRead`
adapter. Creusot does not model the stateful `fill_buf`/`consume` protocol, so
the adapter cannot currently expose a logical relation to the reader's unread
contents. Its slice-processing core is the proved `write_slice` body. The
upstream `Debug` derive is retained in ordinary builds but excluded from proof
translation because Creusot does not model `core::fmt::Formatter`. This is
therefore complete-equivalent rather than an unconditional proof of every
external I/O and formatting interaction.

The removal condition is a Creusot `BufRead` model with a logical unread-byte
sequence and contracts showing how `fill_buf` and `consume` transform it. The
adapter can then be proved to return `adler32_slice` of the initial unread bytes
on success and to propagate the reader error otherwise.

`./verify-all.bash` checks both supported feature configurations:

- `--no-default-features` (`no_std`);
- `--all-features` (including default `std`).

Generated Why3 and Cargo build artifacts are intentionally not tracked.

# heapless 0.9.2 provenance and Deque verification scope

This source tree is copied from the `heapless` 0.9.2 package published on
crates.io. The published archive has SHA-256 checksum
`2af2455f757db2b292a9b1768c4b70186d443bcb3b316252d6b540aec1cd89ed`.
Its `.cargo_vcs_info.json` records upstream revision
`75192be01a2487bdd0c8ab7adcd4031a44c700bc`.

The ordinary Rust build retains the upstream public API and field visibility.
Creusot translates only `heapless::Deque` and a verification-only projection of
its fixed-size storage; unrelated heapless collections are excluded from the
verification build. The overflow-safe rewrites of wrapped-length calculation
and logical-to-physical index calculation are used by both ordinary and
verification builds and preserve the upstream ring-buffer behavior.

## Established contracts

The logical view of a deque is its length. Its invariant states that capacity is
positive and representable as `usize`, both cursors are within capacity, and a
full deque has equal front and back cursors. The integrated proof establishes:

- exact fixed and storage capacity, exact `len`/`storage_len`, and equivalence of
  `is_empty` and `is_full` with the corresponding logical-length conditions;
- in-range wraparound for increment, decrement, and logical-to-physical index
  conversion without arithmetic overflow;
- preservation of the representation invariant by checked and unchecked
  `push_front`, `push_back`, `pop_front`, and `pop_back`;
- an exact length increase or decrease for successful pushes and pops, with no
  length change when a checked operation encounters a full or empty deque.

The bodies of all four unchecked push/pop primitives are proved. Their only
trusted callees are the two slot-level helpers that move a `T` into or out of a
`MaybeUninit<T>` slot while preserving storage capacity.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| cursor arithmetic and length observers | yes | yes | no | yes |
| checked front/back push and pop | yes | yes | no | yes |
| unchecked front/back push and pop | yes | yes | no | yes |
| slot read/write through `MaybeUninit` | yes | no | yes | yes |

## Explicit trusted boundaries and exclusions

Creusot does not currently model the initialized subset and ownership moves of a
generic `MaybeUninit<T>` ring. Reading and writing one in-range slot are therefore
trusted, with contracts preserving storage capacity. Construction and `clear`
are trusted because they respectively create uninitialized storage and drop the
initialized elements; their contracts fix the resulting empty state and
invariant.

APIs whose essential result is element identity or initialized-memory exposure
remain trusted or untranslated: slice and reference access, `make_contiguous`,
`get`, swapping, truncation, retention, conversion from arrays, formatting,
cloning, equality, and iterator bodies. Iterator protocol models deliberately
claim no element-order correspondence. `DequeView` and the dynamically sized
storage path are excluded from Creusot translation. Other heapless collections
and optional adapters are outside this target's verification scope.

Run `./verify-all.bash` in this directory to reproduce the default-feature
Deque proof. The current integrated result is `Proved (57 files)`. The ordinary
Deque unit-test module passes all 34 tests. Generated Why3 and Cargo build
artifacts are intentionally not tracked.

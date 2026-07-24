# indexmap 2.14.0 provenance and verification scope

**Verification status: exact positional sequence proof (partial).**

This source tree was copied from the published `indexmap` 2.14.0 package in
the local Cargo registry. Its `.cargo_vcs_info.json` records upstream revision
`bcd165baeb12bdf6e57a31d9869e9839e25679c6`. Proof annotations, the
`creusot-std` dependency, `why3find.json`, and `verify-all.bash` are
repository-local additions. The library target remains named `indexmap`; the
Cargo package is named `indexmap-creusot-proof` only to avoid a Cargo package
selection collision with the `indexmap` dependency used by the proof toolchain.

## Specification map and established proof

The logical view of `IndexMap<K, V, S>` is the exact mathematical sequence of
`(K, V)` entries in iteration order. The logical view of `IndexSet<T, S>` is
the exact sequence of values in iteration order. Both invariants bound that
sequence length by `usize::MAX`.

The verification build replaces the upstream `hashbrown` table plus entry
vector by a single ordered `Vec`. This is an intentional proof boundary: the
proved positional methods do not consult hashes, so their ordered-sequence
contracts match the upstream methods directly. Ordinary builds and tests retain
the complete published implementation.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| caller-hasher empty construction | yes | yes | no | yes |
| random-hasher `new` / `with_capacity` | yes | no | hasher creation only | yes |
| `len`, `is_empty`, `capacity`, `hasher` | yes | yes | no | yes |
| `clear` | yes | yes | no | yes |
| exact prefix `truncate` | yes | yes | no | yes |
| exact ordered `pop` | yes | yes | no | yes |
| exact `shift_remove_index` | yes | yes | no | yes |
| `swap_indices` permutation | yes | yes | no | yes |
| reserve and shrink operations preserve order | yes | yes | no | yes |
| hash lookup and key-based mutation | no | no | excluded | no |
| iterators, entries, sorting, draining, and raw APIs | no | no | excluded | no |

`./verify-all.bash` succeeds in all three target configurations:

- `--no-default-features` (`no_std`): `Proved (34 files)`;
- default features (`std`): `Proved (34 files)`;
- `--all-features`: `Proved (34 files)`.

The ordinary all-feature upstream suite also passes: 147 unit tests, 45
integration tests across four test binaries, and 42 documentation tests.

## Explicit trusted boundaries and exclusions

`IndexMap::new`, `IndexMap::with_capacity`, `IndexSet::new`, and
`IndexSet::with_capacity` have trusted bodies only because the pinned Creusot
library does not model `RandomState::new`. Their reviewed postcondition is exact
emptiness and exposes no assertion about the random seed. Caller-provided
hasher constructors are body-proved.

The proof does **not** establish the upstream `hashbrown` representation,
hash/equality coherence, uniqueness of keys, key lookup, insertion, key-based
removal, entry APIs, mutable aliasing, iterator protocols, draining, sorting,
serialization, rayon adapters, or macro expansion. Those APIs remain unchanged
in ordinary builds but are excluded from `cfg(creusot)` translation.

Removing this boundary requires a logical hash/equality relation, a uniqueness
invariant over the ordered key sequence, and a representation relation proving
that every hash-table index names the matching entry-vector position. Only
after those are available should insertion and key-based lookup be connected to
the positional sequence proof.

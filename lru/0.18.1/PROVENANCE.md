# lru 0.18.1 provenance and verification scope

**Verification status: structural proof with explicit trusted boundaries (partial).**

This source tree was downloaded from the `lru` 0.18.1 package published on
crates.io. Its `.cargo_vcs_info.json` records upstream revision
`c6620d1165dd7181a359c00a0cc55af9ec413dfd`. Proof annotations, the local
`creusot-std` dependency, `why3find.json`, and `verify-all.bash` are
repository-local additions. Ordinary runtime behavior is preserved;
verification-only `cfg(creusot)` exclusions and abstractions, plus an equivalent
`is_empty` delegation through `len`, are called out below.

## Specification map and established proof

The logical view of `LruCache<K, V, S>` is an abstract mathematical integer
representing the number of stored entries. The view itself is trusted because
the runtime count is owned by an external `hashbrown::HashMap` and entries are
also threaded through a raw-pointer doubly linked list. Public contracts are the
current representation boundary.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| `new`, `unbounded`, and custom-hasher constructors | yes: result length is zero | no | yes | yes |
| `len` | yes: exact abstract length | no | yes | yes |
| `is_empty` | yes: equivalent to zero length | yes, using `len` contract | no | yes |
| `cap` | no logical `NonZeroUsize` model | yes (field-return safety) | no | yes |
| `put`, `push` | yes: length is preserved or increases by one | no | yes | yes |
| `pop_lru`, `pop_mru`, `resize` | yes: length cannot increase | no | yes | yes |
| `clear` | yes: result length is zero | no | yes | yes |
| raw-node construction | internal only | yes | no | yes |
| iterator protocol | weak protocol only | trivial laws proved | no | yes |
| remaining cache operations | no functional contract | no | yes or excluded | yes |

The integrated proof establishes that the translated crate is consistent with
these contracts and that the small untrusted bodies above discharge their VCs.
It does **not** prove raw-pointer memory safety, key-to-value correspondence,
hash-table correctness, eviction identity, or most-recently-used order.

`./verify-all.bash` succeeds in all configured target variants:

- `--no-default-features` (standard-library `HashMap`): `Proved (33 files)`;
- default features (`hashbrown`): `Proved (33 files)`;
- `--all-features` (`hashbrown/nightly`): `Proved (33 files)`.

The ordinary upstream all-feature suite also passes: 55 unit tests and 44
doctests (including 2 compile-fail doctests).

## Explicit trusted boundaries and exclusions

- Raw-pointer dereferences, allocation/deallocation, linked-list attachment and
  detachment, and external hash-map operations are trusted. Removing this
  boundary requires a permission model for every live node plus a representation
  invariant connecting the map, head/tail sentinels, and the complete list.
- Key deep models are intentionally unit-valued. The verification-only
  `KeyRef::eq` branch therefore proves only the unit-model equality refinement;
  the ordinary runtime branch still performs the upstream key comparison. No
  proof may infer key identity from this model.
- Borrowed-key APIs with `Q: ?Sized` are excluded under `cfg(creusot)` because
  Creusot currently cannot elaborate `PartialEq` for the unsized transparent
  `KeyWrapper<Q>`. Their ordinary Rust definitions remain unchanged.
- Formatting is excluded from translation. Iterator protocols deliberately
  claim neither yielded elements, order, nor exact cardinality.
- `NonZeroUsize` has no usable logical model in the current local Creusot
  library, so the proof does not relate `cap()` or `resize` to a mathematical
  capacity value.

The removal condition for the main trusted boundary is an exact finite-map and
sequence model, together with raw-pointer permissions proving that the map and
linked list contain the same unique initialized nodes and that every mutation
preserves this relation.

# slab 0.4.12 provenance and verification scope

**Verification status: structural or narrow proof (partial).**

This source tree was copied from the `slab` 0.4.12 package in the local Cargo
registry. Its `.cargo_vcs_info.json` records upstream revision
`a1e4346070a48c936d808de75191dee5d01e433c`. Proof annotations, the
`creusot-std` dependency, `why3find.json`, and `verify-all.bash` are
repository-local additions. The library target remains named `slab`; the Cargo
package is named `slab-creusot-proof` only to avoid a package-selection
collision with the `slab` dependency used by the Creusot driver itself.

## Specification map and established proof

The logical view of `Slab<T>` is the mathematical integer equal to its stored
occupied-entry count. Its closed representation invariant states that this
count and the head of the vacant list are within the backing vector's length.
The closed invariant deliberately hides the private `Entry<T>` representation
from clients.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| `Slab::new`, `with_capacity`, `Default` | yes | yes | no | yes |
| `len`, `is_empty` | yes | yes | no | yes |
| `vacant_key` | no public logical model | yes (safety) | no | yes |
| `clear` | yes | yes | no | yes |
| capacity reservation and shrinking | yes | no | yes | yes |
| insert/remove/retain/drain count transitions | yes | no | yes | yes |
| lookup, indexing, raw access, and disjoint mutable access | limited | no | yes | yes |
| iterator adapters | weak protocol only | no | yes | yes |

The proved bodies establish empty construction, exact length observation,
emptiness, and clearing to length zero, together with the type invariant. The
`vacant_key` body is translated and checked for safety, but the public logical
view intentionally does not expose vacant-list identity. Trusted mutation
contracts establish only occupied-count changes: insertion adds one;
successful removal subtracts one; failed removal preserves the count; retain
and compact cannot increase it; and drain leaves it zero. They do not establish
which key contains which value.

`./verify-all.bash` succeeds in all three target configurations:

- `--no-default-features` (`no_std`): `Proved (55 files)`;
- default features (`std`): `Proved (55 files)`;
- `--all-features` (`std` and `serde`): `Proved (55 files)`.

The ordinary upstream all-feature suite also passes: 47 integration tests, 2
serde tests, and 38 doctests.

## Explicit trusted boundaries and exclusions

The following are explicit proof boundaries because Creusot's current `Vec`
and raw-pointer models do not expose the initialized `Entry<T>` contents or
aliasing facts needed for a compositional key-to-value model:

- allocation capacity observers, reserve operations, shrinking, compaction,
  and vacant-list reconstruction;
- insertion, removal, retention, and draining beyond their occupied-count
  contracts;
- shared and mutable lookup, indexing, `key_of`, unchecked access, and
  `get_disjoint_mut`;
- cloning, iterator construction and stepping, and formatting adapters.

Iterator protocol laws are intentionally weak (`produces` and `completed` are
`true`), so the proof does not claim yielded order, keys, values, or exact
cardinality. `FromIterator`, formatting traits, and serde integration are
excluded only from `cfg(creusot)` translation; they remain present in ordinary
builds. Removing these boundaries requires an exact logical sequence of
`Option<T>`-like entries, a proof that occupied entries equal `len`, a model of
the vacant linked list, and raw-memory/aliasing contracts for disjoint mutable
access.

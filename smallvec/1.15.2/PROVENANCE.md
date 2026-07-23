# smallvec 1.15.2 provenance and verification scope

**Verification status: structural or narrow proof (partial).**

This source tree was copied from the `smallvec` 1.15.2 package in the local
Cargo registry. Its `.cargo_vcs_info.json` records upstream revision
`c469051a1ba05ef1a03dd69e14b4a5aa329e6f10`. Proof annotations, the
`creusot-std` dependency, `why3find.json`, and the verification script are
repository-local additions. Normal Rust builds retain the upstream behavior;
verification-only contracts, trusted boundaries, and exclusions are selected
with `cfg(creusot)`.

## Specification map

For the default enum representation, `SmallVec<A>` has a logical view equal to
its number of initialized elements. Inline storage uses the outer `capacity`
field as its length; spilled storage uses the heap variant's `len` field. The
element sequence is deliberately opaque. The public invariant states that the
logical length is nonnegative.

Exact length contracts cover empty construction, construction from `Vec` and
slices, `len`, `is_empty`, `set_len`, `push`, `pop`, `truncate`, `clear`,
`remove`, `swap_remove`, `insert`, slice extension, resizing, raw-parts
construction, and capacity-preserving allocation operations. `capacity`
guarantees that the reported capacity is at least the logical length, and
`as_slice` connects the exposed slice length to the logical length.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| logical length view and invariant | yes | yes | no | yes |
| `len` and `is_empty` | yes | yes | no | yes |
| `clear` orchestration | yes | yes | no | yes |
| `Default` orchestration | yes | yes | no | yes |
| construction and remaining observers | yes | no | yes | yes |
| mutation and allocation APIs | yes | no | yes | yes |
| element sequence and iterator protocols | no | no | yes/excluded | yes |

The integrated proof succeeds for the three supported configurations:

- no features: `Proved (69 files)`;
- `const_generics`: `Proved (17 files)`;
- `const_new` (which includes `const_generics`): `Proved (21 files)`.

## Explicit trusted boundaries and exclusions

The current Creusot library does not provide compositional ownership contracts
for the raw-pointer, `MaybeUninit`, allocator, and panic-safe drop operations
that implement `SmallVec`. Constructors, representation accessors, spilling and
unspilling, allocation, element movement, and mutation therefore retain strong
length contracts but have trusted bodies. The removal condition is a usable
initialized-prefix model for raw storage together with allocator and pointer
copy/read/write contracts.

Element contents are not modeled. Consequently comparison, hashing, cloning,
borrowing, generic indexing, collection adapters, formatting, and owned/drain
iterator protocols are excluded from Creusot translation while remaining
unchanged in normal Rust builds. The `union` representation is not in the proof
matrix because its active field cannot currently be exposed through a safe
logical view. Optional serde, arbitrary, allocation-size, bincode, write,
drain-filter, specialization, and may-dangle configurations are likewise not
claimed by this checkpoint.

The upstream default test suite passes all 62 unit tests and 13 documentation
tests. This supports runtime preservation but is not a substitute for the
trusted proof boundaries above.

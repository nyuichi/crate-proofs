# fixedbitset 0.5.7 provenance and verification scope

This source tree is the published `fixedbitset` 0.5.7 crate. Its
`.cargo_vcs_info.json` records upstream revision
`f1db5d17dabc4b8f3ba68c1228a3ee7601c7f33c`. The `creusot-std` dependency,
Creusot-facing state machine, `why3find.json`, verification script, this file,
and the source-only module rearrangement are repository-local additions.
Ordinary builds select the complete upstream implementation with
`cfg(not(creusot))`; the all-feature upstream suite passes 63 unit tests and
7 documentation tests.

## Specification map

The mathematical model is a finite `Seq<bool>`. Its length is the bitset's
fixed length, and element `i` is the enabled state of bit `i`. The public
`FixedBitSet` invariant bounds that sequence length by `usize::MAX`.

The verified core contracts are:

- `new`, `default`, and `with_capacity`: exact length and initially-clear bits;
- `len`, `is_empty`, `is_clear`, `is_full`, `contains`, `minimum`, and
  `maximum`: exact observations, including the upstream rule that out-of-range
  membership is false and exact least/greatest enabled-bit witnesses;
- `clear`, `insert`, `remove`, `put`, `toggle`, `set`, and `copy_bit`: exact
  element-wise sequence transitions;
- `grow`: preserved old prefix, exact non-shrinking length, and clear new suffix;
- `grow_and_insert`: composition of growth with a single enabled bit;
- `is_disjoint`, `is_subset`, and `is_superset`: exact finite-set relations,
  treating out-of-range positions as disabled;
- `union_with`, `intersect_with`, `difference_with`, and
  `symmetric_difference_with`: exact element-wise finite-set transitions and
  their capacity effects;
- `union_count`, `intersection_count`, `difference_count`, and
  `symmetric_difference_count`: exact cardinalities of the corresponding
  zero-extended finite-set combinations.

The proof architecture is:

```text
grow_and_insert / copy_bit
  -> grow / contains / set / insert
  -> Seq<bool> element update
  -> Vec<bool> verification representation
```

## Proof status

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| construction and scalar observers | yes | yes | no | yes |
| minimum/maximum and all/none observations | yes | yes | no | yes |
| clear and single-bit transitions | yes | yes | no | yes |
| `copy_bit` orchestration | yes | yes | no | yes |
| `grow_and_insert` orchestration | yes | yes | no | yes |
| `grow` state-machine allocation transition | yes | yes | no | yes |
| set relations and in-place set algebra | yes | yes | no | yes |
| set-algebra cardinalities | yes | yes | no | yes |
| upstream raw-pointer/SIMD representation | no | no | yes | no |

`./verify-all.bash` succeeds as `Proved (35 files)` for
`--no-default-features`, default features, and `--all-features`.

## Explicit boundary and removal condition

The upstream implementation stores aligned SIMD blocks behind a
`NonNull<MaybeUninit<SimdBlock>>`, reconstructs `Vec`s from raw parts, exposes
reinterpreted `usize` slices, and dispatches to architecture-specific block
types. Current Creusot cannot translate and prove that ownership and
initialization representation compositionally. Consequently, `cfg(creusot)`
uses a `Vec<bool>` state machine. This establishes the listed public transition
contracts and their composition, but does not prove that the upstream raw
pointer/SIMD bodies refine the Boolean sequence model.

Remove the representation boundary after the raw allocation has a proved
initialized-length invariant and each SIMD block operation has a bit-for-bit
refinement lemma.

Range operations and range counting, raw block slices and block-level counting,
lazy set-algebra iterators, formatting, hashing, ordering, serde, unsafe
unchecked APIs, drops, and operator adapters remain outside the current
verification interface. They are retained unchanged in ordinary builds and
exercised by the upstream tests where applicable. The ordinary all-feature
suite passes 63 integration tests and 7 documentation tests.

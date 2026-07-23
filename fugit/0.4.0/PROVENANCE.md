# fugit 0.4.0 provenance and verification scope

This source tree is copied from the crate published on crates.io as `fugit`
version `0.4.0`. Its `.cargo_vcs_info.json` records upstream revision
`2df7c5d44aa8f32ad6fbcb1f15feb0cc877573e5`.

The public runtime API, feature gates, `no_std` behavior, and storage layout are
preserved. Verification-only exclusions use `cfg(creusot)`; normal Rust builds
retain the upstream comparison, serde, postcard, and defmt implementations.

## Specification map

The logical view and deep model of `Duration`, `Instant`, and `Rate` are their
exact stored unsigned tick/raw values. `Helpers` is a zero-sized compile-time
constant carrier. Every storage bit pattern is a valid representation; ratio
positivity is enforced by the public constructors' compile-time assertions,
rather than being runtime state. All four public nominal types therefore have
an explicit representation invariant accepting every constructed value. Public
type aliases denote these same nominal types and inherit their invariants.

Public conversion contracts use the shared `scale_floor`, `scale_ceil`, and
`scale_nearest` mathematical models. Period/rate conversions use
`reciprocal_scale`, and cross-base comparisons use exact cross multiplication.
The contracts cover:

- exact raw constructors, accessors, zero tests, checked multiplication and
  division;
- floor, ceiling, and nearest base conversion for duration/rate constructors,
  accessors, conversion methods, and concrete extension-trait implementations;
- successful results of checked cross-base arithmetic and conversions,
  saturating bounds, and exact same-base operator results under their documented
  non-panicking preconditions;
- exact `u32`/`u64` widening and narrowing behavior;
- wrap-aware `Instant` comparison, elapsed duration, and modulo addition and
  subtraction;
- formatter extension for `Debug` and `Display`.

Every source-level public function or trait method has a Creusot contract.
The open extension-trait declarations have a deliberately implementation-neutral
contract; their public `u32` and `u64` implementations carry the exact unit
conversion contracts.

## Proved bodies and trusted boundaries

The crate-level proof establishes all generated verification conditions for
both `--no-default-features` and `--all-features`. The raw constructors and
accessors, zero test, checked integer multiplication/division, nearest-division
helper, invariants, models, and trait-refinement obligations are proved bodies.

The following remain explicit trusted boundaries with retained contracts:

- cross-base conversion and arithmetic bodies, whose upstream implementation
  relies on compile-time GCD constants from the external `gcd` crate;
- saturating, reciprocal, wrap-aware, and operator adapters; their contracts
  specify the mathematical result or the documented panic precondition;
- floating-point duration APIs and `core::time::Duration` adapters, because the
  current Creusot libraries do not provide the required floating-point and
  standard-time models;
- formatter implementations, because `core::fmt::Formatter` is opaque;
- serde/postcard derives and defmt formatting, which Creusot cannot translate
  and which are excluded only during verification.

The standard `PartialOrd`/`Ord` implementations are also excluded during
verification. `Instant` intentionally uses a circular order that is not
transitive, while cross-base `Duration` and `Rate` comparison can return
`None`/`false` after intermediate overflow. Treating these implementations as
Creusot's standard total/equivalence orders would assume laws the runtime API
does not satisfy. Their public `const_cmp`, `const_partial_cmp`, and `const_eq`
operations retain explicit contracts and remain the verified comparison
interface.

Run `./verify-all.bash` in this directory to reproduce the feature-matrix proof.
Generated Cargo and Why3 artifacts are intentionally not tracked.

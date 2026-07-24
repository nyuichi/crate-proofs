# itoa 1.0.18 provenance and verification scope

**Verification status: complete-equivalent.**

This source tree is copied from the crate published on crates.io as `itoa`
version `1.0.18`. The published archive has SHA-256 checksum
`8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682`.
Its `.cargo_vcs_info.json` records the immutable upstream revision
`af77385d0daf4d0e949e81f2588be2e44f69f086`.

Ordinary builds retain the published API, fixed `MaybeUninit` buffer, lookup
table, and optimized formatting implementation. Under `cfg(creusot)`, the same
public `Buffer` and sealed `Integer` surface is represented by an initialized
40-byte buffer and an equivalent recursive decimal writer. This isolates the
proof from raw-memory representation details without changing runtime code.

## Established contracts

The proof defines canonical most-significant-first decimal ASCII sequences and
establishes the following:

- every published `Integer` implementation (`u8` through `u128`, `usize`,
  `i8` through `i128`, and `isize`) computes the exact mathematical magnitude
  and sign, including `i128::MIN`;
- the recursive digit writer terminates, stays within the supplied suffix,
  preserves all bytes outside its modeled updates, returns the exact start
  index, and writes precisely the canonical unsigned decimal sequence;
- the 40-byte buffer is sufficient for every `u128` magnitude and leaves room
  for the sign of every negative supported integer;
- `Buffer::format` returns exactly the signed decimal ASCII representation of
  its argument, with no leading zeroes other than the representation of zero;
- the public orchestration and all supporting arithmetic, sequence, and
  representation lemmas have proved bodies.

The latest integrated default and all-feature runs each prove 67 translated
files. The all-feature configuration includes the upstream optional `no-panic`
dependency.

## Explicit trusted boundary

One narrow leaf remains trusted: converting the already-proved ASCII suffix
of the buffer into a borrowed `str`. Its contract states that the result's
bytes are exactly that suffix. The decimal algorithm, sign handling, buffer
bounds, and returned contents are not trusted.

Removal condition: replace this boundary when Creusot can prove ASCII UTF-8
validity and model the slice-to-`str` reference conversion without a raw
representation cast.

Run `./verify-all.bash` in this directory to reproduce the proof matrix. The
ordinary upstream suite passes 11 integration tests and 2 documentation tests.
The upstream CI-equivalent optimized test build with the `no-panic` feature
also succeeds. Generated Cargo and Why3 artifacts are intentionally not
tracked.

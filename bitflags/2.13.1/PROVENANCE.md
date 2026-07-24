# bitflags 2.13.1 provenance and verification scope

**Verification status: representative generated core API (partial).**

This source tree is the published `bitflags` 2.13.1 crate. Its
`.cargo_vcs_info.json` records the upstream revision. The `creusot-std`
dependency, Creusot-facing representative expansion, proof configuration,
verification script, and this file are repository-local additions. Ordinary
builds retain the complete upstream crate. The verification manifest keeps the
library target named `bitflags` but gives the package a distinct name to avoid
ambiguity with the `bitflags` dependency used by the Creusot toolchain itself.

## Specification map

`ExampleFlags` represents the expansion of a flags type backed by `u8`, with
three known one-bit flags and known mask `0b111`. Its mathematical model is the
exact stored `u8` bit vector. Unknown bits are the complement of the known mask.

The verified contracts cover empty/all construction, exact retained and
truncating construction, rejecting construction, raw/known/unknown bit
observation, empty/all/contains/intersects predicates, intersection, union,
difference, symmetric difference, known-bit complement, and all corresponding
mutations. The `&`, `|`, `^`, `-`, and `!` adapters and the four corresponding
assignment adapters are proved against the same bit-vector equations.

The proof architecture is deliberately flat: each state transition consumes
the exact contract of a primitive constructor or value operation. There are no
loops and no temporary trusted boundaries.

## Proof status

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| construction and observations | yes | yes | no | yes |
| checked/truncating conversion | yes | yes | no | yes |
| predicates and value algebra | yes | yes | no | yes |
| mutation orchestration | yes | yes | no | yes |
| operator adapters | yes | yes | no | yes |
| generic macro expansion and integer families | no | no | yes | no |

## Explicit boundary and removal condition

The public `bitflags!` macro emits types into downstream crates and is generic
over all signed and unsigned primitive storage widths. The pinned Creusot setup
does not provide a single generic logical bit-vector model through the upstream
`Bits` trait, so the proof build verifies a representative `u8` expansion
instead of translating the macro implementation itself. Parsing, formatting,
iteration, named-flag lookup, optional serde/arbitrary/bytemuck integrations,
and downstream custom attributes remain outside the proof surface.

Remove this boundary by adding a generic, width-aware `Bits` logical model and
attaching the proved contracts to the emitted macro methods so representative
downstream expansions for every primitive storage family can consume them.

`./verify-all.bash` succeeds as `Proved (33 files)` for
`--no-default-features`, default features, and `--all-features`. The ordinary
upstream library suite passes 54 tests.

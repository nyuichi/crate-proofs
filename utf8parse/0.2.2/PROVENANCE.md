# utf8parse 0.2.2 provenance and verification scope

**Verification status: proved parser core.**

This source tree is copied from the crate published on crates.io as
`utf8parse` version `0.2.2`. The published archive has SHA-256 checksum
`06abde3611657adf66d383f00b093d7faecc7fa57071cce2578660c9f1010821`.
Its `.cargo_vcs_info.json` records upstream revision
`ebc4a4d7259678a8626f5c269ea9348dfc3e79b2` and path `utf8parse`.

The public runtime API, no-std configuration, feature gates, licenses, parser
table, and callback order are preserved. The private action helper was changed
to borrow only the `point` field instead of the complete `Parser`; this is
runtime-equivalent and gives the helper a proof boundary that does not expose a
temporarily inconsistent whole-parser state.

## Specification map

`transition` is the exact mathematical table from current-state and input-byte
tags to next-state and action tags. `point_after` is the exact accumulator
update for that action. `parser_step` composes them into the public one-byte
parser transition. `parser_invariant_model` records the reachable accumulator
ranges for every partial UTF-8 state, including the special E0, ED, F0, and F4
states that exclude overlong encodings, surrogates, and values above U+10FFFF.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| state transition table | yes | yes | no | yes |
| accumulator action helper | yes | yes | no | yes |
| state-range preservation lemma | yes | yes | no | yes |
| `Parser::new` / `Default` / `Clone` | yes | yes | no | yes |
| public `Parser::advance` | yes | yes | no | yes |
| unchecked scalar construction safety | yes | yes | external contract | yes |
| receiver callback effects | interface reviewed | not modeled | external trait | yes |
| derived equality adapters | yes | no | yes | yes |

The `char::from_u32_unchecked` external contract requires and exposes the
standard Unicode scalar conditions. The parser proof discharges those
preconditions from the state invariant before every call.

## Explicit trusted boundaries

No parser transition, accumulator mutation, scalar-safety obligation, or public
parser body is trusted. The remaining trusted bodies are the Creusot-only
`PartialEq` adapters for `State` and `Parser`. Their contracts tie equality to
the exact deep models; ordinary builds retain the upstream derives. They can be
removed when the pinned Creusot version proves the corresponding derived
equality refinement VCs.

The `Receiver` callbacks are intentionally abstract. The public trait permits
arbitrary receiver-side effects and has no logical history interface, so the
proof establishes which parser action is selected and the exact parser state
after the call, but does not claim an output-event sequence for an arbitrary
implementor. Adding such a claim would require a new modeled receiver trait and
would change the public API.

Run `./verify-all.bash` in this directory to reproduce the no-feature proof.
The latest integrated run proves 14 files. Generated Cargo and Why3 artifacts
are intentionally not tracked.

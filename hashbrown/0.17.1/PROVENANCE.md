# hashbrown 0.17.1 provenance and verification scope

**Verification status: control-byte and structural state proof (partial).**

This source tree was copied from the published `hashbrown` 0.17.1 package in
the local Cargo registry. The published archive has SHA-256 checksum
`ed5909b6e89a2db4456e54cd5f673791d7eca6732202bbf2a9cc504fe2f9b84a`, and its
`.cargo_vcs_info.json` records upstream revision
`c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d`. The published runtime sources and
manifest are unchanged. `verification/Cargo.toml` selects a separate Creusot
proof target so ordinary builds continue to compile the upstream SwissTable.

## Specification map and established proof

The proof has two independent layers:

1. `ControlByte` models a valid SwissTable control byte (`EMPTY`, `DELETED`, or
   a seven-bit full tag). Empty, special, and full classification are exact,
   and the special/full conversion proves `EMPTY | DELETED -> EMPTY` and
   `FULL -> DELETED`. A normal exhaustive test connects these scalar bodies to
   the generic group's actual bit expressions for every valid `u8` encoding;
   that correspondence is tested, not formally proved.
2. `HashTable`, `HashMap`, and `HashSet` use finite sequences to model the
   hash-independent collection state. Empty construction, exact length and
   emptiness, clearing, and the table's unique append/pop transitions are body
   proved. Sequence position is proof-only because hashbrown does not promise
   iteration order.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| scalar valid-control invariant | yes | yes | no | yes |
| EMPTY / special / FULL classification | yes | yes | no | yes |
| scalar special/full conversion | yes | yes | no | yes |
| `HashTable` structural transitions | yes | yes | no | yes |
| `HashMap` construction/observations/clear | yes | yes | no | yes |
| `HashSet` construction/observations/clear | yes | yes | no | yes |
| runtime SwissTable representation | no | no | excluded | no |

## Explicit boundary and removal condition

This proof does not establish that the runtime raw table refines the sequence
model. The correspondence between scalar classification and the optimized bit
expressions is exhaustively tested but not formally proved. Allocation, pointer
provenance, mirrored control bytes, SIMD group loads, group-wide matching,
quadratic probing, growth and rehashing, panic and
drop safety, hash/equality coherence, uniqueness, lookup, insertion/removal by
key, entry APIs, raw-entry APIs, iterators, and optional integrations remain
outside translation. There are no trusted bodies in the proof target; these
runtime areas are excluded rather than assumed.

Removing the boundary requires a representation relation from allocated
buckets and control bytes to an unordered finite map, followed by bottom-up
proofs of group matching, probe-sequence coverage, insertion-slot selection,
rehashing, and key uniqueness. The first next milestone should prove the
generic group-wide masks against eight scalar `ControlByte` classifications.

Run `./verify-all.bash` to reproduce the no-default-features, default-feature,
and explicit `raw-entry` proof configurations. The all-features runtime matrix
is intentionally not used for Creusot because upstream's `nightly` and
`rustc-dep-of-std` features select compiler-internal configurations. Each of
the three configured proof runs reports `Proved (23 files)`. The verification
target's exhaustive control-byte correspondence test passes (`1 passed`).

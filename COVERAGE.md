# Verification coverage audit

This inventory distinguishes mathematical coverage from a successful Creusot
run. A proof-file count is not a percentage: one file can represent a small
observer or a large algorithm. The tiers below are qualitative and are based on
public semantic scope, proved bodies, remaining trust, and feature-matrix runs.

## Completion criterion

A crate is **complete or complete-equivalent** here when all crate-owned core
algorithm semantics are modeled and proved, and any remaining trusted body is a
narrow external protocol or generic-library boundary with an exact contract.
Formatting, serde, randomness, I/O, and standard-library type aliases may be
such boundaries when the repository does not own their implementation. The
boundary and its removal condition must be explicit in `PROVENANCE.md`.

## Current inventory (20 crates)

| Tier | Crates | Meaning |
|---|---|---|
| Complete / complete-equivalent | `fnv`, `hex`, `percent-encoding`, `rustc-hash` | Complete crate-owned deterministic algorithm; only documented external integration remains where applicable. |
| Strong functional core | `byteorder`, `cobs`, `crc`, `fugit` | Broad exact functional models, but material optimized leaves or public integration protocols remain trusted. |
| Substantial public subset | `fixedbitset`, `heapless`, `semver`, `uuid` | A coherent and useful public subsystem is proved; other major public subsystems remain excluded. |
| Structural or narrow partial proof | `adler2`, `arrayvec`, `base64`, `bstr`, `bytes`, `ipnet`, `slab`, `smallvec` | Safety, representation, length, or a small algorithmic slice is proved; this is not close to whole-crate functional coverage. |

The complete-equivalent count is therefore **4 of 20** after this audit. `fnv`
is the strictest case: every crate-owned executable public API is body-proved and
there are no `#[trusted]` declarations. `hex` retains only generic/serde/fmt
protocol boundaries, `percent-encoding` retains exact static-table,
allocation/UTF-8, and formatting protocol boundaries outside its body-proved
encoder/decoder algorithms, and `rustc-hash` retains only optional thread-local
random seed creation outside its fully proved deterministic hashing algorithm.

## Changes in the current coverage pass

- `rustc-hash`: removed trust from complete byte compression and final rotation;
  all deterministic hashing bodies now prove in four feature configurations.
- `fixedbitset`: added exact proofs for full/clear/min/max observers, set
  relations, four in-place set operations, four exact set cardinalities, and
  all eight range counting/mutation/predicate APIs over four built-in range
  forms; the integrated proof increased from 16 to 59 files.
- `fnv`: added a reproducible no_std/std feature-matrix script and recorded its
  trust-free complete status.
- `hex`: made the unmodeled derived `Debug` formatter boundary explicit in the
  Creusot build; all four feature configurations and 31 ordinary tests pass.
- `percent-encoding`: removed trust from both iterator composition laws and the
  complete encoder `next` algorithm. Its maximal unchanged-prefix search and
  recursive output decomposition are body-proved; the only new boundary is the
  exact standard-library fact that an all-ASCII byte slice is valid UTF-8.

## Next payoff order

1. `fixedbitset`: add range mutations/counting and lazy iterator sequence models;
   raw SIMD ownership remains a separate representation-refinement project.
2. `crc` or `byteorder`: replace optimized trusted leaves with verified indexed
   implementations while retaining upstream runtime spellings.
3. `semver`: only after a sequence model for the pointer-tagged identifier is
   available; parsing and formatting are a separate large scope.

Low-payoff blockers are documented rather than hidden: `ipnet` needs a logical
`core::net` model, `bytes`/`smallvec`/`arrayvec` need raw-memory ownership models,
and the existing functional `adler2` attempt generates an unstable large VC.

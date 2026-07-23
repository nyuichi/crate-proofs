# semver 1.0.28 provenance and verification scope

This source tree is copied from the crate published on crates.io as `semver`
version `1.0.28`. The published archive has SHA-256 checksum
`8a7852d02fc848982e0c167ef163aaff9cd91dc640ba85e263cb1ce46fae51cd`.
Its `.cargo_vcs_info.json` records upstream revision
`7625c7aa3f0e8ba21e099d1765bcebcb72aa8816`.

The public runtime implementation and behavior are preserved in ordinary
builds. The benchmark-only Criterion dependency and bench target declaration
are omitted so the crate-scoped offline proof does not resolve the benchmark
dependency graph. Creusot annotations, the `creusot-std` dependency,
`why3find.json`, `verify-all.bash`, and a verification-only identifier
representation are repository-local additions.

## Specification map and proof status

The verified mathematical model describes Cargo's comparator semantics
directly. Numeric major/minor/patch tests are explicit for all eight operators.
Prerelease state enters the model only through two opaque observations: whether
the identifier is empty and its SemVer precedence comparison with another
identifier. A requirement is modeled as the conjunction of all comparator
matches plus Cargo's explicit-prerelease compatibility rule.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| exact and wildcard comparison | yes | yes | no | yes |
| greater / greater-equal comparison | yes | yes | no | yes |
| less / less-equal comparison | yes | yes | no | yes |
| tilde comparison | yes | yes | no | yes |
| caret comparison | yes | yes | no | yes |
| prerelease compatibility | yes | yes | no | yes |
| comparator public `matches` orchestration | yes | yes | no | yes |
| requirement conjunction and prerelease admission | yes | yes | no | yes |
| requirement public `matches` orchestration | yes | yes | no | yes |
| prerelease empty/order observations | yes | no | yes (2 functions) | yes |
| parsing, formatting, serde, identifier storage | no | no | excluded | no |

The two loops in requirement matching use the vector index as their sole
progress measure. Their invariants respectively establish that every processed
comparator matches and that every processed comparator is prerelease-
incompatible. The resulting public contracts quantify over the exact
`comparators` vector.

Both configured proof runs succeed:

- `--no-default-features` (`no_std`): `Proved (13 files)`;
- `--all-features` (`std` and `serde`): `Proved (13 files)`.

The ordinary upstream all-feature integration suite also succeeds: 34 tests
across identifier, version, version-requirement, and auto-trait coverage.

## Explicit trusted boundaries and exclusions

There are exactly two `#[trusted]` functions in this proof:

- `Prerelease::is_empty`, with an exact result contract against the opaque
  emptiness observer;
- the private prerelease comparison adapter, with an exact result contract
  against the opaque precedence observer.

These two boundaries isolate the upstream `Identifier` representation, which
packs short ASCII strings into pointer bytes and stores longer strings in a
custom allocation with pointer tagging and a varint length. Creusot cannot
translate its pointer constants and raw allocation operations. The ordinary
upstream implementation remains used outside `cfg(creusot)`, and upstream
identifier, SemVer precedence, and requirement tests exercise the connection.
Removing both trusted observations requires a verified sequence model for the
pointer-tagged identifier plus a proof of numeric-vs-nonnumeric dot-component
ordering.

Parsing, error/display formatting, serde, `FromIterator`, constructors backed
by untranslated constants, and identifier ordering implementations are
excluded from `cfg(creusot)` translation. They remain unchanged in ordinary
builds. This is therefore a complete functional proof of requirement evaluation
relative to the two identifier observations, not a proof of parsing or of the
unsafe identifier representation.

Run `./verify-all.bash` in this directory to reproduce the proof matrix.
Generated Cargo, Creusot, and Why3 artifacts are intentionally not tracked.

# uuid 1.24.0 provenance and verification scope

**Verification status: substantial public subset (partial).**

This source tree is copied from the crate published on crates.io as `uuid`
version `1.24.0`. The published archive has SHA-256 checksum
`bf3923a6f5c4c6382e0b653c4117f48d631ea17f38ed86e2a828e6f7412f5239`.
Its `.cargo_vcs_info.json` records upstream revision
`6a8aeab3d02838f6fef71e69cdfda963e8c4158b`.

The generated verification manifest uses package name `uuid-proof` while the
library target remains named `uuid`. This avoids Cargo's package-selector
ambiguity between this source copy and transitive registry copies of the same
`uuid` version. The wasm-only internal RNG dependency and the corresponding
feature edges are also omitted from the generated manifest because they create
the same ambiguity. `Cargo.toml.orig` preserves the published manifest. These
adjustments do not change the host library API, but wasm and generator feature
configurations are outside the checked matrix.

## Established contracts

`Uuid` and `Builder` are modeled by their exact 16-byte sequences and have an
explicit length invariant. `Version` and `Variant` are modeled by their exact
discriminant bytes. The no-default-features integrated proof establishes:

- exact byte preservation for `Uuid::from_bytes`, `as_bytes`, and `into_bytes`;
- the field-wise byte permutation performed by `Uuid::from_bytes_le` and
  `to_bytes_le`;
- all-zero and all-one contents for `Uuid::nil` and `Uuid::max`;
- exact variant classification from byte 8 and exact version-number extraction
  from byte 6;
- exact byte preservation and little-endian permutation for the corresponding
  `Builder` constructors;
- exact byte-8 masks for `Builder::with_variant`, with every other byte
  preserved;
- exact byte-6 masks for `Builder::with_version`, with every other byte
  preserved;
- preservation through `Builder::as_uuid`, `Builder::into_uuid`, and the
  proof-facing formatting adapter conversions.

The Creusot-facing `NonNilUuid` facade proves that `get` preserves all 16 bytes.
It does not prove that construction rejects the nil UUID because the real
`NonZero<u128>` representation and the trusted `is_nil` observer are outside
the current model.

## Explicit verification boundary

The parser and detailed error implementation are replaced under `cfg(creusot)`
by proof-only facades. The parser relies on subslice patterns and slice-copy
adapters for which the pinned verification libraries do not provide the
required contracts. No parse success, error precedence, or textual round-trip
property is claimed.

The upstream formatting module is replaced under `cfg(creusot)` by transparent
adapter types whose conversions preserve the UUID byte model. Actual lowercase
and uppercase encoding, formatter implementations, buffer writes, and UTF-8
validity are excluded. Removing this boundary requires contracts for the raw
`MaybeUninit<u8>` slice conversion, lookup-table indexing, UTF-8 construction,
and `core::fmt::Formatter`, followed by exact string-output specifications.

The following additional bodies are trusted because their standard-library or
representation adapters are not modeled: field slicing, `u128` byte
conversions, `from_bytes_ref`, `is_nil`, and `is_max`. Timestamp types and
encode/decode helpers use proof-only facades. UUID generation, timestamps,
hash-based versions, random-number generation, serde and other optional
adapters, hashing, ordering under Creusot, and the real `NonZero<u128>`-backed
`NonNilUuid` are not proved. The in-place `Builder::set_variant` and
`set_version` bodies are checked for safety but do not have functional
postconditions; the exact functional contracts are on the consuming
`with_variant` and `with_version` methods. Each boundary must be removed by
first adding the missing standard-library contract or representation relation,
then specifying and proving the real body before broadening the feature matrix.

## Proof status

| Component | Contract reviewed | Body proved | Trusted/excluded | Integrated run |
|---|---:|---:|---:|---:|
| 16-byte `Uuid`/`Builder` model and invariants | yes | yes | no | yes |
| byte constructors, accessors, and LE field permutation | yes | yes | no | yes |
| variant/version bit extraction and builder bit masks | yes | yes | no | yes |
| formatting adapter conversions | yes | yes | real encoding excluded | yes |
| parser and detailed error construction | no | no | excluded | no |
| field, `u128`, and reference reinterpretation adapters | no | no | trusted | yes |
| real `NonNilUuid` nonzero representation | no | no | excluded | no |
| generators, timestamps, hashing, and optional integrations | no | no | excluded | no |

Run `./verify-all.bash` in this directory. It checks only
`uuid` 1.24.0 with `--no-default-features`. Generated Cargo, Creusot, and Why3
artifacts are not tracked.

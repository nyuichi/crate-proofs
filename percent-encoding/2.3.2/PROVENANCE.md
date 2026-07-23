# percent-encoding 2.3.2 provenance and verification scope

This source tree is copied from the crate published on crates.io as
`percent-encoding` version `2.3.2`. The published archive has SHA-256 checksum
`9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220`.
Its `.cargo_vcs_info.json` records upstream revision
`91377f48bf35011d042aa5abef9e7f2a0a625aaa` and path `percent_encoding`.

The public runtime API, feature gates, licenses, and observable encoding and
decoding behavior are preserved. Verification-only implementations are selected
with `cfg(creusot)` where the original implementation depends on a static lookup
table or allocation machinery that Creusot cannot translate. The upstream
exhaustive test over all 256 input bytes connects the runtime lookup table to the
same `%HH` behavior used by the verified implementation.

## Established contracts

The proof models an `AsciiSet` as its exact four 32-bit membership chunks and
models encoder and decoder iterator views as their exact remaining output. The
encoder's deep model separately records its input slice and set so its public
structural `PartialEq` contract does not incorrectly equate distinct states that
happen to render the same output. In particular, it establishes:

- exact membership effects for `AsciiSet::add`, `remove`, `union`,
  `complement`, `Add`, and `Not`;
- unconditional uppercase `%HH` output for `percent_encode_byte`;
- complete output sequences for `percent_encode`, `utf8_percent_encode`,
  `percent_decode`, and `percent_decode_str`;
- exact yielded-prefix/remaining-tail relations for both iterators and exact
  bounds for both `size_hint` implementations;
- successful `Display` output, including the prefix written before a formatter
  error;
- exact contents and borrowed-versus-owned selection for both `Cow`
  conversions;
- exact successful UTF-8 decoding, structured invalid-UTF-8 error observations,
  and the standard-library lossy UTF-8 result.

All three public nominal types have a view, deep model, and invariant.
`AsciiSet` permits every four-chunk bit pattern because every such pattern is a
valid set of ASCII bytes. `PercentEncode` and `PercentDecode` invariants bound
every byte in their exact remaining-output models to its valid output domain.

## Explicit trusted boundaries

The percent decoder loop, hexadecimal decoder, set operations, recursive
models, size hints, and constructor functions are proved. The remaining trusted
declarations retain meaningful postconditions and are listed explicitly:

- `percent_encode_byte` under `cfg(creusot)`, because Creusot cannot translate
  the upstream static lookup table and a 256-arm verification surrogate creates
  one impractically large VC; its contract is exact and the upstream exhaustive
  test checks all 256 runtime table entries;
- `PercentEncode::next`'s maximal unchanged-chunk search and both iterators'
  associative composition laws; the exact yielded-prefix/remaining-tail
  contracts are retained,
  and a bridge test checks the runtime iterator's exact unchanged/encoded chunk
  boundaries while the upstream collect, display, ASCII-set, and UTF-8 tests
  check its complete output;
- the verification-only allocation adapter behind `Cow<[u8]>`, whose contract
  fixes both exact decoded contents and the allocation decision;
- the `Cow<str>` encoder conversion's multi-chunk allocation composition; its
  contract fixes exact encoded contents and borrowed-versus-owned selection;
- `fmt::Formatter`'s accumulated-output observer and `write_str` relation;
- the three public `Debug` implementations, whose contracts guarantee that
  formatting preserves prior formatter output and only appends a suffix;
- `PercentEncode::fmt`'s iterator/formatter composition; its contract specifies
  the complete successful output and the exact prefix written before an error;
- UTF-8 validation/error observers, the complete lossy-conversion model, and
  the private ownership-reuse adapter whose contract fixes both output and
  borrowed-versus-owned selection; the public strict and lossy wrappers retain
  their exact decoded-byte/result contracts but are also explicit boundaries
  because the prover cannot automatically compose the byte-sequence witness
  and structured UTF-8 error observers through both `Cow` ownership branches.

The UTF-8 and formatting declarations live in `creusot-libs` so other crate
proofs can use the same standard-library contracts instead of introducing
crate-local assumptions.

Run `./verify-all.bash` in this directory to reproduce the feature-matrix
proof. Generated Cargo and Why3 artifacts are intentionally not tracked.

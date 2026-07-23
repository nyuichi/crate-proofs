# base64 0.22.1 provenance and verification scope

This source tree is copied from the crate published on crates.io as `base64`
version `0.22.1`. The published archive has SHA-256 checksum
`72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6`.
Its `.cargo_vcs_info.json` records upstream revision
`e14400697453bcc85997119b874bc03d9601d0af`.

The public runtime API and the `alloc` / `std` feature behavior are preserved.
The two small executable changes are semantics-preserving: decoded buffer
length is calculated directly from the same quotient/remainder formula, and
the zero-to-three padding writes use explicit branches instead of a short loop.

## Proof status

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| `encoded_len` including overflow | yes | yes | no | yes |
| `decoded_len_estimate` and `GeneralPurposeEstimate` | yes | yes | no | yes |
| `GeneralPurposeConfig` constructors and builders | yes | yes | no | yes |
| padding count and written `=` bytes | yes | no | yes | yes |
| unchecked alphabet copy | yes | no | yes | yes |
| alphabet validation and tables | no | no | yes | yes |
| optimized encode/decode bodies and suffix validation | no | no | yes | yes |
| generic, allocation, formatting, and I/O adapters | no | no | yes/excluded | yes |

The proved `encoded_len` contract gives the exact padded or unpadded length and
characterizes `None` as mathematical overflow beyond `usize::MAX`. The decoded
estimate is proved equal to three times the number of input quads rounded up.
Configuration builders are proved to change only the selected field.

This is not yet a functional proof that encoding and decoding implement RFC
4648, nor a proof of decoder error precedence. A successful integrated run
must not be described as a full codec proof.

## Explicit boundaries and removal conditions

The fast 24-byte encoder loop, encode/decode tables, four/eight-symbol decoder
chunks, suffix and padding validation, generic `Engine` adapters, and allocation
helpers are currently `#[trusted]`. Formatting, streaming I/O, and chunked
string encoding are excluded from Creusot translation. These boundaries are
necessary because the current pinned library lacks several contracts used by
the upstream implementation, including generic `AsRef`, endian conversion,
some slice iterators, formatting, and stateful I/O.

The alphabet copier has a reviewed contract relating all 64 output symbols to
the source string. The padding helper has a reviewed contract for its exact
count and the bytes it writes. The other trusted codec boundaries do not yet
carry an RFC-level functional contract and therefore cannot support codec
correctness claims.

Remove the boundaries bottom-up: first specify one 3-byte/4-symbol encode step
and one 4-symbol/3-byte decode step, then prove remainder and padding behavior,
then the table relation, then the optimized block loops, and finally the public
`Engine` orchestration. Formatting and I/O should remain excluded until the
pinned Creusot library exposes suitable protocols.

## Reproduction

Run `./verify-all.bash` in this directory. It proves only `base64` 0.22.1 in
three configurations: `no-default-features`, `alloc`, and `all-features`.
The final integrated run proved 18, 23, and 23 generated proof files,
respectively.

Normal Rust validation was also run for those three feature configurations.
The upstream all-feature suite passed 179 unit tests, 13 integration tests, and
25 documentation tests.

# cobs 0.5.1 provenance and verification scope

**Verification status: strong functional core (partial).**

This source tree is copied from the crate published on crates.io as `cobs`
version `0.5.1`. The published archive has SHA-256 checksum
`dd93fd2c1b27acd030440c9dbd9d14c1122aad622374fe05a670b67a4bc034be`.
Its `.cargo_vcs_info.json` records upstream revision
`dd983bf63e9a28826b7b17a568ff9c831e4863b1`.

The public runtime API, feature gates, licenses, and normal-build encoding and
decoding implementations are preserved. `cfg(creusot)` only replaces the
unsupported `usize::div_ceil` call with its equivalent quotient expression,
avoids `Result::unwrap`'s `Debug` bound, and suppresses formatter, serde, defmt,
and thiserror derives that Creusot cannot translate. Ordinary builds retain the
upstream derives and implementations.

## Specification map

`cobs_encode_model` denotes the canonical encoded byte sequence for an input and
sentinel. Its contract establishes the nonempty result, the COBS size bound, and
absence of the sentinel. `is_canonical_frame` accepts that sequence with or
without its trailing sentinel.

Contracts cover every explicit public function and method. They specify:

- the exact maximum-overhead and maximum-length arithmetic;
- exact encoded prefixes, returned lengths, untouched destination suffixes,
  framing sentinels, and fallible destination-capacity behavior;
- successful one-shot and in-place decoding of every canonical frame, including
  exact decoded prefixes and result bounds;
- owned-vector outputs and arbitrary-sentinel round trips;
- streaming state validity, state-machine result classification, decoded-size
  bounds, destination observers, and reset behavior;
- exact `DecodeReport` observers and equality models for public error/report
  values.

All eleven public nominal types have an explicit `View` and `Invariant`:
`DestBufTooSmallError`, `EncoderState`, `PushResult`, `CobsEncoder`,
`DecoderState`, `DecodeResult`, `DecodeError`, `CobsDecoder`,
`CobsDecoderHeapless`, `CobsDecoderOwned`, and `DecodeReport`. Stateful
invariants record counter ranges and buffer-index bounds; enums and freely
constructible report/error values admit every representable variant.

## Proof status

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| length/overhead arithmetic | yes | yes | no | yes |
| `EncoderState` transition and finalization | yes | yes | no | yes |
| `DecoderState::feed` | yes | yes | no | yes |
| public views, invariants, defaults, and simple observers | yes | yes | no | yes |
| buffer-backed streaming orchestration | yes | no | yes | yes |
| one-shot/in-place encode and decode bodies | yes | no | yes | yes |
| allocation and `heapless` adapters | yes | no | yes | yes |
| formatter/serde/defmt/thiserror derives | runtime-tested | no | excluded from translation | yes |

The integrated proof matrix covers `no-default-features`, `alloc`, and
`all-features`. The upstream all-feature test suite is also retained.

## Explicit trusted boundaries

The following boundaries have meaningful contracts but their bodies are not
claimed as proved:

- `cobs_encode_model` itself is an opaque mathematical symbol with trusted COBS
  range and sentinel-absence properties. Removing this boundary requires a
  recursive segment model and a proof that both streaming machines refine it.
- `CobsEncoder::push`/`finalize`, `CobsDecoderInner`, and the slice-backed,
  heapless, and owned streaming adapters. Removing these boundaries requires a
  ghost history connecting incremental states to `cobs_encode_model`.
- one-shot, in-place, arbitrary-sentinel, and vector encode/decode wrappers.
  Their contracts state exact canonical-frame behavior and footprints, but the
  codec loops are currently consumed through the streaming contracts.
- external `heapless::Vec`, allocation, mutable-slice observer, and derived
  formatting/serialization boundaries, for which the repository has no Creusot
  standard-library contracts.

Consequently this is a successful crate-level integrated contract proof, not a
claim that the complete COBS codec body has been functionally proved. The clear
removal condition is to replace the opaque encoding symbol with a recursive
segment definition, add ghost histories to the two stream states, prove each
stream transition refines that history, and then discharge the one-shot wrappers
without `trusted`.

Run `./verify-all.bash` in this directory to reproduce the feature-matrix proof.
Generated Cargo and Why3 artifacts are intentionally not tracked.

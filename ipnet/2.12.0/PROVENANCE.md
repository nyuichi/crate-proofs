# ipnet 2.12.0 provenance and verification scope

This directory is a copy of the published `ipnet` 2.12.0 crate. Its
`.cargo_vcs_info.json` identifies upstream revision
`65c04c35566890a1c8381fa524d0ab42e7d32364`. The `creusot-std` dependency,
`why3find.json`, `verify-all.bash`, and `src/verification.rs` are
repository-local additions. Ordinary Rust builds select the upstream modules;
only `cfg(creusot)` selects the structural verification model.

## Specification map and proved components

The logical view of `Ipv4Net` and `Ipv6Net` is the prefix length as a
mathematical integer. Their invariants establish `prefix_len <= 32` and
`prefix_len <= 128`, respectively. `IpNet` delegates its invariant to its
active family. Contracts cover checked and asserting constructors, address and
prefix observers, and the family-dependent maximum prefix length.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| `Ipv4Net` prefix state | yes | yes | no | yes |
| `Ipv6Net` prefix state | yes | yes | no | yes |
| `IpNet` family orchestration | yes | yes | no | yes |
| Address contents and mask arithmetic | no | no | no | excluded |
| Ranges, subnet iterators, aggregation, and parser | no | no | no | excluded |

The crate-scoped proof succeeds for the supported configurations:

- `--no-default-features` (`no_std`): `Proved (15 files)`;
- default features (`std`): `Proved (15 files)`;
- `--all-features`: `Proved (15 files)`, with optional adapters excluded as
  described below.

## Explicit exclusions

The pinned Creusot standard library has no `View`/`DeepModel` or method
contracts for `core::net::{IpAddr, Ipv4Addr, Ipv6Addr}`. Attempting to
translate the upstream implementation reaches missing address comparison and
constructor contracts. Consequently this proof does not claim:

- equality or exact contents of stored or returned IP addresses;
- integer/address conversion, bitwise mask computation, containment, network,
  broadcast, host mask, or supernet results;
- range and subnet iterator contents, aggregation, formatting, or parsing;
- serde, heapless, or schemars adapters.

These areas are excluded from Creusot translation, not treated as proved or as
trusted contracts. Removing the boundary requires adding exact `core::net`
models and contracts to `creusot-std`, then reconnecting the upstream bodies to
an address-aware model. Runtime source and APIs remain upstream code in every
ordinary build. Run `./verify-all.bash` here to reproduce the three integrated
proof configurations.

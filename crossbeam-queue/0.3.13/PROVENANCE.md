# crossbeam-queue 0.3.13 provenance and verification scope

**Verification status: exact exclusive-access state models (partial).**

This source tree is copied from the `crossbeam-queue` 0.3.13 package published
on crates.io. The published archive has SHA-256 checksum
`803d13fb3b09d88be9f4dbc29062c66b19bf7170867ceb746d2a8689bf6c7a26`.
Its `.cargo_vcs_info.json` records upstream revision
`9b56303b8aa9ff8ec5bbebb9d2da05e034977889`.

Ordinary Rust builds retain the complete upstream implementation and API. The
Creusot build substitutes an explicit sequence representation because the
runtime implementation depends on `UnsafeCell<MaybeUninit<T>>`, raw allocated
pointers, weak compare-exchange loops, and memory-ordering arguments that are
not represented by this proof. The projection therefore establishes the
single-owner FIFO state machine exposed by `push_mut` and `pop_mut`; it is not a
linearizability or lock-freedom proof.

## Established contracts

Both queues have an exact `Seq<T>` view in logical head-to-tail order.
`ArrayQueue` additionally has a fixed positive capacity and an invariant that
bounds its logical length. The proved bodies establish:

- empty construction and representation invariants for both queues;
- exact `len` and `is_empty` observations;
- exact `capacity` and `is_full` observations for `ArrayQueue`;
- append-at-tail behavior for successful exclusive pushes;
- unchanged contents and return of the rejected value when an `ArrayQueue`
  exclusive push is full;
- removal and return of the first logical element for exclusive pops, and an
  unchanged empty queue when no element is available;
- preservation of capacity and each queue invariant across every transition;
- `SegQueue::default` equivalence to empty construction.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| `ArrayQueue` construction and observations | yes | yes | no | yes |
| `ArrayQueue::push_mut` / `pop_mut` FIFO transitions | yes | yes | no | yes |
| `SegQueue` construction and observations | yes | yes | no | yes |
| `SegQueue::push_mut` / `pop_mut` FIFO transitions | yes | yes | no | yes |
| concurrent runtime implementations | no | no | excluded | no |

## Explicit exclusions and removal conditions

The shared-reference `push`, `pop`, and `force_push` operations, atomic stamp
and block-index protocols, compare-exchange retry loops, memory ordering,
linearization points, progress guarantees, slot initialization, allocation and
reclamation, drops, iterators, and formatting remain outside Creusot
translation. `ArrayQueue` wraparound arithmetic and `SegQueue` segment rollover
are therefore not proved against their runtime bodies.

Removing this boundary requires a Creusot concurrency proof that connects
atomic permissions and synchronization histories to the same sequence model,
plus a separation/ownership model for initialized `MaybeUninit<T>` slots and
the linked block reclamation protocol. The runtime concurrency tests support
the upstream implementation but are not a substitute for those proofs.

Run `./verify-all.bash` in this directory to reproduce the `no_std + alloc`,
default `std`, and all-features proof matrix. Each configuration currently
reports `Proved (14 files)`. The ordinary all-features suite passes 23
integration tests (including the upstream SPSC, MPMC, and linearizability
tests) and 19 documentation tests. Generated Why3 and Cargo build artifacts
are intentionally not tracked.

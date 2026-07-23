# Repository instructions

## Keep verification scoped to the requested crate

Each `<name>/<version>` directory is an independent verification target.
When adding or changing support for a crate, run checks only for the crate named
in the request unless the user explicitly asks for repository-wide validation.

- Run the target crate's `verify-all.bash` when it exists.
- Otherwise run `./verify.bash <name>/<version>` from the repository root.
- Run Cargo commands from the target crate directory or pass its exact
  `--manifest-path`. Do not run workspace-wide or repository-wide Cargo commands.
- Do not loop over other top-level crate directories, and do not use failures from
  other verified crates to judge whether the target crate passes.
- If a command unexpectedly checks or reports another crate, stop using that
  command and rerun the narrow equivalent for the requested crate. Ignore the
  unrelated result; do not fix unrelated crates without an explicit request.
- A shared `CARGO_TARGET_DIR` may reuse or rebuild dependencies. Compiling a shared
  dependency such as `creusot-std` is expected and does not mean that another
  crate's proof or test suite was run.
- Changes under `creusot-libs/` are allowed when the target proof needs a missing
  standard-library contract. Validate the target crate and only the minimum
  directly relevant library build or test; do not reverify every existing crate.
- Keep status reports precise: say which crate, version, and feature configuration
  was checked. Do not imply repository-wide verification from a crate-scoped run.

## Run Why3 proof processes outside the sandbox

Why3/`why3find` proof execution uses Unix-domain socket communication that is
blocked by the filesystem sandbox in this environment. Proof commands must be
requested with unsandboxed/elevated execution on the first attempt rather than
waiting for a sandbox failure.

This applies to commands that invoke the proof phase, including:

- `cargo creusot ... prove`
- `./verify.bash ...`
- a crate-local `verify-all.bash`
- direct `why3find prove` or Why3 server commands

Translation or ordinary Rust build steps that do not start Why3 may remain in the
sandbox. Errors such as `Unix.Unix_error(Unix.EPERM, "bind", ...)`, socket
bind/connect failures, or inability to start the Why3 proof server are environment
failures, not failed proof obligations. Rerun the same target-scoped proof command
outside the sandbox and report the proof result from that run.

## Follow the playbook for mathematical verification

Before verifying a crate with mathematical, iterative, block-based, modular, or
bit-level algorithms, read [`.agents/playbooks/verification.md`](.agents/playbooks/verification.md).
This includes checksums, hashes, cryptography, compression, numeric algorithms,
and nontrivial encodings.

Use the playbook's staged trusted-scaffolding workflow, component boundaries,
proof-status terminology, VC budget, and stop conditions. In particular:

- Prove a small orchestration skeleton before attempting difficult loop or round
  bodies. Temporary trusted boundaries must have strong reviewed contracts and a
  recorded removal condition.
- Give each loop one canonical progress measure. Do not make a large caller
  repeatedly translate between iterator length, an explicit counter, and several
  equivalent arithmetic expressions.
- Do not keep adding assertions or wrapper certificates when a large caller
  repeatedly fails on equivalent indexing, modular-arithmetic, or sequence goals.
  If the same shape fails twice, review the interface or split the caller. If the
  same area fails three times or proof progress moves backward between runs, stop
  and restructure before continuing.
- Treat roughly 100--150 goals in one function, or hundreds of lines of proof
  guidance in one body, as a design warning rather than a challenge to solve by
  increasing prover time or depth.
- Distinguish a proved helper, a proved function body, a trusted contract, and a
  successful crate-level integrated proof. Never report one as another.

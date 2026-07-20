# Verification playbook for mathematical crates

This playbook applies to crates whose behavior is naturally described by
recurrences, modular arithmetic, bit-level transformations, blocks, rounds, or
weighted accumulations. Typical examples are checksums, hashes, cryptographic
primitives, compression routines, numeric algorithms, and encodings.

Its purpose is not merely to make proofs pass. It is to produce proofs that are
small enough to diagnose, stable under unrelated changes, explicit about trusted
boundaries, and economical in both prover time and agent iterations.

## 1. Start with a specification map

Before editing an implementation, record:

1. Every public function and its intended contract.
2. Every public type and its invariant.
3. The mathematical model used by those contracts.
4. The representation relation between runtime values and the model.
5. The intended internal proof boundaries.
6. Any temporary trusted boundaries and the condition for removing them.

For a block-based algorithm, the intended dependency graph should normally look
like this:

```text
public API
  -> input partition / padding
  -> sequence of complete blocks
  -> one-block transformation
  -> one round or step
  -> primitive word and representation lemmas
  -> remainder / finalization
  -> output representation
```

Do not begin by adding assertions to the largest implementation function. First
decide which contracts should let these layers be proved independently.

## 2. Prove the orchestration skeleton first

Use temporary trusted scaffolding to test the proof architecture before investing
in difficult bodies.

1. Extract component functions with strong contracts.
2. Temporarily trust those component bodies.
3. Prove that the top-level function composes their contracts into the public
   result.
4. Once the top-level orchestration is stable, remove trust from one leaf
   component at a time and prove its body independently.

Temporary trust is a proof-development boundary, not permission to weaken the
specification. A temporarily trusted component must have:

- a reviewed functional contract;
- all required range and representation conditions;
- a comment stating why it is trusted;
- a TODO describing exactly what remains to remove trust;
- retained partial proof work when it is useful for later completion.

This order prevents simultaneous debugging of both a difficult component and a
large caller that consumes it.

## 3. Split by proof state, not by runtime size

A loop that runs at most three times can still deserve its own function if it has
its own accumulator, prefix model, overflow argument, or progress measure.

Extract a component when it introduces any of the following:

- a new loop invariant;
- a new mathematical accumulator;
- a separate overflow or range argument;
- a separate prefix, suffix, or subsequence relation;
- a new representation conversion;
- a different modular reduction phase.

For example, a checksum implementation should normally separate:

```text
compute
  -> process_complete_chunks
     -> process_one_chunk
  -> combine_parallel_lanes
  -> process_remainder
  -> reduce_final_state
```

The top-level `compute` proof should mostly connect component postconditions. It
should not contain the detailed loop proof for all four phases.

For a cryptographic primitive, prefer boundaries such as:

```text
public hash / MAC API
  -> padding and block partition
  -> fold over complete blocks
  -> one-block compression
     -> message schedule
     -> one round
     -> repeated rounds
  -> final block
  -> output encoding
```

Prove the mathematical round first, then the runtime word implementation, then
round iteration, block compression, block folding, and finally the public API.

## 4. Use one canonical progress measure per loop

Choose one representation for the loop position and use it everywhere:

- in the loop invariant;
- in component preconditions and postconditions;
- in prefix or suffix indices;
- in accumulator definitions;
- in the termination variant.

Avoid representing the same position simultaneously as:

- the number of visited items;
- the length of the remaining iterator;
- `initial_len - remaining_len`;
- `old_index + 1`;
- a separate runtime counter.

If two representations are unavoidable, prove their relation once in a small
component. Do not re-establish it at the end of every loop iteration.

Prefer contracts whose postconditions already use the updated canonical index.
Post-hoc rewriting from `old_index + 1` to `new_index` looks mathematically
trivial, but it can become unstable when buried in a large VC.

## 5. Separate kinds of reasoning

Avoid mixing all of the following in one VC:

- functional recurrence;
- overflow safety;
- modular congruence;
- iterator position;
- sequence partitioning;
- representation conversion.

Build and prove reusable leaf facts first. Examples include:

```text
wrapping_add(x, y) represents (x + y) mod 2^word_size
rotate_left represents the corresponding bit permutation
byte decoding represents the selected endian word
one round implements the mathematical round function
one block preserves the chaining-state relation
```

The block or round proof should consume these facts without reopening their
low-level arithmetic each time.

## 6. Design lemmas for their callers

A lemma passing in isolation does not prove that its interface is usable from a
large caller. Check both properties early:

1. The lemma body is independently proved.
2. A small representative caller can use its postcondition without relying on
   accidental quantifier instantiation.

When a boolean certificate is appropriate, explicitly connect its result to the
fact needed by the caller:

```rust
#[ensures(result)]
#[ensures(result == target_facts(...))]
fn certificate(...) -> bool { ... }
```

Make the postcondition syntactically close to the consumer's goal. Do not build a
chain of wrappers whose only purpose is to persuade the prover to instantiate a
previously proved quantified lemma.

If the same proved lemma fails to apply in the caller twice:

1. Compare the callee postcondition and caller goal syntactically.
2. Change the component interface so it returns the fact in the caller's
   canonical vocabulary.
3. If that is insufficient, split the caller so lemma application occurs in a
   small VC.
4. Only then consider a different prover strategy or a larger timeout.

## 7. Use a proof budget and stop conditions

The following are design warnings, not rigid correctness limits:

- roughly 100--150 goals generated by one function;
- hundreds of lines of proof guidance inside one implementation body;
- repeated full runs of a function with several hundred goals;
- a helper that passes alone but repeatedly fails to apply in its caller;
- a run reaching a late goal and a later run failing again on an earlier goal.

Apply these stop conditions:

- Same-shaped failure twice: stop adding wrappers and review the interface.
- Same proof area fails three times: stop and split or redesign the component.
- Progress moves backward between equivalent runs: treat the proof as unstable,
  even if a previous run reached a later goal.
- No structural progress for 30--45 minutes: record a checkpoint and report the
  obstruction before continuing.

Increasing timeout, search depth, hammer level, or prover count can be useful for
diagnosis. It is not a substitute for reducing a large context. Prefer explicit
proof structure over a configuration that happens to pass once.

## 8. Run proofs bottom-up

Use this verification order:

1. Primitive arithmetic or sequence lemma.
2. Representative small caller of that lemma.
3. One round, step, lane, or byte update.
4. One block, chunk, or remainder component.
5. Top-level orchestration.
6. Crate-scoped integrated proof.

Do not rerun a 500-goal top-level function after every leaf edit. Save full runs
for component milestones. When possible, preserve the first failing goal and
replay only that target while developing its local proof.

Parallel prover jobs may reduce wall-clock time, but they do not repair an
unstable proof interface or reduce the number of debugging iterations.

## 9. Track proof status precisely

Maintain a status table in the crate README or verification notes while work is
in progress:

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| component example | yes | no | temporary | yes |

Use these terms precisely:

- **Contract reviewed:** the specification was checked for strength and meaning.
- **Body proved:** the implementation body was successfully proved against that
  contract.
- **Trusted:** callers may use the contract, but the body was not proved.
- **Integrated run:** the target crate's configured proof command succeeded with
  the component included.

Do not use “fully proved” for a helper proved in isolation, a trusted function, or
a function that merely appeared in a run which later failed.

At a checkpoint, record:

- the exact component and goal that remain;
- which bodies are independently proved;
- every trusted boundary;
- the last successful integrated command;
- approaches already tried;
- the recommended structural next step.

## 10. Case study: adler2

The adler2 proof exposed several failure modes this playbook is intended to
prevent.

### What happened

- The top-level `compute` proof grew to approximately 540 goals and accumulated
  thousands of lines of proof-oriented code and intermediate assertions.
- The difficult `process_chunk` body and its large caller were developed together
  for too long. A strong temporary trusted boundary was introduced only after
  substantial effort had already been spent.
- The same chunk position was represented using iterator remainder length, an
  explicit `processed_chunks` counter, `processed_before + 1`, and differences
  from the initial chunk count.
- This eventually left goals equivalent to:

  ```text
  remaining = initial[(processed_before + 1)..end]
  processed_chunks = processed_before + 1
  ------------------------------------------------
  remaining = initial[processed_chunks..end]
  ```

- Several certificates proved independently in one or two seconds, yet their
  quantified contracts were not applied reliably inside the huge `compute` VC.
- Additional wrapper certificates changed which goal failed without making the
  proof stable. Runs moved among roughly `526/537`, `527/537`, and later goals,
  sometimes failing again at a previously passed location.
- The final zero-to-three-byte remainder loop was kept inside `compute` because it
  was operationally small. In proof terms it was a separate component: it had its
  own progress index, prefix sums, weighted sum, and overflow bounds.
- Proof status was reported imprecisely at one point: an independently proved
  helper was confused with completion of the corresponding implementation body.

### What should have happened

The efficient development order would have been:

1. Review the strong contracts for `process_chunk`, lane combination, remainder
   processing, and final reduction.
2. Temporarily trust those component bodies.
3. Prove a thin `compute` orchestration using only their contracts.
4. Prove `process_remainder` independently, despite its maximum length of three.
5. Prove lane combination independently.
6. Prove `process_chunk` last, retaining it as an explicit trusted boundary until
   its loop proof is complete.
7. Run the crate-scoped integrated proof after each component milestone.

Once independently proved certificates failed to apply twice in the large
caller, development should have stopped adding certificates and split `compute`.
The correct response was to reduce the VC and align its indices, not to keep
raising prover settings.

Commented-out Rust or proof text does not itself enlarge a VC. It still harms
maintainability when left in a huge body. The proof-performance problem came from
active assertions, snapshots, invariants, and the logical context needed by the
monolithic caller.

## 11. Checklist for a new crate

Before proof implementation begins, answer:

- What are all public contracts and public type invariants?
- What is the mathematical model?
- What is the runtime-to-model relation?
- What are the round, block, remainder, and finalization boundaries?
- Can the top-level function first be proved with those components temporarily
  trusted?
- What single progress measure will each loop use?
- Which facts establish overflow safety separately from functional correctness?
- What is the expected goal count for each component?
- What are the stop conditions for this task?
- How will trusted, independently proved, and integrated status be reported?
- What exact crate-scoped commands will validate the result?

If these questions do not have clear answers, plan the proof architecture before
running the prover.

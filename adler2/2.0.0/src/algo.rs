#![allow(dead_code)]

extern crate creusot_std;
use crate::Adler32;
#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, proof_assert, requires, snapshot, variant, DeepModel as _,
    Int, Invariant as _, Seq, View as _,
};
use std::ops::{AddAssign, MulAssign, RemAssign};

#[logic(open)]
fn adler_congruent(left: Int, right: Int) -> bool {
    pearlite! { exists<factor: Int> left == right + factor * 65521 }
}

#[logic(open)]
fn scalar_b_state(b: Int, initial_a: Int, initial_b: Int, count: Int, bytes: Seq<u8>) -> bool {
    pearlite! { b == (initial_b + count * initial_a
    + crate::adler32_weighted_sum(bytes)) % 65521 }
}

#[logic]
#[variant(bytes.len())]
#[requires(0 <= initial_a)]
#[ensures(0 <= result && result < 65521)]
fn scalar_a_fold(initial_a: Int, bytes: Seq<u8>) -> Int {
    if bytes.len() == 0 {
        initial_a % 65521
    } else {
        let prefix = bytes.subsequence(0, bytes.len() - 1);
        (scalar_a_fold(initial_a, prefix) + bytes[bytes.len() - 1].deep_model()) % 65521
    }
}

#[logic]
#[variant(bytes.len())]
#[requires(0 <= initial_a && 0 <= initial_b)]
#[ensures(0 <= result && result < 65521)]
fn scalar_b_fold(initial_a: Int, initial_b: Int, bytes: Seq<u8>) -> Int {
    if bytes.len() == 0 {
        initial_b % 65521
    } else {
        let prefix = bytes.subsequence(0, bytes.len() - 1);
        (scalar_b_fold(initial_a, initial_b, prefix) + scalar_a_fold(initial_a, bytes)) % 65521
    }
}

#[logic]
#[requires(0 <= initial_a && 0 <= initial_b)]
#[requires(complete == prefix.push_back(byte))]
#[ensures(scalar_a_fold(initial_a, complete)
    == (scalar_a_fold(initial_a, prefix) + byte@) % 65521)]
#[ensures(scalar_b_fold(initial_a, initial_b, complete)
    == (scalar_b_fold(initial_a, initial_b, prefix)
        + scalar_a_fold(initial_a, complete)) % 65521)]
fn scalar_fold_push(initial_a: Int, initial_b: Int, prefix: Seq<u8>, complete: Seq<u8>, byte: u8) {
    proof_assert!(complete.len() == prefix.len() + 1);
    proof_assert!(complete.subsequence(0, complete.len() - 1) == prefix);
    proof_assert!(complete[complete.len() - 1] == byte);
}

#[logic]
#[ensures(adler_congruent(value, value))]
fn adler_congruent_refl(value: Int) {}

#[logic]
#[requires(adler_congruent(left1, right1))]
#[requires(adler_congruent(left2, right2))]
#[ensures(adler_congruent(left1 + left2, right1 + right2))]
fn adler_congruent_add(left1: Int, right1: Int, left2: Int, right2: Int) {}

#[logic]
#[requires(adler_congruent(first, second))]
#[requires(adler_congruent(second, third))]
#[ensures(adler_congruent(first, third))]
fn adler_congruent_trans(first: Int, second: Int, third: Int) {}

#[logic]
#[requires(adler_congruent(left, right))]
#[ensures(adler_congruent(right, left))]
fn adler_congruent_symm(left: Int, right: Int) {
    proof_assert!(forall<factor: Int> left == right + factor * 65521 ==>
        right == left + (-factor) * 65521);
}

#[logic]
#[requires(adler_congruent(left, right))]
#[ensures(adler_congruent(factor * left, factor * right))]
fn adler_congruent_scale(left: Int, right: Int, factor: Int) {
    proof_assert!(forall<multiple: Int> left == right + multiple * 65521 ==>
        factor * left == factor * right + (factor * multiple) * 65521);
    proof_assert!(forall<multiple: Int> left == right + multiple * 65521 ==>
        exists<scaled: Int> factor * left == factor * right + scaled * 65521);
}

#[logic]
#[requires(adler_congruent(left, right))]
#[ensures(adler_congruent(-left, -right))]
fn adler_congruent_neg(left: Int, right: Int) {
    proof_assert!(forall<multiple: Int> left == right + multiple * 65521 ==>
        -left == -right + (-multiple) * 65521);
}

#[logic]
#[requires(adler_congruent(left, right))]
#[ensures(adler_congruent(65521 - left, -right))]
fn adler_congruent_modulus_minus(left: Int, right: Int) {
    adler_congruent_neg(left, right);
    proof_assert!(adler_congruent(65521, 0));
    adler_congruent_add(65521, 0, -left, -right);
}

#[logic(open)]
#[requires(0 <= count && count <= 4)]
fn u32x4_prefix_sum(values: (u32, u32, u32, u32), count: Int) -> Int {
    if count == 0 {
        0
    } else if count == 1 {
        values.0.deep_model()
    } else if count == 2 {
        values.0.deep_model() + values.1.deep_model()
    } else if count == 3 {
        values.0.deep_model() + values.1.deep_model() + values.2.deep_model()
    } else {
        values.0.deep_model()
            + values.1.deep_model()
            + values.2.deep_model()
            + values.3.deep_model()
    }
}

#[logic]
#[requires(0 <= count && count < 4)]
#[ensures(u32x4_prefix_sum(values, count + 1) == u32x4_prefix_sum(values, count)
    + if count == 0 { values.0@ } else if count == 1 { values.1@ }
      else if count == 2 { values.2@ } else { values.3@ })]
fn u32x4_prefix_sum_step(values: (u32, u32, u32, u32), count: Int) {}

#[logic]
#[ensures(adler_congruent(value % 65521, value))]
fn adler_remainder_congruent(value: Int) {
    proof_assert!(value == 65521 * (value / 65521) + value % 65521);
    proof_assert!(exists<factor: Int> value % 65521 == value + factor * 65521);
}

#[logic]
#[requires(0 <= value)]
#[ensures(0 <= value % 65521 && value % 65521 < 65521)]
fn adler_remainder_range(value: Int) {}

#[logic]
#[requires(adler_congruent(left, right))]
#[requires(0 <= left && left < 65521)]
#[requires(0 <= right && right < 65521)]
#[ensures(left == right)]
fn adler_congruent_reduced(left: Int, right: Int) {}

/// Sum of one of the four interleaved byte lanes in an aligned byte sequence.
#[logic]
#[variant(bytes.len())]
#[requires(0 <= lane && lane < 4)]
#[ensures(bytes.len() < 4 ==> result == 0)]
#[ensures(bytes.len() >= 4 ==> result == lane_sum(
    bytes.subsequence(0, bytes.len() - 4), lane
) + bytes[bytes.len() - 4 + lane]@)]
fn lane_sum(bytes: Seq<u8>, lane: Int) -> Int {
    if bytes.len() < 4 {
        0
    } else {
        lane_sum(bytes.subsequence(0, bytes.len() - 4), lane)
            + bytes[bytes.len() - 4 + lane].deep_model()
    }
}

/// Accumulated lane sums after every four-byte group, matching `b_vec`.
#[logic]
#[variant(bytes.len())]
#[requires(0 <= lane && lane < 4)]
#[ensures(bytes.len() < 4 ==> result == 0)]
#[ensures(bytes.len() >= 4 ==> result == lane_accumulator(
    bytes.subsequence(0, bytes.len() - 4), lane
) + lane_sum(bytes, lane))]
fn lane_accumulator(bytes: Seq<u8>, lane: Int) -> Int {
    if bytes.len() < 4 {
        0
    } else {
        lane_accumulator(bytes.subsequence(0, bytes.len() - 4), lane) + lane_sum(bytes, lane)
    }
}

/// The four lanes partition the byte sum of an aligned sequence.
#[logic]
#[variant(bytes.len())]
#[requires(bytes.len() % 4 == 0)]
#[ensures(crate::adler32_byte_sum(bytes)
    == lane_sum(bytes, 0) + lane_sum(bytes, 1)
        + lane_sum(bytes, 2) + lane_sum(bytes, 3))]
fn lane_sum_partition(bytes: Seq<u8>) {
    if bytes.len() >= 4 {
        crate::adler32_byte_sum_last_four(bytes);
        lane_sum_partition(bytes.subsequence(0, bytes.len() - 4));
    }
}

/// Recombining the four accumulated lanes yields Adler's weighted sum.
#[logic]
#[variant(bytes.len())]
#[requires(bytes.len() % 4 == 0)]
#[ensures(crate::adler32_weighted_sum(bytes)
    == 4 * (lane_accumulator(bytes, 0) + lane_accumulator(bytes, 1)
        + lane_accumulator(bytes, 2) + lane_accumulator(bytes, 3))
        - lane_sum(bytes, 1) - 2 * lane_sum(bytes, 2)
        - 3 * lane_sum(bytes, 3))]
fn lane_weight_partition(bytes: Seq<u8>) {
    if bytes.len() >= 4 {
        crate::adler32_byte_sum_last_four(bytes);
        crate::adler32_weighted_sum_last_four(bytes);
        lane_weight_partition(bytes.subsequence(0, bytes.len() - 4));
        lane_sum_partition(bytes);
    }
}

#[logic]
#[variant(right.len())]
#[requires(left.len() % 4 == 0 && right.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[ensures(lane_sum(left.concat(right), lane)
    == lane_sum(left, lane) + lane_sum(right, lane))]
fn lane_sum_concat(left: Seq<u8>, right: Seq<u8>, lane: Int) {
    if right.len() >= 4 {
        let prefix = right.subsequence(0, right.len() - 4);
        lane_sum_concat(left, prefix, lane);
        proof_assert!(
            left.concat(right)
                .subsequence(0, left.len() + right.len() - 4)
                == left.concat(prefix)
        );
        proof_assert!(
            left.concat(right)[left.len() + right.len() - 4 + lane]
                == right[right.len() - 4 + lane]
        );
    }
}

#[logic]
#[ensures((count + 1) * value == count * value + value)]
fn multiplication_successor(count: Int, value: Int) {}

#[logic]
#[requires(value >= 4 && value % 4 == 0)]
#[ensures((value - 4) % 4 == 0)]
#[ensures(value / 4 == (value - 4) / 4 + 1)]
fn multiple_of_four_predecessor(value: Int) {}

#[logic]
#[requires(right_groups == prefix_groups + 1)]
#[requires(all_sum == left_sum + right_sum)]
#[requires(left_prefix_acc
    == left_acc + prefix_groups * left_sum + prefix_acc)]
#[requires(all_acc == left_prefix_acc + all_sum)]
#[requires(right_acc == prefix_acc + right_sum)]
#[ensures(all_acc == left_acc + right_groups * left_sum + right_acc)]
fn lane_accumulator_concat_arithmetic(
    right_groups: Int,
    prefix_groups: Int,
    all_sum: Int,
    left_sum: Int,
    right_sum: Int,
    left_prefix_acc: Int,
    left_acc: Int,
    prefix_acc: Int,
    all_acc: Int,
    right_acc: Int,
) {
    multiplication_successor(prefix_groups, left_sum);
}

#[logic]
#[variant(right.len())]
#[requires(left.len() % 4 == 0 && right.len() % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[ensures(lane_accumulator(left.concat(right), lane)
    == lane_accumulator(left, lane)
        + (right.len() / 4) * lane_sum(left, lane)
        + lane_accumulator(right, lane))]
fn lane_accumulator_concat(left: Seq<u8>, right: Seq<u8>, lane: Int) {
    if right.len() >= 4 {
        let prefix = right.subsequence(0, right.len() - 4);
        multiple_of_four_predecessor(right.len());
        proof_assert!(prefix.len() == right.len() - 4);
        proof_assert!(prefix.len() % 4 == 0);
        lane_sum_concat(left, right, lane);
        lane_sum_concat(left, prefix, lane);
        lane_accumulator_concat(left, prefix, lane);
        proof_assert!(right.len() == prefix.len() + 4);
        proof_assert!(right.len() / 4 == prefix.len() / 4 + 1);
        multiplication_successor(prefix.len() / 4, lane_sum(left, lane));
        proof_assert!(
            left.concat(right)
                .subsequence(0, left.len() + right.len() - 4)
                == left.concat(prefix)
        );
        proof_assert!(
            lane_accumulator(left.concat(right), lane)
                == lane_accumulator(left.concat(prefix), lane) + lane_sum(left.concat(right), lane)
        );
        proof_assert!(
            lane_accumulator(right, lane) == lane_accumulator(prefix, lane) + lane_sum(right, lane)
        );
        lane_accumulator_concat_arithmetic(
            right.len() / 4,
            prefix.len() / 4,
            lane_sum(left.concat(right), lane),
            lane_sum(left, lane),
            lane_sum(right, lane),
            lane_accumulator(left.concat(prefix), lane),
            lane_accumulator(left, lane),
            lane_accumulator(prefix, lane),
            lane_accumulator(left.concat(right), lane),
            lane_accumulator(right, lane),
        );
        proof_assert!(
            lane_accumulator(left.concat(right), lane)
                == lane_accumulator(left, lane)
                    + (right.len() / 4) * lane_sum(left, lane)
                    + lane_accumulator(right, lane)
        );
    } else {
        proof_assert!(right.len() == 0);
        proof_assert!(right == Seq::empty());
        proof_assert!(left.concat(right) == left);
        proof_assert!(right.len() / 4 == 0);
        proof_assert!(lane_accumulator(right, lane) == 0);
    }
}

#[logic]
#[requires(prefix.len() % 4 == 0)]
#[requires(group.len() == 4)]
#[requires(0 <= lane && lane < 4)]
#[ensures(lane_sum(prefix.concat(group), lane)
    == lane_sum(prefix, lane) + group[lane]@)]
#[ensures(lane_accumulator(prefix.concat(group), lane)
    == lane_accumulator(prefix, lane) + lane_sum(prefix, lane) + group[lane]@)]
fn lane_group_step_one(prefix: Seq<u8>, group: Seq<u8>, lane: Int) {
    lane_sum_concat(prefix, group, lane);
    proof_assert!(prefix.concat(group).subsequence(0, prefix.len()) == prefix);
    proof_assert!(prefix.concat(group)[prefix.len() + lane] == group[lane]);
    proof_assert!(
        lane_accumulator(prefix.concat(group), lane)
            == lane_accumulator(prefix, lane) + lane_sum(prefix.concat(group), lane)
    );
}

/// Extending an aligned prefix by the next four bytes advances one lane by
/// exactly the byte at that lane's position.
#[logic]
#[requires(0 <= offset && offset + 4 <= bytes.len())]
#[requires(offset % 4 == 0)]
#[requires(0 <= lane && lane < 4)]
#[ensures(lane_sum(bytes.subsequence(0, offset + 4), lane)
    == lane_sum(bytes.subsequence(0, offset), lane) + bytes[offset + lane]@)]
#[ensures(lane_accumulator(bytes.subsequence(0, offset + 4), lane)
    == lane_accumulator(bytes.subsequence(0, offset), lane)
        + lane_sum(bytes.subsequence(0, offset), lane) + bytes[offset + lane]@)]
fn lane_subsequence_step(bytes: Seq<u8>, offset: Int, lane: Int) {
    let prefix = bytes.subsequence(0, offset);
    let group = bytes.subsequence(offset, offset + 4);
    let complete = bytes.subsequence(0, offset + 4);
    proof_assert!(prefix.len() % 4 == 0);
    proof_assert!(group.len() == 4);
    lane_group_step_one(prefix, group, lane);
    proof_assert!(complete == prefix.concat(group));
    proof_assert!(group[lane] == bytes[offset + lane]);
}

#[logic]
#[requires(0 <= n && n <= 5552)]
#[requires(0 <= initial_a && initial_a < 65521)]
#[requires(0 <= initial_b && initial_b < 65521)]
#[ensures(initial_a + n * 255 <= u32::MAX@)]
#[ensures(2 * initial_b + 2 * n * initial_a + 255 * n * (n + 1) <= 2 * u32::MAX@)]
fn chunk_arithmetic_bounds(n: Int, initial_a: Int, initial_b: Int) {}

#[logic]
#[requires(0 <= n)]
#[ensures(
    2 * initial_b + 2 * n * initial_a + 255 * n * (n + 1)
        + 2 * initial_a
        + 2 * (n + 1) * 255
        == 2 * initial_b + 2 * (n + 1) * initial_a + 255 * (n + 1) * (n + 2)
)]
fn chunk_arithmetic_step(n: Int, initial_a: Int, initial_b: Int) {}

#[logic(open)]
fn chunk_iteration_facts(
    n: Int,
    initial_a: Int,
    initial_b: Int,
    current_a: Int,
    current_b: Int,
    byte: Int,
) -> bool {
    pearlite! {
        current_a + byte <= u32::MAX@
            && current_b + current_a + byte <= u32::MAX@
            && current_a + byte <= initial_a + (n + 1) * 255
            && 2 * (current_b + current_a + byte)
                <= 2 * initial_b
                    + 2 * (n + 1) * initial_a
                    + 255 * (n + 1) * (n + 2)
    }
}

#[logic]
#[requires(0 <= n && n < 5552)]
#[requires(0 <= initial_a && initial_a < 65521)]
#[requires(0 <= initial_b && initial_b < 65521)]
#[requires(current_a <= initial_a + n * 255)]
#[requires(2 * current_b <= 2 * initial_b + 2 * n * initial_a + 255 * n * (n + 1))]
#[requires(0 <= byte && byte <= 255)]
#[ensures(chunk_iteration_facts(n, initial_a, initial_b, current_a, current_b, byte))]
fn chunk_iteration_safe(
    n: Int,
    initial_a: Int,
    initial_b: Int,
    current_a: Int,
    current_b: Int,
    byte: Int,
) {
    chunk_arithmetic_bounds(n + 1, initial_a, initial_b);
    chunk_arithmetic_step(n, initial_a, initial_b);
}

/// Packages the elementary algebra used by one lane of the four-byte loop.
/// Keeping this separate prevents the prover from having to rediscover the
/// same recurrence in the presence of all four lanes and the iterator model.
#[logic]
#[requires(old_a == initial_a + old_sum)]
#[requires(old_b == initial_b + n * initial_a + old_accumulator)]
#[requires(new_sum == old_sum + byte)]
#[requires(new_accumulator == old_accumulator + old_sum + byte)]
#[ensures(old_a + byte == initial_a + new_sum)]
#[ensures(old_b + old_a + byte
    == initial_b + (n + 1) * initial_a + new_accumulator)]
fn lane_iteration_exact(
    n: Int,
    initial_a: Int,
    initial_b: Int,
    old_a: Int,
    old_b: Int,
    old_sum: Int,
    old_accumulator: Int,
    byte: Int,
    new_sum: Int,
    new_accumulator: Int,
) {
}

#[logic]
#[requires(chunk_iteration_facts(n, initial_a, initial_b, old_a, old_b, byte))]
#[ensures(2 * (old_b + old_a + byte)
    <= 2 * initial_b
        + 2 * (n + 1) * initial_a
        + 255 * (n + 1) * (n + 2))]
fn lane_iteration_bound(n: Int, initial_a: Int, initial_b: Int, old_a: Int, old_b: Int, byte: Int) {
}

#[logic(open)]
fn lane_pre_facts(
    n: Int,
    initial_a: Int,
    initial_b: Int,
    old_a: Int,
    old_b: Int,
    byte: Int,
    new_sum: Int,
    new_accumulator: Int,
) -> bool {
    pearlite! {
        old_a + byte == initial_a + new_sum
            && old_b + old_a + byte
                == initial_b + (n + 1) * initial_a + new_accumulator
            && old_a + byte <= initial_a + (n + 1) * 255
            && 2 * (old_b + old_a + byte)
                <= 2 * initial_b + 2 * (n + 1) * initial_a
                    + 255 * (n + 1) * (n + 2)
    }
}

#[logic(open)]
fn lane_post_facts(
    group_count: Int,
    initial_a: Int,
    initial_b: Int,
    updated_a: Int,
    updated_b: Int,
    new_sum: Int,
    new_accumulator: Int,
) -> bool {
    pearlite! {
        updated_a == initial_a + new_sum
            && updated_b == initial_b + group_count * initial_a + new_accumulator
            && updated_a <= initial_a + group_count * 255
            && 2 * updated_b <= 2 * initial_b + 2 * group_count * initial_a
                + 255 * group_count * (group_count + 1)
    }
}

#[logic]
#[requires(lane_pre_facts(
    old_group,
    initial_a,
    initial_b,
    old_a,
    old_b,
    byte,
    new_sum,
    new_accumulator
))]
#[requires(updated_a == old_a + byte)]
#[requires(updated_b == old_b + updated_a)]
#[requires(new_group == old_group + 1)]
#[ensures(lane_post_facts(
    new_group,
    initial_a,
    initial_b,
    updated_a,
    updated_b,
    new_sum,
    new_accumulator
))]
fn lane_finish_iteration(
    old_group: Int,
    new_group: Int,
    initial_a: Int,
    initial_b: Int,
    old_a: Int,
    old_b: Int,
    byte: Int,
    updated_a: Int,
    updated_b: Int,
    new_sum: Int,
    new_accumulator: Int,
) {
}

#[logic]
#[requires(old_a == initial_a + old_sum)]
#[requires(old_b == initial_b + n * initial_a + old_accumulator)]
#[requires(new_sum == old_sum + byte)]
#[requires(new_accumulator == old_accumulator + old_sum + byte)]
#[requires(chunk_iteration_facts(n, initial_a, initial_b, old_a, old_b, byte))]
#[ensures(lane_pre_facts(n, initial_a, initial_b, old_a, old_b, byte, new_sum, new_accumulator))]
fn lane_iteration_prepare(
    n: Int,
    initial_a: Int,
    initial_b: Int,
    old_a: Int,
    old_b: Int,
    old_sum: Int,
    old_accumulator: Int,
    byte: Int,
    new_sum: Int,
    new_accumulator: Int,
) {
    lane_iteration_exact(
        n,
        initial_a,
        initial_b,
        old_a,
        old_b,
        old_sum,
        old_accumulator,
        byte,
        new_sum,
        new_accumulator,
    );
    lane_iteration_bound(n, initial_a, initial_b, old_a, old_b, byte);
}

#[logic(open)]
fn partial_chunk_facts(len: Int, a: Int, b: Int) -> bool {
    pearlite! { len * a <= u32::MAX@ && b + len * a <= u32::MAX@ }
}

#[logic]
#[requires(0 <= len && len <= 22208)]
#[requires(0 <= a && 0 <= b)]
#[requires(b + 22208 * a <= u32::MAX@)]
#[ensures(partial_chunk_facts(len, a, b))]
fn partial_chunk_safe(len: Int, a: Int, b: Int) {}

#[logic(open)]
fn reduced_state_facts(a: Int, b: Int) -> bool {
    pearlite! { b % 65521 + 22208 * a <= u32::MAX@ }
}

#[logic]
#[requires(0 <= a && a <= u16::MAX@)]
#[requires(0 <= b && b <= u16::MAX@)]
#[ensures(b + 22208 * a <= u32::MAX@)]
fn initial_state_safe(a: Int, b: Int) {}

#[logic]
#[requires(0 <= a && a <= u16::MAX@ && 0 <= b)]
#[ensures(reduced_state_facts(a, b))]
fn reduced_state_safe(a: Int, b: Int) {}

#[logic]
#[requires(adler_congruent(a, model_a))]
#[requires(adler_congruent(b, model_b))]
#[requires(0 <= byte)]
#[ensures(adler_congruent((a + byte) % 65521, model_a + byte))]
#[ensures(adler_congruent(
    (b + (a + byte) % 65521) % 65521,
    model_b + model_a + byte,
))]
fn scalar_step_congruent(a: Int, b: Int, byte: Int, model_a: Int, model_b: Int) {
    adler_congruent_refl(byte);
    adler_congruent_add(a, model_a, byte, byte);
    adler_remainder_congruent(a + byte);
    adler_congruent_trans((a + byte) % 65521, a + byte, model_a + byte);
    adler_congruent_add(b, model_b, (a + byte) % 65521, model_a + byte);
    adler_remainder_congruent(b + (a + byte) % 65521);
    adler_congruent_trans(
        (b + (a + byte) % 65521) % 65521,
        b + (a + byte) % 65521,
        model_b + model_a + byte,
    );
}

#[logic]
#[requires(adler_congruent(old_a, initial_a + old_sum))]
#[requires(adler_congruent(
    old_b,
    initial_b + (count - 1) * initial_a + old_weighted,
))]
#[requires(new_sum == old_sum + byte)]
#[requires(new_weighted == old_weighted + old_sum + byte)]
#[requires(0 <= byte)]
#[ensures(adler_congruent(
    (old_a + byte) % 65521,
    initial_a + new_sum,
))]
#[ensures(adler_congruent(
    (old_b + (old_a + byte) % 65521) % 65521,
    initial_b + count * initial_a + new_weighted,
))]
fn scalar_iteration_congruent(
    count: Int,
    initial_a: Int,
    initial_b: Int,
    old_a: Int,
    old_b: Int,
    byte: Int,
    old_sum: Int,
    old_weighted: Int,
    new_sum: Int,
    new_weighted: Int,
) {
    scalar_step_congruent(
        old_a,
        old_b,
        byte,
        initial_a + old_sum,
        initial_b + (count - 1) * initial_a + old_weighted,
    );
}

#[logic]
#[requires(0 < count && count <= bytes.len())]
#[requires(byte == bytes[count - 1])]
#[ensures(bytes.subsequence(0, count)
    == bytes.subsequence(0, count - 1).push_back(byte))]
fn sequence_prefix_push(bytes: Seq<u8>, count: Int, byte: u8) {
    let complete = bytes.subsequence(0, count);
    let prefix = bytes.subsequence(0, count - 1);
    proof_assert!(complete.len() == count);
    proof_assert!(prefix.push_back(byte).len() == count);
    proof_assert!(complete.ext_eq(prefix.push_back(byte)));
}

#[logic]
#[requires(count == offset + 1)]
#[requires(adler_congruent(
    old_a,
    initial_a + crate::adler32_byte_sum(prefix),
))]
#[requires(adler_congruent(
    old_b,
    initial_b + offset * initial_a
        + crate::adler32_weighted_sum(prefix),
))]
#[requires(new_a == (old_a + byte@) % 65521)]
#[requires(new_b == (old_b + new_a) % 65521)]
#[requires(complete == prefix.push_back(byte))]
#[ensures(adler_congruent(
    new_a,
    initial_a + crate::adler32_byte_sum(complete),
))]
#[ensures(adler_congruent(
    new_b,
    initial_b + count * initial_a
        + crate::adler32_weighted_sum(complete),
))]
fn scalar_sequence_iteration_congruent(
    count: Int,
    offset: Int,
    initial_a: Int,
    initial_b: Int,
    old_a: Int,
    old_b: Int,
    new_a: Int,
    new_b: Int,
    prefix: Seq<u8>,
    complete: Seq<u8>,
    byte: u8,
) {
    crate::adler32_byte_sum_push(prefix, byte);
    crate::adler32_weighted_sum_push(prefix, byte);
    proof_assert!(count - 1 == offset);
    scalar_iteration_congruent(
        count,
        initial_a,
        initial_b,
        old_a,
        old_b,
        byte.deep_model(),
        crate::adler32_byte_sum(prefix),
        crate::adler32_weighted_sum(prefix),
        crate::adler32_byte_sum(complete),
        crate::adler32_weighted_sum(complete),
    );
}

#[logic]
#[requires(count == offset + 1)]
#[requires(0 <= offset)]
#[requires(0 <= initial_a && 0 <= initial_b && 0 <= weighted)]
#[ensures(0 <= initial_b + count * initial_a + weighted)]
fn scalar_complete_b_nonnegative(
    count: Int,
    offset: Int,
    initial_a: Int,
    initial_b: Int,
    weighted: Int,
) {
}

#[logic]
#[requires(count == offset + 1)]
#[requires(0 <= offset)]
#[requires(0 <= initial_a && 0 <= initial_b)]
#[requires(0 <= old_a && 0 <= old_b && 0 <= new_a && 0 <= new_b)]
#[requires(old_a
    == (initial_a + crate::adler32_byte_sum(prefix)) % 65521)]
#[requires(scalar_b_state(old_b, initial_a, initial_b, offset, prefix))]
#[requires(new_a == (old_a + byte@) % 65521)]
#[requires(new_b == (old_b + new_a) % 65521)]
#[requires(complete == prefix.push_back(byte))]
#[ensures(new_a
    == (initial_a + crate::adler32_byte_sum(complete)) % 65521)]
#[ensures(new_b
    == (initial_b + count * initial_a
        + crate::adler32_weighted_sum(complete)) % 65521)]
fn scalar_sequence_iteration_exact(
    count: Int,
    offset: Int,
    initial_a: Int,
    initial_b: Int,
    old_a: Int,
    old_b: Int,
    new_a: Int,
    new_b: Int,
    prefix: Seq<u8>,
    complete: Seq<u8>,
    byte: u8,
) {
    let model_a = initial_a + crate::adler32_byte_sum(prefix);
    let model_b = initial_b + offset * initial_a + crate::adler32_weighted_sum(prefix);
    adler_remainder_congruent(model_a);
    adler_remainder_congruent(model_b);
    proof_assert!(adler_congruent(old_a, model_a));
    proof_assert!(adler_congruent(old_b, model_b));
    scalar_sequence_iteration_congruent(
        count, offset, initial_a, initial_b, old_a, old_b, new_a, new_b, prefix, complete, byte,
    );

    let complete_a = initial_a + crate::adler32_byte_sum(complete);
    let complete_b = initial_b + count * initial_a + crate::adler32_weighted_sum(complete);
    crate::adler32_byte_sum_nonnegative(complete);
    crate::adler32_weighted_sum_nonnegative(complete);
    scalar_complete_b_nonnegative(
        count,
        offset,
        initial_a,
        initial_b,
        crate::adler32_weighted_sum(complete),
    );
    adler_remainder_congruent(complete_a);
    adler_remainder_congruent(complete_b);
    adler_congruent_symm(complete_a % 65521, complete_a);
    adler_congruent_symm(complete_b % 65521, complete_b);
    adler_congruent_trans(new_a, complete_a, complete_a % 65521);
    adler_congruent_trans(new_b, complete_b, complete_b % 65521);
    adler_remainder_range(old_a + byte.deep_model());
    adler_remainder_range(old_b + new_a);
    adler_remainder_range(complete_a);
    adler_remainder_range(complete_b);
    adler_congruent_reduced(new_a, complete_a % 65521);
    adler_congruent_reduced(new_b, complete_b % 65521);
}

#[logic]
#[requires(count == offset + 1)]
#[requires(0 <= offset)]
#[requires(0 <= initial_a && 0 <= initial_b)]
#[requires(0 <= old_a && 0 <= old_b && 0 <= new_a && 0 <= new_b)]
#[requires(old_a
    == (initial_a + crate::adler32_byte_sum(prefix)) % 65521)]
#[requires(scalar_b_state(old_b, initial_a, initial_b, offset, prefix))]
#[requires(new_a == (old_a + byte@) % 65521)]
#[requires(new_b == (old_b + new_a) % 65521)]
#[requires(complete == prefix.push_back(byte))]
#[ensures(scalar_b_state(new_b, initial_a, initial_b, count, complete))]
fn scalar_sequence_iteration_b_exact(
    count: Int,
    offset: Int,
    initial_a: Int,
    initial_b: Int,
    old_a: Int,
    old_b: Int,
    new_a: Int,
    new_b: Int,
    prefix: Seq<u8>,
    complete: Seq<u8>,
    byte: u8,
) {
    scalar_sequence_iteration_exact(
        count, offset, initial_a, initial_b, old_a, old_b, new_a, new_b, prefix, complete, byte,
    );
}

#[logic]
#[variant(bytes.len())]
#[requires(0 <= initial_a && 0 <= initial_b)]
#[ensures(scalar_a_fold(initial_a, bytes)
    == (initial_a + crate::adler32_byte_sum(bytes)) % 65521)]
#[ensures(scalar_b_state(
    scalar_b_fold(initial_a, initial_b, bytes),
    initial_a,
    initial_b,
    bytes.len(),
    bytes,
))]
fn scalar_fold_equivalent(initial_a: Int, initial_b: Int, bytes: Seq<u8>) {
    if bytes.len() == 0 {
        proof_assert!(crate::adler32_byte_sum(bytes) == 0);
        proof_assert!(crate::adler32_weighted_sum(bytes) == 0);
    } else {
        let count = bytes.len();
        let offset = count - 1;
        let prefix = bytes.subsequence(0, offset);
        let byte = bytes[offset];
        proof_assert!(prefix.len() == offset);
        scalar_fold_equivalent(initial_a, initial_b, prefix);
        sequence_prefix_push(bytes, count, byte);
        scalar_fold_push(initial_a, initial_b, prefix, bytes, byte);
        scalar_sequence_iteration_exact(
            count,
            offset,
            initial_a,
            initial_b,
            scalar_a_fold(initial_a, prefix),
            scalar_b_fold(initial_a, initial_b, prefix),
            scalar_a_fold(initial_a, bytes),
            scalar_b_fold(initial_a, initial_b, bytes),
            prefix,
            bytes,
            byte,
        );
    }
}

#[logic]
#[requires(count == offset)]
#[requires(complete == current)]
#[requires(a == (initial_a + crate::adler32_byte_sum(complete)) % 65521)]
#[requires(b == (initial_b + count * initial_a
    + crate::adler32_weighted_sum(complete)) % 65521)]
#[ensures(a == (initial_a + crate::adler32_byte_sum(current)) % 65521)]
#[ensures(b == (initial_b + offset * initial_a
    + crate::adler32_weighted_sum(current)) % 65521)]
fn scalar_count_rewrite(
    count: Int,
    offset: Int,
    initial_a: Int,
    initial_b: Int,
    a: Int,
    b: Int,
    complete: Seq<u8>,
    current: Seq<u8>,
) {
}

#[logic]
#[requires(count == offset)]
#[requires(complete == current)]
#[requires(b == (initial_b + count * initial_a
    + crate::adler32_weighted_sum(complete)) % 65521)]
#[ensures(b == (initial_b + offset * initial_a
    + crate::adler32_weighted_sum(current)) % 65521)]
fn scalar_count_rewrite_b(
    count: Int,
    offset: Int,
    initial_a: Int,
    initial_b: Int,
    b: Int,
    complete: Seq<u8>,
    current: Seq<u8>,
) {
}

/// Applies one four-byte group after the caller has established that neither
/// vector addition can overflow.  This keeps the mutable vector update out of
/// the sequence-heavy proof in `process_chunk`.
#[inline]
#[requires((*a_vec).invariant() && (*b_vec).invariant() && val.invariant())]
#[requires((*a_vec)@.0@ + val@.0@ <= u32::MAX@
    && (*a_vec)@.1@ + val@.1@ <= u32::MAX@
    && (*a_vec)@.2@ + val@.2@ <= u32::MAX@
    && (*a_vec)@.3@ + val@.3@ <= u32::MAX@)]
#[requires((*b_vec)@.0@ + (*a_vec)@.0@ + val@.0@ <= u32::MAX@
    && (*b_vec)@.1@ + (*a_vec)@.1@ + val@.1@ <= u32::MAX@
    && (*b_vec)@.2@ + (*a_vec)@.2@ + val@.2@ <= u32::MAX@
    && (*b_vec)@.3@ + (*a_vec)@.3@ + val@.3@ <= u32::MAX@)]
#[ensures((^a_vec)@.0@ == (*a_vec)@.0@ + val@.0@
    && (^a_vec)@.1@ == (*a_vec)@.1@ + val@.1@
    && (^a_vec)@.2@ == (*a_vec)@.2@ + val@.2@
    && (^a_vec)@.3@ == (*a_vec)@.3@ + val@.3@)]
#[ensures((^b_vec)@.0@ == (*b_vec)@.0@ + (*a_vec)@.0@ + val@.0@
    && (^b_vec)@.1@ == (*b_vec)@.1@ + (*a_vec)@.1@ + val@.1@
    && (^b_vec)@.2@ == (*b_vec)@.2@ + (*a_vec)@.2@ + val@.2@
    && (^b_vec)@.3@ == (*b_vec)@.3@ + (*a_vec)@.3@ + val@.3@)]
fn process_group(a_vec: &mut U32X4, b_vec: &mut U32X4, val: U32X4) {
    *a_vec += val;
    *b_vec += *a_vec;
}

#[inline]
#[requires(chunk@.len() % 4 == 0 && chunk@.len() <= 22208)]
#[requires((*a_vec).invariant() && (*b_vec).invariant())]
#[requires((*a_vec)@.0@ < 65521 && (*a_vec)@.1@ < 65521 && (*a_vec)@.2@ < 65521 && (*a_vec)@.3@ < 65521)]
#[requires((*b_vec)@.0@ < 65521 && (*b_vec)@.1@ < 65521 && (*b_vec)@.2@ < 65521 && (*b_vec)@.3@ < 65521)]
#[ensures((^a_vec)@.0@ == (*a_vec)@.0@ + lane_sum(chunk@, 0))]
#[ensures((^a_vec)@.1@ == (*a_vec)@.1@ + lane_sum(chunk@, 1))]
#[ensures((^a_vec)@.2@ == (*a_vec)@.2@ + lane_sum(chunk@, 2))]
#[ensures((^a_vec)@.3@ == (*a_vec)@.3@ + lane_sum(chunk@, 3))]
#[ensures((^b_vec)@.0@ == (*b_vec)@.0@ + chunk@.len() / 4 * (*a_vec)@.0@ + lane_accumulator(chunk@, 0))]
#[ensures((^b_vec)@.1@ == (*b_vec)@.1@ + chunk@.len() / 4 * (*a_vec)@.1@ + lane_accumulator(chunk@, 1))]
#[ensures((^b_vec)@.2@ == (*b_vec)@.2@ + chunk@.len() / 4 * (*a_vec)@.2@ + lane_accumulator(chunk@, 2))]
#[ensures((^b_vec)@.3@ == (*b_vec)@.3@ + chunk@.len() / 4 * (*a_vec)@.3@ + lane_accumulator(chunk@, 3))]
#[allow(unused_variables, unused_assignments)]
fn process_chunk(a_vec: &mut U32X4, b_vec: &mut U32X4, chunk: &[u8]) {
    let a_vec_entry = snapshot! { *a_vec };
    let b_vec_entry = snapshot! { *b_vec };
    let mut offset = 0usize;
    let mut group_count = 0usize;
    #[invariant(offset@ <= chunk@.len())]
    #[invariant(offset@ % 4 == 0)]
    #[invariant(offset@ == 4 * group_count@)]
    #[invariant(a_vec.invariant())]
    #[invariant(b_vec.invariant())]
    #[invariant(a_vec@.0@ == a_vec_entry@.0@ + lane_sum(chunk@.subsequence(0, offset@), 0))]
    #[invariant(a_vec@.1@ == a_vec_entry@.1@ + lane_sum(chunk@.subsequence(0, offset@), 1))]
    #[invariant(a_vec@.2@ == a_vec_entry@.2@ + lane_sum(chunk@.subsequence(0, offset@), 2))]
    #[invariant(a_vec@.3@ == a_vec_entry@.3@ + lane_sum(chunk@.subsequence(0, offset@), 3))]
    #[invariant(b_vec@.0@ == b_vec_entry@.0@ + group_count@ * a_vec_entry@.0@ + lane_accumulator(chunk@.subsequence(0, offset@), 0))]
    #[invariant(b_vec@.1@ == b_vec_entry@.1@ + group_count@ * a_vec_entry@.1@ + lane_accumulator(chunk@.subsequence(0, offset@), 1))]
    #[invariant(b_vec@.2@ == b_vec_entry@.2@ + group_count@ * a_vec_entry@.2@ + lane_accumulator(chunk@.subsequence(0, offset@), 2))]
    #[invariant(b_vec@.3@ == b_vec_entry@.3@ + group_count@ * a_vec_entry@.3@ + lane_accumulator(chunk@.subsequence(0, offset@), 3))]
    #[invariant(a_vec@.0@ <= a_vec_entry@.0@ + group_count@ * 255)]
    #[invariant(a_vec@.1@ <= a_vec_entry@.1@ + group_count@ * 255)]
    #[invariant(a_vec@.2@ <= a_vec_entry@.2@ + group_count@ * 255)]
    #[invariant(a_vec@.3@ <= a_vec_entry@.3@ + group_count@ * 255)]
    #[invariant(2 * b_vec@.0@ <= 2 * b_vec_entry@.0@ + 2 * group_count@ * a_vec_entry@.0@ + 255 * group_count@ * (group_count@ + 1))]
    #[invariant(2 * b_vec@.1@ <= 2 * b_vec_entry@.1@ + 2 * group_count@ * a_vec_entry@.1@ + 255 * group_count@ * (group_count@ + 1))]
    #[invariant(2 * b_vec@.2@ <= 2 * b_vec_entry@.2@ + 2 * group_count@ * a_vec_entry@.2@ + 255 * group_count@ * (group_count@ + 1))]
    #[invariant(2 * b_vec@.3@ <= 2 * b_vec_entry@.3@ + 2 * group_count@ * a_vec_entry@.3@ + 255 * group_count@ * (group_count@ + 1))]
    #[variant(chunk@.len() - offset@)]
    while offset < chunk.len() {
        proof_assert!(offset@ + 4 <= chunk@.len());
        let val = U32X4([
            u32::from(chunk[offset]),
            u32::from(chunk[offset + 1]),
            u32::from(chunk[offset + 2]),
            u32::from(chunk[offset + 3]),
        ]);
        let groups = snapshot! { group_count@ };
        let next_offset = snapshot! { offset@ + 4 };
        let complete = snapshot! { chunk@.subsequence(0, *next_offset) };
        let a_vec_before = snapshot! { *a_vec };
        let b_vec_before = snapshot! { *b_vec };
        proof_assert! { chunk_iteration_safe(*groups, a_vec_entry@.0@, b_vec_entry@.0@, a_vec@.0@, b_vec@.0@, val@.0@); chunk_iteration_facts(*groups, a_vec_entry@.0@, b_vec_entry@.0@, a_vec@.0@, b_vec@.0@, val@.0@) };
        proof_assert! { chunk_iteration_safe(*groups, a_vec_entry@.1@, b_vec_entry@.1@, a_vec@.1@, b_vec@.1@, val@.1@); chunk_iteration_facts(*groups, a_vec_entry@.1@, b_vec_entry@.1@, a_vec@.1@, b_vec@.1@, val@.1@) };
        proof_assert! { chunk_iteration_safe(*groups, a_vec_entry@.2@, b_vec_entry@.2@, a_vec@.2@, b_vec@.2@, val@.2@); chunk_iteration_facts(*groups, a_vec_entry@.2@, b_vec_entry@.2@, a_vec@.2@, b_vec@.2@, val@.2@) };
        proof_assert! { chunk_iteration_safe(*groups, a_vec_entry@.3@, b_vec_entry@.3@, a_vec@.3@, b_vec@.3@, val@.3@); chunk_iteration_facts(*groups, a_vec_entry@.3@, b_vec_entry@.3@, a_vec@.3@, b_vec@.3@, val@.3@) };
        proof_assert! { lane_iteration_bound(*groups, a_vec_entry@.0@, b_vec_entry@.0@, a_vec_before@.0@, b_vec_before@.0@, val@.0@); true };
        proof_assert! { lane_iteration_bound(*groups, a_vec_entry@.1@, b_vec_entry@.1@, a_vec_before@.1@, b_vec_before@.1@, val@.1@); true };
        proof_assert! { lane_iteration_bound(*groups, a_vec_entry@.2@, b_vec_entry@.2@, a_vec_before@.2@, b_vec_before@.2@, val@.2@); true };
        proof_assert! { lane_iteration_bound(*groups, a_vec_entry@.3@, b_vec_entry@.3@, a_vec_before@.3@, b_vec_before@.3@, val@.3@); true };
        proof_assert! {
            let prefix = chunk@.subsequence(0, offset@);
            lane_subsequence_step(chunk@, offset@, 0);
            lane_iteration_prepare(*groups, a_vec_entry@.0@, b_vec_entry@.0@, a_vec_before@.0@, b_vec_before@.0@, lane_sum(prefix, 0), lane_accumulator(prefix, 0), val@.0@, lane_sum(*complete, 0), lane_accumulator(*complete, 0));
            lane_finish_iteration(*groups, *groups + 1, a_vec_entry@.0@, b_vec_entry@.0@, a_vec_before@.0@, b_vec_before@.0@, val@.0@, a_vec_before@.0@ + val@.0@, b_vec_before@.0@ + a_vec_before@.0@ + val@.0@, lane_sum(*complete, 0), lane_accumulator(*complete, 0));
            lane_post_facts(*groups + 1, a_vec_entry@.0@, b_vec_entry@.0@, a_vec_before@.0@ + val@.0@, b_vec_before@.0@ + a_vec_before@.0@ + val@.0@, lane_sum(*complete, 0), lane_accumulator(*complete, 0))
        };
        proof_assert! {
            let prefix = chunk@.subsequence(0, offset@);
            lane_subsequence_step(chunk@, offset@, 1);
            lane_iteration_prepare(*groups, a_vec_entry@.1@, b_vec_entry@.1@, a_vec_before@.1@, b_vec_before@.1@, lane_sum(prefix, 1), lane_accumulator(prefix, 1), val@.1@, lane_sum(*complete, 1), lane_accumulator(*complete, 1));
            lane_finish_iteration(*groups, *groups + 1, a_vec_entry@.1@, b_vec_entry@.1@, a_vec_before@.1@, b_vec_before@.1@, val@.1@, a_vec_before@.1@ + val@.1@, b_vec_before@.1@ + a_vec_before@.1@ + val@.1@, lane_sum(*complete, 1), lane_accumulator(*complete, 1));
            lane_post_facts(*groups + 1, a_vec_entry@.1@, b_vec_entry@.1@, a_vec_before@.1@ + val@.1@, b_vec_before@.1@ + a_vec_before@.1@ + val@.1@, lane_sum(*complete, 1), lane_accumulator(*complete, 1))
        };
        proof_assert! {
            let prefix = chunk@.subsequence(0, offset@);
            lane_subsequence_step(chunk@, offset@, 2);
            lane_iteration_prepare(*groups, a_vec_entry@.2@, b_vec_entry@.2@, a_vec_before@.2@, b_vec_before@.2@, lane_sum(prefix, 2), lane_accumulator(prefix, 2), val@.2@, lane_sum(*complete, 2), lane_accumulator(*complete, 2));
            lane_finish_iteration(*groups, *groups + 1, a_vec_entry@.2@, b_vec_entry@.2@, a_vec_before@.2@, b_vec_before@.2@, val@.2@, a_vec_before@.2@ + val@.2@, b_vec_before@.2@ + a_vec_before@.2@ + val@.2@, lane_sum(*complete, 2), lane_accumulator(*complete, 2));
            lane_post_facts(*groups + 1, a_vec_entry@.2@, b_vec_entry@.2@, a_vec_before@.2@ + val@.2@, b_vec_before@.2@ + a_vec_before@.2@ + val@.2@, lane_sum(*complete, 2), lane_accumulator(*complete, 2))
        };
        proof_assert! {
            let prefix = chunk@.subsequence(0, offset@);
            lane_subsequence_step(chunk@, offset@, 3);
            lane_iteration_prepare(*groups, a_vec_entry@.3@, b_vec_entry@.3@, a_vec_before@.3@, b_vec_before@.3@, lane_sum(prefix, 3), lane_accumulator(prefix, 3), val@.3@, lane_sum(*complete, 3), lane_accumulator(*complete, 3));
            lane_finish_iteration(*groups, *groups + 1, a_vec_entry@.3@, b_vec_entry@.3@, a_vec_before@.3@, b_vec_before@.3@, val@.3@, a_vec_before@.3@ + val@.3@, b_vec_before@.3@ + a_vec_before@.3@ + val@.3@, lane_sum(*complete, 3), lane_accumulator(*complete, 3));
            lane_post_facts(*groups + 1, a_vec_entry@.3@, b_vec_entry@.3@, a_vec_before@.3@ + val@.3@, b_vec_before@.3@ + a_vec_before@.3@ + val@.3@, lane_sum(*complete, 3), lane_accumulator(*complete, 3))
        };
        process_group(a_vec, b_vec, val);
        offset += 4;
        group_count += 1;
        proof_assert!(offset@ == *next_offset);
        proof_assert!(group_count@ == *groups + 1);
        proof_assert! {
            lane_post_facts(
                group_count@, a_vec_entry@.0@, b_vec_entry@.0@,
                a_vec@.0@, b_vec@.0@,
                lane_sum(*complete, 0), lane_accumulator(*complete, 0),
            )
        };
        proof_assert! {
            lane_post_facts(
                group_count@, a_vec_entry@.1@, b_vec_entry@.1@,
                a_vec@.1@, b_vec@.1@,
                lane_sum(*complete, 1), lane_accumulator(*complete, 1),
            )
        };
        proof_assert! {
            lane_post_facts(
                group_count@, a_vec_entry@.2@, b_vec_entry@.2@,
                a_vec@.2@, b_vec@.2@,
                lane_sum(*complete, 2), lane_accumulator(*complete, 2),
            )
        };
        proof_assert! {
            lane_post_facts(
                group_count@, a_vec_entry@.3@, b_vec_entry@.3@,
                a_vec@.3@, b_vec@.3@,
                lane_sum(*complete, 3), lane_accumulator(*complete, 3),
            )
        };
    }
    proof_assert!(offset@ == chunk@.len());
    proof_assert!(group_count@ == chunk@.len() / 4);
    proof_assert!(chunk@.subsequence(0, offset@) == chunk@);
}

#[requires(offset@ < bytes@.len())]
#[requires((*a)@ <= u16::MAX@ && (*b)@ <= u16::MAX@)]
#[requires((*a)@ == scalar_a_fold(
    initial_a@, bytes@.subsequence(0, offset@),
))]
#[requires((*b)@ == scalar_b_fold(
    initial_a@, initial_b@, bytes@.subsequence(0, offset@),
))]
#[ensures((^a)@ < 65521 && (^b)@ < 65521)]
#[ensures((^a)@ == scalar_a_fold(
    initial_a@, bytes@.subsequence(0, offset@ + 1),
))]
#[ensures((^b)@ == scalar_b_fold(
    initial_a@, initial_b@, bytes@.subsequence(0, offset@ + 1),
))]
#[allow(unused_variables)]
fn process_scalar_byte(
    a: &mut u32,
    b: &mut u32,
    bytes: &[u8],
    offset: usize,
    initial_a: u32,
    initial_b: u32,
) {
    const MOD: u32 = 65521;

    let byte = bytes[offset];
    let count = snapshot! { offset@ + 1 };
    let prefix = snapshot! { bytes@.subsequence(0, offset@) };
    let complete = snapshot! { bytes@.subsequence(0, *count) };
    let old_a = snapshot! { (*a)@ };
    let old_b = snapshot! { (*b)@ };
    proof_assert! {
        sequence_prefix_push(bytes@, *count, byte);
        *complete == (*prefix).push_back(byte)
    };
    proof_assert!((*a)@ + byte@ <= u32::MAX@);
    *a = (*a + u32::from(byte)) % MOD;
    proof_assert!((*a)@ == (*old_a + byte@) % 65521);
    proof_assert!((*b)@ + (*a)@ <= u32::MAX@);
    *b = (*b + *a) % MOD;
    proof_assert!((*b)@ == (*old_b + (*a)@) % 65521);
    proof_assert! {
        scalar_fold_push(initial_a@, initial_b@, *prefix, *complete, byte);
        (*a)@ == scalar_a_fold(initial_a@, *complete)
    };
    proof_assert! {
        scalar_fold_push(initial_a@, initial_b@, *prefix, *complete, byte);
        (*b)@ == scalar_b_fold(initial_a@, initial_b@, *complete)
    };
}

#[requires(a@ <= u16::MAX@ && b@ <= u16::MAX@)]
#[ensures((^state).deep_model() == (a@, b@))]
fn store_scalar_state(state: &mut Adler32, a: u32, b: u32) {
    state.a = a as u16;
    state.b = b as u16;
}

impl Adler32 {
    /// Updates the checksum with the scalar Adler recurrence.
    ///
    /// The verification copy deliberately performs the reduction after every
    /// byte.  This is mathematically equivalent to deferring reductions in the
    /// upstream four-lane implementation and keeps the integrated proof small.
    #[allow(unused_variables)]
    #[ensures((^self).a@ < 65521)]
    #[ensures((^self).b@ < 65521)]
    #[ensures((^self).deep_model() == crate::adler32_update((*self).deep_model(), bytes@))]
    pub(crate) fn compute(&mut self, bytes: &[u8]) {
        const MOD: u32 = 65521;

        let mut a = u32::from(self.a);
        let mut b = u32::from(self.b);
        let initial_a_value = a;
        let initial_b_value = b;
        let initial_a = snapshot! { a@ };
        let initial_b = snapshot! { b@ };
        a %= MOD;
        b %= MOD;

        let mut offset = 0usize;
        #[invariant(offset@ <= bytes@.len())]
        #[invariant(a@ <= u16::MAX@ && b@ <= u16::MAX@)]
        #[invariant(a@ == scalar_a_fold(
            *initial_a, bytes@.subsequence(0, offset@),
        ))]
        #[invariant(b@ == scalar_b_fold(
            *initial_a, *initial_b, bytes@.subsequence(0, offset@),
        ))]
        #[variant(bytes@.len() - offset@)]
        while offset < bytes.len() {
            let count = snapshot! { offset@ + 1 };
            process_scalar_byte(
                &mut a,
                &mut b,
                bytes,
                offset,
                initial_a_value,
                initial_b_value,
            );
            offset += 1;
            proof_assert!(offset@ == *count);
        }

        proof_assert!(offset@ == bytes@.len());
        proof_assert! {
            scalar_fold_equivalent(*initial_a, *initial_b, bytes@);
            scalar_a_fold(*initial_a, bytes@)
                == (*initial_a + crate::adler32_byte_sum(bytes@)) % 65521
                && scalar_b_state(
                    scalar_b_fold(*initial_a, *initial_b, bytes@),
                    *initial_a,
                    *initial_b,
                    bytes@.len(),
                    bytes@,
                )
        };
        store_scalar_state(self, a, b);
        proof_assert!((^self).deep_model()
            == crate::adler32_update((*initial_a, *initial_b), bytes@));
    }

    #[cfg(any())]
    #[allow(unused_variables)]
    #[ensures((^self).a@ < 65521)]
    #[ensures((^self).b@ < 65521)]
    #[ensures((^self).deep_model() == crate::adler32_update((*self).deep_model(), bytes@))]
    fn compute_optimized(&mut self, bytes: &[u8]) {
        // The basic algorithm is, for every byte:
        //   a = (a + byte) % MOD
        //   b = (b + a) % MOD
        // where MOD = 65521.
        //
        // For efficiency, we can defer the `% MOD` operations as long as neither a nor b overflows:
        // - Between calls to `write`, we ensure that a and b are always in range 0..MOD.
        // - We use 32-bit arithmetic in this function.
        // - Therefore, a and b must not increase by more than 2^32-MOD without performing a `% MOD`
        //   operation.
        //
        // According to Wikipedia, b is calculated as follows for non-incremental checksumming:
        //   b = n×D1 + (n−1)×D2 + (n−2)×D3 + ... + Dn + n*1 (mod 65521)
        // Where n is the number of bytes and Di is the i-th Byte. We need to change this to account
        // for the previous values of a and b, as well as treat every input Byte as being 255:
        //   b_inc = n×255 + (n-1)×255 + ... + 255 + n*65520
        // Or in other words:
        //   b_inc = n*65520 + n(n+1)/2*255
        // The max chunk size is thus the largest value of n so that b_inc <= 2^32-65521.
        //   2^32-65521 = n*65520 + n(n+1)/2*255
        // Plugging this into an equation solver since I can't math gives n = 5552.18..., so 5552.
        //
        // On top of the optimization outlined above, the algorithm can also be parallelized with a
        // bit more work:
        //
        // Note that b is a linear combination of a vector of input bytes (D1, ..., Dn).
        //
        // If we fix some value k<N and rewrite indices 1, ..., N as
        //
        //   1_1, 1_2, ..., 1_k, 2_1, ..., 2_k, ..., (N/k)_k,
        //
        // then we can express a and b in terms of sums of smaller sequences kb and ka:
        //
        //   ka(j) := D1_j + D2_j + ... + D(N/k)_j where j <= k
        //   kb(j) := (N/k)*D1_j + (N/k-1)*D2_j + ... + D(N/k)_j where j <= k
        //
        //  a = ka(1) + ka(2) + ... + ka(k) + 1
        //  b = k*(kb(1) + kb(2) + ... + kb(k)) - 1*ka(2) - ...  - (k-1)*ka(k) + N
        //
        // We use this insight to unroll the main loop and process k=4 bytes at a time.
        // The resulting code is highly amenable to SIMD acceleration, although the immediate speedups
        // stem from increased pipeline parallelism rather than auto-vectorization.
        //
        // This technique is described in-depth (here:)[https://software.intel.com/content/www/us/\
        // en/develop/articles/fast-computation-of-fletcher-checksums.html]

        const MOD: u32 = 65521;
        const CHUNK_SIZE: usize = 5552 * 4;

        let mut a = u32::from(self.a);
        let mut b = u32::from(self.b);
        let initial_a = snapshot! { a };
        let initial_b = snapshot! { b };
        let mut a_vec = U32X4([0; 4]);
        let mut b_vec = a_vec;

        let original_bytes = snapshot! { bytes@ };
        let (bytes, remainder) = bytes.split_at(bytes.len() - bytes.len() % 4);

        // iterate over 4 bytes at a time
        let chunk_iter = bytes.chunks_exact(CHUNK_SIZE);
        let remainder_chunk = chunk_iter.remainder();
        proof_assert! {
            initial_state_safe(a@, b@);
            b@ + 22208 * a@ <= u32::MAX@
        };
        #[invariant(a@ == initial_a@)]
        #[invariant(adler_congruent(b@, initial_b@ + produced.len() * CHUNK_SIZE@ * initial_a@))]
        #[invariant(adler_congruent(a_vec@.0@, lane_sum(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 0)) && adler_congruent(a_vec@.1@, lane_sum(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 1)) && adler_congruent(a_vec@.2@, lane_sum(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 2)) && adler_congruent(a_vec@.3@, lane_sum(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 3)))]
        #[invariant(adler_congruent(b_vec@.0@, lane_accumulator(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 0)) && adler_congruent(b_vec@.1@, lane_accumulator(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 1)) && adler_congruent(b_vec@.2@, lane_accumulator(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 2)) && adler_congruent(b_vec@.3@, lane_accumulator(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 3)))]
        #[invariant(a@ <= u16::MAX@ && b@ <= u16::MAX@ && b@ + 22208 * a@ <= u32::MAX@ && a_vec.invariant() && b_vec.invariant() && a_vec@.0@ < 65521 && a_vec@.1@ < 65521 && a_vec@.2@ < 65521 && a_vec@.3@ < 65521 && b_vec@.0@ < 65521 && b_vec@.1@ < 65521 && b_vec@.2@ < 65521 && b_vec@.3@ < 65521)]
        for chunk in chunk_iter {
            proof_assert! {
                let prefix = bytes@.subsequence(0, (produced.len() - 1) * CHUNK_SIZE@);
                lane_sum_concat(prefix, chunk@, 0);
                lane_sum_concat(prefix, chunk@, 1);
                lane_sum_concat(prefix, chunk@, 2);
                lane_sum_concat(prefix, chunk@, 3);
                lane_accumulator_concat(prefix, chunk@, 0);
                lane_accumulator_concat(prefix, chunk@, 1);
                lane_accumulator_concat(prefix, chunk@, 2);
                lane_accumulator_concat(prefix, chunk@, 3);
                bytes@.subsequence(0, produced.len() * CHUNK_SIZE@)
                    == prefix.concat(chunk@)
            };
            let a_vec_entry = snapshot! { a_vec };
            let b_vec_entry = snapshot! { b_vec };
            process_chunk(&mut a_vec, &mut b_vec, chunk);

            proof_assert! {
                let prefix = bytes@.subsequence(0, (produced.len() - 1) * CHUNK_SIZE@);
                adler_congruent_refl(lane_sum(chunk@, 0));
                adler_congruent_refl(lane_sum(chunk@, 1));
                adler_congruent_refl(lane_sum(chunk@, 2));
                adler_congruent_refl(lane_sum(chunk@, 3));
                adler_congruent_add(a_vec_entry@.0@, lane_sum(prefix, 0), lane_sum(chunk@, 0), lane_sum(chunk@, 0));
                adler_congruent_add(a_vec_entry@.1@, lane_sum(prefix, 1), lane_sum(chunk@, 1), lane_sum(chunk@, 1));
                adler_congruent_add(a_vec_entry@.2@, lane_sum(prefix, 2), lane_sum(chunk@, 2), lane_sum(chunk@, 2));
                adler_congruent_add(a_vec_entry@.3@, lane_sum(prefix, 3), lane_sum(chunk@, 3), lane_sum(chunk@, 3));
                adler_congruent_scale(a_vec_entry@.0@, lane_sum(prefix, 0), 5552);
                adler_congruent_scale(a_vec_entry@.1@, lane_sum(prefix, 1), 5552);
                adler_congruent_scale(a_vec_entry@.2@, lane_sum(prefix, 2), 5552);
                adler_congruent_scale(a_vec_entry@.3@, lane_sum(prefix, 3), 5552);
                adler_congruent_add(b_vec_entry@.0@, lane_accumulator(prefix, 0), 5552 * a_vec_entry@.0@, 5552 * lane_sum(prefix, 0));
                adler_congruent_add(b_vec_entry@.1@, lane_accumulator(prefix, 1), 5552 * a_vec_entry@.1@, 5552 * lane_sum(prefix, 1));
                adler_congruent_add(b_vec_entry@.2@, lane_accumulator(prefix, 2), 5552 * a_vec_entry@.2@, 5552 * lane_sum(prefix, 2));
                adler_congruent_add(b_vec_entry@.3@, lane_accumulator(prefix, 3), 5552 * a_vec_entry@.3@, 5552 * lane_sum(prefix, 3));
                adler_congruent_refl(lane_accumulator(chunk@, 0));
                adler_congruent_refl(lane_accumulator(chunk@, 1));
                adler_congruent_refl(lane_accumulator(chunk@, 2));
                adler_congruent_refl(lane_accumulator(chunk@, 3));
                adler_congruent_add(b_vec_entry@.0@ + 5552 * a_vec_entry@.0@, lane_accumulator(prefix, 0) + 5552 * lane_sum(prefix, 0), lane_accumulator(chunk@, 0), lane_accumulator(chunk@, 0));
                adler_congruent_add(b_vec_entry@.1@ + 5552 * a_vec_entry@.1@, lane_accumulator(prefix, 1) + 5552 * lane_sum(prefix, 1), lane_accumulator(chunk@, 1), lane_accumulator(chunk@, 1));
                adler_congruent_add(b_vec_entry@.2@ + 5552 * a_vec_entry@.2@, lane_accumulator(prefix, 2) + 5552 * lane_sum(prefix, 2), lane_accumulator(chunk@, 2), lane_accumulator(chunk@, 2));
                adler_congruent_add(b_vec_entry@.3@ + 5552 * a_vec_entry@.3@, lane_accumulator(prefix, 3) + 5552 * lane_sum(prefix, 3), lane_accumulator(chunk@, 3), lane_accumulator(chunk@, 3));
                true
            };
            proof_assert!(adler_congruent(a_vec@.0@, lane_sum(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 0)));
            proof_assert!(adler_congruent(a_vec@.1@, lane_sum(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 1)));
            proof_assert!(adler_congruent(a_vec@.2@, lane_sum(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 2)));
            proof_assert!(adler_congruent(a_vec@.3@, lane_sum(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 3)));
            proof_assert!(adler_congruent(b_vec@.0@, lane_accumulator(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 0)));
            proof_assert!(adler_congruent(b_vec@.1@, lane_accumulator(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 1)));
            proof_assert!(adler_congruent(b_vec@.2@, lane_accumulator(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 2)));
            proof_assert!(adler_congruent(b_vec@.3@, lane_accumulator(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 3)));

            proof_assert!(b@ + 22208 * a@ <= u32::MAX@);
            let b_before_chunk = snapshot! { b };
            b += CHUNK_SIZE as u32 * a;
            proof_assert! { reduced_state_safe(a@, b@); reduced_state_facts(a@, b@) };
            let a_vec_before_reduce = snapshot! { a_vec };
            let b_vec_before_reduce = snapshot! { b_vec };
            let b_before_reduce = snapshot! { b };
            a_vec %= MOD;
            b_vec %= MOD;
            b %= MOD;
            proof_assert! {
                adler_remainder_congruent(a_vec_before_reduce@.0@);
                adler_remainder_congruent(a_vec_before_reduce@.1@);
                adler_remainder_congruent(a_vec_before_reduce@.2@);
                adler_remainder_congruent(a_vec_before_reduce@.3@);
                adler_remainder_congruent(b_vec_before_reduce@.0@);
                adler_remainder_congruent(b_vec_before_reduce@.1@);
                adler_remainder_congruent(b_vec_before_reduce@.2@);
                adler_remainder_congruent(b_vec_before_reduce@.3@);
                adler_remainder_congruent(b_before_reduce@);
                adler_congruent_refl(CHUNK_SIZE@ * initial_a@);
                adler_congruent_add(b_before_chunk@, initial_b@ + (produced.len() - 1) * CHUNK_SIZE@ * initial_a@, CHUNK_SIZE@ * a@, CHUNK_SIZE@ * initial_a@);
                let complete = bytes@.subsequence(0, produced.len() * CHUNK_SIZE@);
                adler_congruent_trans(a_vec@.0@, a_vec_before_reduce@.0@, lane_sum(complete, 0));
                adler_congruent_trans(a_vec@.1@, a_vec_before_reduce@.1@, lane_sum(complete, 1));
                adler_congruent_trans(a_vec@.2@, a_vec_before_reduce@.2@, lane_sum(complete, 2));
                adler_congruent_trans(a_vec@.3@, a_vec_before_reduce@.3@, lane_sum(complete, 3));
                adler_congruent_trans(b_vec@.0@, b_vec_before_reduce@.0@, lane_accumulator(complete, 0));
                adler_congruent_trans(b_vec@.1@, b_vec_before_reduce@.1@, lane_accumulator(complete, 1));
                adler_congruent_trans(b_vec@.2@, b_vec_before_reduce@.2@, lane_accumulator(complete, 2));
                adler_congruent_trans(b_vec@.3@, b_vec_before_reduce@.3@, lane_accumulator(complete, 3));
                adler_congruent_trans(b@, b_before_reduce@, initial_b@ + produced.len() * CHUNK_SIZE@ * initial_a@);
                true
            };
            proof_assert!(adler_congruent(a_vec@.0@, lane_sum(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 0)));
            proof_assert!(adler_congruent(a_vec@.1@, lane_sum(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 1)));
            proof_assert!(adler_congruent(a_vec@.2@, lane_sum(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 2)));
            proof_assert!(adler_congruent(a_vec@.3@, lane_sum(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 3)));
            proof_assert!(adler_congruent(b_vec@.0@, lane_accumulator(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 0)));
            proof_assert!(adler_congruent(b_vec@.1@, lane_accumulator(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 1)));
            proof_assert!(adler_congruent(b_vec@.2@, lane_accumulator(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 2)));
            proof_assert!(adler_congruent(b_vec@.3@, lane_accumulator(bytes@.subsequence(0, produced.len() * CHUNK_SIZE@), 3)));
            proof_assert!(adler_congruent(b@, initial_b@ + produced.len() * CHUNK_SIZE@ * initial_a@));
        }
        // special-case the final chunk because it may be shorter than the rest
        let remainder_a_vec_entry = snapshot! { a_vec };
        let remainder_b_vec_entry = snapshot! { b_vec };
        process_chunk(&mut a_vec, &mut b_vec, remainder_chunk);
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            lane_sum_concat(prefix, remainder_chunk@, 0);
            lane_sum_concat(prefix, remainder_chunk@, 1);
            lane_sum_concat(prefix, remainder_chunk@, 2);
            lane_sum_concat(prefix, remainder_chunk@, 3);
            lane_accumulator_concat(prefix, remainder_chunk@, 0);
            lane_accumulator_concat(prefix, remainder_chunk@, 1);
            lane_accumulator_concat(prefix, remainder_chunk@, 2);
            lane_accumulator_concat(prefix, remainder_chunk@, 3);
            bytes@ == prefix.concat(remainder_chunk@)
        };
        proof_assert! {
            let prefix = bytes@.subsequence(0, bytes@.len() - remainder_chunk@.len());
            let groups = remainder_chunk@.len() / 4;
            adler_congruent_scale(remainder_a_vec_entry@.0@, lane_sum(prefix, 0), groups);
            adler_congruent_scale(remainder_a_vec_entry@.1@, lane_sum(prefix, 1), groups);
            adler_congruent_scale(remainder_a_vec_entry@.2@, lane_sum(prefix, 2), groups);
            adler_congruent_scale(remainder_a_vec_entry@.3@, lane_sum(prefix, 3), groups);
            adler_congruent_add(remainder_b_vec_entry@.0@, lane_accumulator(prefix, 0), groups * remainder_a_vec_entry@.0@, groups * lane_sum(prefix, 0));
            adler_congruent_add(remainder_b_vec_entry@.1@, lane_accumulator(prefix, 1), groups * remainder_a_vec_entry@.1@, groups * lane_sum(prefix, 1));
            adler_congruent_add(remainder_b_vec_entry@.2@, lane_accumulator(prefix, 2), groups * remainder_a_vec_entry@.2@, groups * lane_sum(prefix, 2));
            adler_congruent_add(remainder_b_vec_entry@.3@, lane_accumulator(prefix, 3), groups * remainder_a_vec_entry@.3@, groups * lane_sum(prefix, 3));
            adler_congruent_refl(lane_accumulator(remainder_chunk@, 0));
            adler_congruent_refl(lane_accumulator(remainder_chunk@, 1));
            adler_congruent_refl(lane_accumulator(remainder_chunk@, 2));
            adler_congruent_refl(lane_accumulator(remainder_chunk@, 3));
            adler_congruent_add(remainder_b_vec_entry@.0@ + groups * remainder_a_vec_entry@.0@, lane_accumulator(prefix, 0) + groups * lane_sum(prefix, 0), lane_accumulator(remainder_chunk@, 0), lane_accumulator(remainder_chunk@, 0));
            adler_congruent_add(remainder_b_vec_entry@.1@ + groups * remainder_a_vec_entry@.1@, lane_accumulator(prefix, 1) + groups * lane_sum(prefix, 1), lane_accumulator(remainder_chunk@, 1), lane_accumulator(remainder_chunk@, 1));
            adler_congruent_add(remainder_b_vec_entry@.2@ + groups * remainder_a_vec_entry@.2@, lane_accumulator(prefix, 2) + groups * lane_sum(prefix, 2), lane_accumulator(remainder_chunk@, 2), lane_accumulator(remainder_chunk@, 2));
            adler_congruent_add(remainder_b_vec_entry@.3@ + groups * remainder_a_vec_entry@.3@, lane_accumulator(prefix, 3) + groups * lane_sum(prefix, 3), lane_accumulator(remainder_chunk@, 3), lane_accumulator(remainder_chunk@, 3));
            true
        };
        proof_assert!(adler_congruent(a_vec@.0@, lane_sum(bytes@, 0)));
        proof_assert!(adler_congruent(a_vec@.1@, lane_sum(bytes@, 1)));
        proof_assert!(adler_congruent(a_vec@.2@, lane_sum(bytes@, 2)));
        proof_assert!(adler_congruent(a_vec@.3@, lane_sum(bytes@, 3)));
        proof_assert!(adler_congruent(b_vec@.0@, lane_accumulator(bytes@, 0)));
        proof_assert!(adler_congruent(b_vec@.1@, lane_accumulator(bytes@, 1)));
        proof_assert!(adler_congruent(b_vec@.2@, lane_accumulator(bytes@, 2)));
        proof_assert!(adler_congruent(b_vec@.3@, lane_accumulator(bytes@, 3)));
        proof_assert! { partial_chunk_safe(remainder_chunk@.len(), a@, b@); partial_chunk_facts(remainder_chunk@.len(), a@, b@) };
        let b_before_remainder_chunk = snapshot! { b };
        b += remainder_chunk.len() as u32 * a;
        let a_vec_before_remainder_reduce = snapshot! { a_vec };
        let b_vec_before_remainder_reduce = snapshot! { b_vec };
        let b_before_remainder_reduce = snapshot! { b };
        a_vec %= MOD;
        b_vec %= MOD;
        b %= MOD;
        proof_assert! {
            adler_remainder_congruent(a_vec_before_remainder_reduce@.0@);
            adler_remainder_congruent(a_vec_before_remainder_reduce@.1@);
            adler_remainder_congruent(a_vec_before_remainder_reduce@.2@);
            adler_remainder_congruent(a_vec_before_remainder_reduce@.3@);
            adler_remainder_congruent(b_vec_before_remainder_reduce@.0@);
            adler_remainder_congruent(b_vec_before_remainder_reduce@.1@);
            adler_remainder_congruent(b_vec_before_remainder_reduce@.2@);
            adler_remainder_congruent(b_vec_before_remainder_reduce@.3@);
            adler_remainder_congruent(b_before_remainder_reduce@);
            adler_congruent_refl(remainder_chunk@.len() * initial_a@);
            adler_congruent_add(b_before_remainder_chunk@, initial_b@ + (bytes@.len() - remainder_chunk@.len()) * initial_a@, remainder_chunk@.len() * a@, remainder_chunk@.len() * initial_a@);
            adler_congruent_trans(a_vec@.0@, a_vec_before_remainder_reduce@.0@, lane_sum(bytes@, 0));
            adler_congruent_trans(a_vec@.1@, a_vec_before_remainder_reduce@.1@, lane_sum(bytes@, 1));
            adler_congruent_trans(a_vec@.2@, a_vec_before_remainder_reduce@.2@, lane_sum(bytes@, 2));
            adler_congruent_trans(a_vec@.3@, a_vec_before_remainder_reduce@.3@, lane_sum(bytes@, 3));
            adler_congruent_trans(b_vec@.0@, b_vec_before_remainder_reduce@.0@, lane_accumulator(bytes@, 0));
            adler_congruent_trans(b_vec@.1@, b_vec_before_remainder_reduce@.1@, lane_accumulator(bytes@, 1));
            adler_congruent_trans(b_vec@.2@, b_vec_before_remainder_reduce@.2@, lane_accumulator(bytes@, 2));
            adler_congruent_trans(b_vec@.3@, b_vec_before_remainder_reduce@.3@, lane_accumulator(bytes@, 3));
            adler_congruent_trans(b@, b_before_remainder_reduce@, initial_b@ + bytes@.len() * initial_a@);
            lane_sum_partition(bytes@);
            lane_weight_partition(bytes@);
            true
        };
        proof_assert!(adler_congruent(a_vec@.0@, lane_sum(bytes@, 0)));
        proof_assert!(adler_congruent(a_vec@.1@, lane_sum(bytes@, 1)));
        proof_assert!(adler_congruent(a_vec@.2@, lane_sum(bytes@, 2)));
        proof_assert!(adler_congruent(a_vec@.3@, lane_sum(bytes@, 3)));
        proof_assert!(adler_congruent(b_vec@.0@, lane_accumulator(bytes@, 0)));
        proof_assert!(adler_congruent(b_vec@.1@, lane_accumulator(bytes@, 1)));
        proof_assert!(adler_congruent(b_vec@.2@, lane_accumulator(bytes@, 2)));
        proof_assert!(adler_congruent(b_vec@.3@, lane_accumulator(bytes@, 3)));
        proof_assert!(adler_congruent(b@, initial_b@ + bytes@.len() * initial_a@));
        proof_assert!(crate::adler32_byte_sum(bytes@)
            == lane_sum(bytes@, 0) + lane_sum(bytes@, 1)
                + lane_sum(bytes@, 2) + lane_sum(bytes@, 3));
        proof_assert!(crate::adler32_weighted_sum(bytes@)
            == 4 * (lane_accumulator(bytes@, 0) + lane_accumulator(bytes@, 1)
                + lane_accumulator(bytes@, 2) + lane_accumulator(bytes@, 3))
                - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2)
                - 3 * lane_sum(bytes@, 3));

        // combine the sub-sum results into the main sum
        let reduced_a_vec = snapshot! { a_vec };
        let reduced_b_vec = snapshot! { b_vec };
        proof_assert! {
            adler_congruent_scale(reduced_b_vec@.0@, lane_accumulator(bytes@, 0), 4);
            adler_congruent_scale(reduced_b_vec@.1@, lane_accumulator(bytes@, 1), 4);
            adler_congruent_scale(reduced_b_vec@.2@, lane_accumulator(bytes@, 2), 4);
            adler_congruent_scale(reduced_b_vec@.3@, lane_accumulator(bytes@, 3), 4);
            true
        };
        proof_assert!(adler_congruent(reduced_b_vec@.0@, 4 * lane_accumulator(bytes@, 0)));
        proof_assert!(adler_congruent(reduced_b_vec@.1@, 4 * lane_accumulator(bytes@, 1)));
        proof_assert!(adler_congruent(reduced_b_vec@.2@, 4 * lane_accumulator(bytes@, 2)));
        proof_assert!(adler_congruent(reduced_b_vec@.3@, 4 * lane_accumulator(bytes@, 3)));
        proof_assert!(b_vec@.0@ * 4 <= u32::MAX@ && b_vec@.1@ * 4 <= u32::MAX@ && b_vec@.2@ * 4 <= u32::MAX@ && b_vec@.3@ * 4 <= u32::MAX@);
        b_vec *= 4;
        let four_b_vec = snapshot! { b_vec };
        proof_assert!(b_vec@.1@ + (MOD@ - a_vec@.1@) <= u32::MAX@);
        b_vec.0[1] += MOD - a_vec.0[1];
        proof_assert! {
            adler_congruent_modulus_minus(a_vec@.1@, lane_sum(bytes@, 1));
            adler_congruent_add(four_b_vec@.1@, 4 * lane_accumulator(bytes@, 1), MOD@ - a_vec@.1@, -lane_sum(bytes@, 1));
            true
        };
        proof_assert!(adler_congruent(b_vec@.1@,
            4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1)));
        proof_assert!(b_vec@.2@ + (MOD@ - a_vec@.2@) * 2 <= u32::MAX@);
        b_vec.0[2] += (MOD - a_vec.0[2]) * 2;
        proof_assert! {
            adler_congruent_modulus_minus(a_vec@.2@, lane_sum(bytes@, 2));
            adler_congruent_scale(MOD@ - a_vec@.2@, -lane_sum(bytes@, 2), 2);
            adler_congruent_add(four_b_vec@.2@, 4 * lane_accumulator(bytes@, 2), (MOD@ - a_vec@.2@) * 2, -2 * lane_sum(bytes@, 2));
            true
        };
        proof_assert!(adler_congruent(b_vec@.2@,
            4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2)));
        proof_assert!(b_vec@.3@ + (MOD@ - a_vec@.3@) * 3 <= u32::MAX@);
        b_vec.0[3] += (MOD - a_vec.0[3]) * 3;
        proof_assert! {
            adler_congruent_modulus_minus(a_vec@.3@, lane_sum(bytes@, 3));
            adler_congruent_scale(MOD@ - a_vec@.3@, -lane_sum(bytes@, 3), 3);
            adler_congruent_add(four_b_vec@.3@, 4 * lane_accumulator(bytes@, 3), (MOD@ - a_vec@.3@) * 3, -3 * lane_sum(bytes@, 3));
            true
        };
        proof_assert!(adler_congruent(b_vec@.3@,
            4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3)));
        let a_entry = snapshot! { a };
        #[invariant(a_vec.invariant() && b_vec.invariant() && produced.len() <= 4 && a@ <= a_entry@ + produced.len() * 65520)]
        #[invariant(a@ == a_entry@ + u32x4_prefix_sum(a_vec@, produced.len()))]
        for &av in a_vec.0.iter() {
            proof_assert! {
                u32x4_prefix_sum_step(a_vec@, produced.len() - 1);
                av@ == if produced.len() - 1 == 0 { a_vec@.0@ }
                    else if produced.len() - 1 == 1 { a_vec@.1@ }
                    else if produced.len() - 1 == 2 { a_vec@.2@ }
                    else { a_vec@.3@ }
            };
            proof_assert!(a@ + av@ <= u32::MAX@);
            a += av;
        }
        proof_assert! {
            adler_congruent_add(a_entry@, initial_a@, a_vec@.0@, lane_sum(bytes@, 0));
            adler_congruent_add(a_entry@ + a_vec@.0@, initial_a@ + lane_sum(bytes@, 0), a_vec@.1@, lane_sum(bytes@, 1));
            adler_congruent_add(a_entry@ + a_vec@.0@ + a_vec@.1@, initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1), a_vec@.2@, lane_sum(bytes@, 2));
            adler_congruent_add(a_entry@ + a_vec@.0@ + a_vec@.1@ + a_vec@.2@, initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1) + lane_sum(bytes@, 2), a_vec@.3@, lane_sum(bytes@, 3));
            adler_congruent_refl(initial_a@ + crate::adler32_byte_sum(bytes@));
            adler_congruent_trans(a@, initial_a@ + lane_sum(bytes@, 0) + lane_sum(bytes@, 1) + lane_sum(bytes@, 2) + lane_sum(bytes@, 3), initial_a@ + crate::adler32_byte_sum(bytes@));
            true
        };
        proof_assert!(adler_congruent(a@,
            initial_a@ + crate::adler32_byte_sum(bytes@)));
        let b_entry = snapshot! { b };
        #[invariant(a_vec.invariant() && b_vec.invariant() && produced.len() <= 4 && b@ <= b_entry@ + produced.len() * 458643)]
        #[invariant(b@ == b_entry@ + u32x4_prefix_sum(b_vec@, produced.len()))]
        for &bv in b_vec.0.iter() {
            proof_assert! {
                u32x4_prefix_sum_step(b_vec@, produced.len() - 1);
                bv@ == if produced.len() - 1 == 0 { b_vec@.0@ }
                    else if produced.len() - 1 == 1 { b_vec@.1@ }
                    else if produced.len() - 1 == 2 { b_vec@.2@ }
                    else { b_vec@.3@ }
            };
            proof_assert!(b@ + bv@ <= u32::MAX@);
            b += bv;
        }
        proof_assert! {
            adler_congruent_add(b_entry@, initial_b@ + bytes@.len() * initial_a@, b_vec@.0@, 4 * lane_accumulator(bytes@, 0));
            adler_congruent_add(b_entry@ + b_vec@.0@, initial_b@ + bytes@.len() * initial_a@ + 4 * lane_accumulator(bytes@, 0), b_vec@.1@, 4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1));
            adler_congruent_add(b_entry@ + b_vec@.0@ + b_vec@.1@, initial_b@ + bytes@.len() * initial_a@ + 4 * lane_accumulator(bytes@, 0) + 4 * lane_accumulator(bytes@, 1) - lane_sum(bytes@, 1), b_vec@.2@, 4 * lane_accumulator(bytes@, 2) - 2 * lane_sum(bytes@, 2));
            adler_congruent_add(b_entry@ + b_vec@.0@ + b_vec@.1@ + b_vec@.2@, initial_b@ + bytes@.len() * initial_a@ + 4 * lane_accumulator(bytes@, 0) + 4 * lane_accumulator(bytes@, 1) + 4 * lane_accumulator(bytes@, 2) - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2), b_vec@.3@, 4 * lane_accumulator(bytes@, 3) - 3 * lane_sum(bytes@, 3));
            adler_congruent_refl(initial_b@ + bytes@.len() * initial_a@ + crate::adler32_weighted_sum(bytes@));
            adler_congruent_trans(b@, initial_b@ + bytes@.len() * initial_a@ + 4 * (lane_accumulator(bytes@, 0) + lane_accumulator(bytes@, 1) + lane_accumulator(bytes@, 2) + lane_accumulator(bytes@, 3)) - lane_sum(bytes@, 1) - 2 * lane_sum(bytes@, 2) - 3 * lane_sum(bytes@, 3), initial_b@ + bytes@.len() * initial_a@ + crate::adler32_weighted_sum(bytes@));
            true
        };
        proof_assert!(adler_congruent(b@,
            initial_b@ + bytes@.len() * initial_a@
                + crate::adler32_weighted_sum(bytes@)));

        // iterate over the remaining few bytes in serial
        let remainder_a_entry = snapshot! { a };
        let remainder_b_entry = snapshot! { b };
        #[invariant(produced.len() <= 3 && a@ <= remainder_a_entry@ + produced.len() * 255 && b@ <= remainder_b_entry@ + produced.len() * remainder_a_entry@ + 255 * produced.len() * (produced.len() + 1) / 2)]
        #[invariant(a@ == remainder_a_entry@ + crate::adler32_byte_sum(remainder@.subsequence(0, produced.len())))]
        #[invariant(b@ == remainder_b_entry@ + produced.len() * remainder_a_entry@ + crate::adler32_weighted_sum(remainder@.subsequence(0, produced.len())))]
        for &byte in remainder.iter() {
            proof_assert! {
                let prefix = remainder@.subsequence(0, produced.len() - 1);
                crate::adler32_byte_sum_push(prefix, byte);
                crate::adler32_weighted_sum_push(prefix, byte);
                remainder@.subsequence(0, produced.len()) == prefix.push_back(byte)
            };
            proof_assert!(a@ + byte@ <= u32::MAX@);
            a += u32::from(byte);
            proof_assert!(b@ + a@ <= u32::MAX@);
            b += a;
        }

        proof_assert! {
            adler_congruent_refl(crate::adler32_byte_sum(remainder@));
            adler_congruent_add(remainder_a_entry@, initial_a@ + crate::adler32_byte_sum(bytes@), crate::adler32_byte_sum(remainder@), crate::adler32_byte_sum(remainder@));
            adler_congruent_scale(remainder_a_entry@, initial_a@ + crate::adler32_byte_sum(bytes@), remainder@.len());
            adler_congruent_add(remainder_b_entry@, initial_b@ + bytes@.len() * initial_a@ + crate::adler32_weighted_sum(bytes@), remainder@.len() * remainder_a_entry@, remainder@.len() * (initial_a@ + crate::adler32_byte_sum(bytes@)));
            adler_congruent_refl(crate::adler32_weighted_sum(remainder@));
            adler_congruent_add(remainder_b_entry@ + remainder@.len() * remainder_a_entry@, initial_b@ + bytes@.len() * initial_a@ + crate::adler32_weighted_sum(bytes@) + remainder@.len() * (initial_a@ + crate::adler32_byte_sum(bytes@)), crate::adler32_weighted_sum(remainder@), crate::adler32_weighted_sum(remainder@));
            crate::adler32_byte_sum_concat(bytes@, remainder@);
            crate::adler32_weighted_sum_concat(bytes@, remainder@);
            *original_bytes == bytes@.concat(remainder@)
        };
        proof_assert! {
            adler_congruent_refl(initial_a@ + crate::adler32_byte_sum(*original_bytes));
            adler_congruent_trans(a@, initial_a@ + crate::adler32_byte_sum(bytes@) + crate::adler32_byte_sum(remainder@), initial_a@ + crate::adler32_byte_sum(*original_bytes));
            adler_congruent_refl(initial_b@ + (*original_bytes).len() * initial_a@ + crate::adler32_weighted_sum(*original_bytes));
            adler_congruent_trans(b@, initial_b@ + bytes@.len() * initial_a@ + crate::adler32_weighted_sum(bytes@) + remainder@.len() * (initial_a@ + crate::adler32_byte_sum(bytes@)) + crate::adler32_weighted_sum(remainder@), initial_b@ + (*original_bytes).len() * initial_a@ + crate::adler32_weighted_sum(*original_bytes));
            true
        };
        proof_assert!(adler_congruent(a@,
            initial_a@ + crate::adler32_byte_sum(*original_bytes)));
        proof_assert!(adler_congruent(b@,
            initial_b@ + (*original_bytes).len() * initial_a@
                + crate::adler32_weighted_sum(*original_bytes)));
        let final_a = snapshot! { a };
        let final_b = snapshot! { b };
        self.a = (a % MOD) as u16;
        self.b = (b % MOD) as u16;
        proof_assert! {
            adler_remainder_congruent(final_a@);
            adler_remainder_congruent(final_b@);
            adler_congruent_trans(self.a@, final_a@, initial_a@ + crate::adler32_byte_sum(*original_bytes));
            adler_congruent_trans(self.b@, final_b@, initial_b@ + (*original_bytes).len() * initial_a@ + crate::adler32_weighted_sum(*original_bytes));
            crate::adler32_update_reduced((initial_a@, initial_b@), *original_bytes);
            adler_congruent_reduced(self.a@, (initial_a@ + crate::adler32_byte_sum(*original_bytes)) % 65521);
            adler_congruent_reduced(self.b@, (initial_b@ + (*original_bytes).len() * initial_a@ + crate::adler32_weighted_sum(*original_bytes)) % 65521);
            true
        };
        proof_assert!(self.a@ ==
            (initial_a@ + crate::adler32_byte_sum(*original_bytes)) % 65521);
        proof_assert!(self.b@ ==
            (initial_b@ + (*original_bytes).len() * initial_a@
                + crate::adler32_weighted_sum(*original_bytes)) % 65521);
    }
}

#[derive(Copy, Clone)]
struct U32X4(pub [u32; 4]);
#[allow(non_snake_case)]
mod u32x4_model {
    use super::U32X4;
    #[allow(unused_imports)]
    use creusot_std::prelude::{logic, pearlite, Invariant, View};

    impl View for U32X4 {
        type ViewTy = (u32, u32, u32, u32);

        #[logic(open)]
        fn view(self) -> Self::ViewTy {
            (self.0[0], self.0[1], self.0[2], self.0[3])
        }
    }

    impl Invariant for U32X4 {
        #[logic(open)]
        fn invariant(self) -> bool {
            pearlite! { self.0@.len() == 4 }
        }
    }
}

impl AddAssign<Self> for U32X4 {
    #[inline]
    #[requires((*self)@.0@ + other@.0@ <= u32::MAX@)]
    #[requires((*self)@.1@ + other@.1@ <= u32::MAX@)]
    #[requires((*self)@.2@ + other@.2@ <= u32::MAX@)]
    #[requires((*self)@.3@ + other@.3@ <= u32::MAX@)]
    #[ensures((^self)@.0@ == (*self)@.0@ + other@.0@)]
    #[ensures((^self)@.1@ == (*self)@.1@ + other@.1@)]
    #[ensures((^self)@.2@ == (*self)@.2@ + other@.2@)]
    #[ensures((^self)@.3@ == (*self)@.3@ + other@.3@)]
    fn add_assign(&mut self, other: Self) {
        // Implement this in a primitive manner to help out the compiler a bit.
        self.0[0] += other.0[0];
        self.0[1] += other.0[1];
        self.0[2] += other.0[2];
        self.0[3] += other.0[3];
    }
}

impl RemAssign<u32> for U32X4 {
    #[inline]
    #[requires(quotient@ > 0)]
    #[ensures((^self)@.0@ == (*self)@.0@ % quotient@)]
    #[ensures((^self)@.1@ == (*self)@.1@ % quotient@)]
    #[ensures((^self)@.2@ == (*self)@.2@ % quotient@)]
    #[ensures((^self)@.3@ == (*self)@.3@ % quotient@)]
    fn rem_assign(&mut self, quotient: u32) {
        self.0[0] %= quotient;
        self.0[1] %= quotient;
        self.0[2] %= quotient;
        self.0[3] %= quotient;
    }
}

impl MulAssign<u32> for U32X4 {
    #[inline]
    #[requires((*self)@.0@ * rhs@ <= u32::MAX@)]
    #[requires((*self)@.1@ * rhs@ <= u32::MAX@)]
    #[requires((*self)@.2@ * rhs@ <= u32::MAX@)]
    #[requires((*self)@.3@ * rhs@ <= u32::MAX@)]
    #[ensures((^self)@.0@ == (*self)@.0@ * rhs@)]
    #[ensures((^self)@.1@ == (*self)@.1@ * rhs@)]
    #[ensures((^self)@.2@ == (*self)@.2@ * rhs@)]
    #[ensures((^self)@.3@ == (*self)@.3@ * rhs@)]
    fn mul_assign(&mut self, rhs: u32) {
        self.0[0] *= rhs;
        self.0[1] *= rhs;
        self.0[2] *= rhs;
        self.0[3] *= rhs;
    }
}

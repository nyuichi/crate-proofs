extern crate creusot_std;
use crate::Adler32;
#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, proof_assert, requires, snapshot, Int, Invariant as _,
    View as _,
};
use std::ops::{AddAssign, MulAssign, RemAssign};

#[logic(opaque)]
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
#[requires(0 <= a && a <= u16::MAX@ && 0 <= b)]
#[ensures(reduced_state_facts(a, b))]
fn reduced_state_safe(a: Int, b: Int) {}

impl Adler32 {
    #[allow(unused_variables)]
    #[ensures((^self).a@ < 65521)]
    #[ensures((^self).b@ < 65521)]
    pub(crate) fn compute(&mut self, bytes: &[u8]) {
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
        let mut a_vec = U32X4([0; 4]);
        let mut b_vec = a_vec;

        let (bytes, remainder) = bytes.split_at(bytes.len() - bytes.len() % 4);

        // iterate over 4 bytes at a time
        let chunk_iter = bytes.chunks_exact(CHUNK_SIZE);
        let remainder_chunk = chunk_iter.remainder();
        #[invariant(a@ <= u16::MAX@ && b@ <= u16::MAX@ && b@ + 22208 * a@ <= u32::MAX@ && a_vec.invariant() && b_vec.invariant() && a_vec@.0@ < 65521 && a_vec@.1@ < 65521 && a_vec@.2@ < 65521 && a_vec@.3@ < 65521 && b_vec@.0@ < 65521 && b_vec@.1@ < 65521 && b_vec@.2@ < 65521 && b_vec@.3@ < 65521)]
        for chunk in chunk_iter {
            let a_vec_entry = snapshot! { a_vec };
            let b_vec_entry = snapshot! { b_vec };
            #[invariant(a_vec.invariant() && b_vec.invariant() && produced.len() <= 5552)]
            #[invariant(a_vec@.0@ <= a_vec_entry@.0@ + produced.len() * 255 && a_vec@.1@ <= a_vec_entry@.1@ + produced.len() * 255 && a_vec@.2@ <= a_vec_entry@.2@ + produced.len() * 255 && a_vec@.3@ <= a_vec_entry@.3@ + produced.len() * 255)]
            #[invariant(2 * b_vec@.0@ <= 2 * b_vec_entry@.0@ + 2 * produced.len() * a_vec_entry@.0@ + 255 * produced.len() * (produced.len() + 1) && 2 * b_vec@.1@ <= 2 * b_vec_entry@.1@ + 2 * produced.len() * a_vec_entry@.1@ + 255 * produced.len() * (produced.len() + 1) && 2 * b_vec@.2@ <= 2 * b_vec_entry@.2@ + 2 * produced.len() * a_vec_entry@.2@ + 255 * produced.len() * (produced.len() + 1) && 2 * b_vec@.3@ <= 2 * b_vec_entry@.3@ + 2 * produced.len() * a_vec_entry@.3@ + 255 * produced.len() * (produced.len() + 1))]
            for byte_vec in chunk.chunks_exact(4) {
                let val = U32X4::from(byte_vec);
                proof_assert! { chunk_iteration_safe(produced.len() - 1, a_vec_entry@.0@, b_vec_entry@.0@, a_vec@.0@, b_vec@.0@, val@.0@); chunk_iteration_facts(produced.len() - 1, a_vec_entry@.0@, b_vec_entry@.0@, a_vec@.0@, b_vec@.0@, val@.0@) };
                proof_assert! { chunk_iteration_safe(produced.len() - 1, a_vec_entry@.1@, b_vec_entry@.1@, a_vec@.1@, b_vec@.1@, val@.1@); chunk_iteration_facts(produced.len() - 1, a_vec_entry@.1@, b_vec_entry@.1@, a_vec@.1@, b_vec@.1@, val@.1@) };
                proof_assert! { chunk_iteration_safe(produced.len() - 1, a_vec_entry@.2@, b_vec_entry@.2@, a_vec@.2@, b_vec@.2@, val@.2@); chunk_iteration_facts(produced.len() - 1, a_vec_entry@.2@, b_vec_entry@.2@, a_vec@.2@, b_vec@.2@, val@.2@) };
                proof_assert! { chunk_iteration_safe(produced.len() - 1, a_vec_entry@.3@, b_vec_entry@.3@, a_vec@.3@, b_vec@.3@, val@.3@); chunk_iteration_facts(produced.len() - 1, a_vec_entry@.3@, b_vec_entry@.3@, a_vec@.3@, b_vec@.3@, val@.3@) };
                proof_assert!(a_vec@.0@ + val@.0@ <= u32::MAX@ && a_vec@.1@ + val@.1@ <= u32::MAX@ && a_vec@.2@ + val@.2@ <= u32::MAX@ && a_vec@.3@ + val@.3@ <= u32::MAX@);
                a_vec += val;
                proof_assert!(b_vec@.0@ + a_vec@.0@ <= u32::MAX@ && b_vec@.1@ + a_vec@.1@ <= u32::MAX@ && b_vec@.2@ + a_vec@.2@ <= u32::MAX@ && b_vec@.3@ + a_vec@.3@ <= u32::MAX@);
                b_vec += a_vec;
            }

            proof_assert!(b@ + 22208 * a@ <= u32::MAX@);
            b += CHUNK_SIZE as u32 * a;
            proof_assert! { reduced_state_safe(a@, b@); reduced_state_facts(a@, b@) };
            a_vec %= MOD;
            b_vec %= MOD;
            b %= MOD;
        }
        // special-case the final chunk because it may be shorter than the rest
        let remainder_a_vec_entry = snapshot! { a_vec };
        let remainder_b_vec_entry = snapshot! { b_vec };
        #[invariant(a_vec.invariant() && b_vec.invariant() && produced.len() <= 5552)]
        #[invariant(a_vec@.0@ <= remainder_a_vec_entry@.0@ + produced.len() * 255 && a_vec@.1@ <= remainder_a_vec_entry@.1@ + produced.len() * 255 && a_vec@.2@ <= remainder_a_vec_entry@.2@ + produced.len() * 255 && a_vec@.3@ <= remainder_a_vec_entry@.3@ + produced.len() * 255)]
        #[invariant(2 * b_vec@.0@ <= 2 * remainder_b_vec_entry@.0@ + 2 * produced.len() * remainder_a_vec_entry@.0@ + 255 * produced.len() * (produced.len() + 1) && 2 * b_vec@.1@ <= 2 * remainder_b_vec_entry@.1@ + 2 * produced.len() * remainder_a_vec_entry@.1@ + 255 * produced.len() * (produced.len() + 1) && 2 * b_vec@.2@ <= 2 * remainder_b_vec_entry@.2@ + 2 * produced.len() * remainder_a_vec_entry@.2@ + 255 * produced.len() * (produced.len() + 1) && 2 * b_vec@.3@ <= 2 * remainder_b_vec_entry@.3@ + 2 * produced.len() * remainder_a_vec_entry@.3@ + 255 * produced.len() * (produced.len() + 1))]
        for byte_vec in remainder_chunk.chunks_exact(4) {
            let val = U32X4::from(byte_vec);
            proof_assert! { chunk_iteration_safe(produced.len() - 1, remainder_a_vec_entry@.0@, remainder_b_vec_entry@.0@, a_vec@.0@, b_vec@.0@, val@.0@); chunk_iteration_facts(produced.len() - 1, remainder_a_vec_entry@.0@, remainder_b_vec_entry@.0@, a_vec@.0@, b_vec@.0@, val@.0@) };
            proof_assert! { chunk_iteration_safe(produced.len() - 1, remainder_a_vec_entry@.1@, remainder_b_vec_entry@.1@, a_vec@.1@, b_vec@.1@, val@.1@); chunk_iteration_facts(produced.len() - 1, remainder_a_vec_entry@.1@, remainder_b_vec_entry@.1@, a_vec@.1@, b_vec@.1@, val@.1@) };
            proof_assert! { chunk_iteration_safe(produced.len() - 1, remainder_a_vec_entry@.2@, remainder_b_vec_entry@.2@, a_vec@.2@, b_vec@.2@, val@.2@); chunk_iteration_facts(produced.len() - 1, remainder_a_vec_entry@.2@, remainder_b_vec_entry@.2@, a_vec@.2@, b_vec@.2@, val@.2@) };
            proof_assert! { chunk_iteration_safe(produced.len() - 1, remainder_a_vec_entry@.3@, remainder_b_vec_entry@.3@, a_vec@.3@, b_vec@.3@, val@.3@); chunk_iteration_facts(produced.len() - 1, remainder_a_vec_entry@.3@, remainder_b_vec_entry@.3@, a_vec@.3@, b_vec@.3@, val@.3@) };
            proof_assert!(a_vec@.0@ + val@.0@ <= u32::MAX@ && a_vec@.1@ + val@.1@ <= u32::MAX@ && a_vec@.2@ + val@.2@ <= u32::MAX@ && a_vec@.3@ + val@.3@ <= u32::MAX@);
            a_vec += val;
            proof_assert!(b_vec@.0@ + a_vec@.0@ <= u32::MAX@ && b_vec@.1@ + a_vec@.1@ <= u32::MAX@ && b_vec@.2@ + a_vec@.2@ <= u32::MAX@ && b_vec@.3@ + a_vec@.3@ <= u32::MAX@);
            b_vec += a_vec;
        }
        proof_assert! { partial_chunk_safe(remainder_chunk@.len(), a@, b@); partial_chunk_facts(remainder_chunk@.len(), a@, b@) };
        b += remainder_chunk.len() as u32 * a;
        a_vec %= MOD;
        b_vec %= MOD;
        b %= MOD;

        // combine the sub-sum results into the main sum
        b_vec *= 4;
        b_vec.0[1] += MOD - a_vec.0[1];
        b_vec.0[2] += (MOD - a_vec.0[2]) * 2;
        b_vec.0[3] += (MOD - a_vec.0[3]) * 3;
        let a_entry = snapshot! { a };
        #[invariant(a_vec.invariant() && b_vec.invariant() && produced.len() <= 4 && a@ <= a_entry@ + produced.len() * 65520)]
        for &av in a_vec.0.iter() {
            proof_assert!(a@ + av@ <= u32::MAX@);
            a += av;
        }
        let b_entry = snapshot! { b };
        #[invariant(a_vec.invariant() && b_vec.invariant() && produced.len() <= 4 && b@ <= b_entry@ + produced.len() * 458643)]
        for &bv in b_vec.0.iter() {
            proof_assert!(b@ + bv@ <= u32::MAX@);
            b += bv;
        }

        // iterate over the remaining few bytes in serial
        let remainder_a_entry = snapshot! { a };
        let remainder_b_entry = snapshot! { b };
        #[invariant(produced.len() <= 3 && a@ <= remainder_a_entry@ + produced.len() * 255 && b@ <= remainder_b_entry@ + produced.len() * remainder_a_entry@ + 255 * produced.len() * (produced.len() + 1) / 2)]
        for &byte in remainder.iter() {
            proof_assert!(a@ + byte@ <= u32::MAX@);
            a += u32::from(byte);
            proof_assert!(b@ + a@ <= u32::MAX@);
            b += a;
        }

        self.a = (a % MOD) as u16;
        self.b = (b % MOD) as u16;
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

impl U32X4 {
    #[inline]
    #[requires(bytes@.len() >= 4)]
    #[ensures(result@.0@ == bytes@[0]@)]
    #[ensures(result@.1@ == bytes@[1]@)]
    #[ensures(result@.2@ == bytes@[2]@)]
    #[ensures(result@.3@ == bytes@[3]@)]
    fn from(bytes: &[u8]) -> Self {
        U32X4([
            u32::from(bytes[0]),
            u32::from(bytes[1]),
            u32::from(bytes[2]),
            u32::from(bytes[3]),
        ])
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

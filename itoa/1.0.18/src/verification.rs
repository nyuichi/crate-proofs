use core::str;

#[allow(unused_imports)]
use creusot_std::prelude::{
    check, ensures, logic, pearlite, proof_assert, requires, snapshot, trusted, variant, Int, Seq,
    View,
};

/// Decimal ASCII byte values for a nonnegative mathematical integer.
#[logic(open)]
#[requires(n >= 0)]
#[variant(n)]
pub fn decimal_values(n: Int) -> Seq<Int> {
    if n < 10 {
        Seq::singleton(48 + n)
    } else {
        decimal_values(n / 10).push_back(48 + n % 10)
    }
}

#[logic]
#[requires(n >= 0)]
#[ensures(decimal_values(n) == if n < 10 {
    Seq::singleton(48 + n)
} else {
    decimal_values(n / 10).push_back(48 + n % 10)
})]
fn decimal_values_unfold(n: Int) {}

#[logic]
#[requires(0 <= n && n < 10)]
#[ensures(decimal_values(n) == Seq::singleton(48 + n))]
fn decimal_values_one_digit(n: Int) {
    decimal_values_unfold(n);
}

#[logic(open)]
#[requires(0 <= exponent)]
#[variant(exponent)]
pub fn power_of_ten(exponent: Int) -> Int {
    if exponent == 0 {
        1
    } else {
        10 * power_of_ten(exponent - 1)
    }
}

#[logic]
#[requires(0 <= exponent)]
#[ensures(power_of_ten(exponent) == if exponent == 0 {
    1
} else {
    10 * power_of_ten(exponent - 1)
})]
fn power_of_ten_unfold(exponent: Int) {}

#[logic]
#[ensures(power_of_ten(39)
    == 10 * 100_000_000_000_000_000_000_000_000_000_000_000_000)]
fn power_of_ten_39() {
    power_of_ten_unfold(39);
    power_of_ten_unfold(38);
    power_of_ten_unfold(37);
    power_of_ten_unfold(36);
    power_of_ten_unfold(35);
    power_of_ten_unfold(34);
    power_of_ten_unfold(33);
    power_of_ten_unfold(32);
    power_of_ten_unfold(31);
    power_of_ten_unfold(30);
    power_of_ten_unfold(29);
    power_of_ten_unfold(28);
    power_of_ten_unfold(27);
    power_of_ten_unfold(26);
    power_of_ten_unfold(25);
    power_of_ten_unfold(24);
    power_of_ten_unfold(23);
    power_of_ten_unfold(22);
    power_of_ten_unfold(21);
    power_of_ten_unfold(20);
    power_of_ten_unfold(19);
    power_of_ten_unfold(18);
    power_of_ten_unfold(17);
    power_of_ten_unfold(16);
    power_of_ten_unfold(15);
    power_of_ten_unfold(14);
    power_of_ten_unfold(13);
    power_of_ten_unfold(12);
    power_of_ten_unfold(11);
    power_of_ten_unfold(10);
    power_of_ten_unfold(9);
    power_of_ten_unfold(8);
    power_of_ten_unfold(7);
    power_of_ten_unfold(6);
    power_of_ten_unfold(5);
    power_of_ten_unfold(4);
    power_of_ten_unfold(3);
    power_of_ten_unfold(2);
    power_of_ten_unfold(1);
    power_of_ten_unfold(0);
}

#[logic]
#[requires(0 <= n)]
#[requires(1 <= digits)]
#[requires(n < power_of_ten(digits))]
#[ensures(decimal_values(n).len() <= digits)]
#[variant(digits)]
fn decimal_len_bounded(n: Int, digits: Int) {
    decimal_values_unfold(n);
    if n >= 10 {
        proof_assert!(1 < digits);
        proof_assert!(n / 10 < power_of_ten(digits - 1));
        decimal_len_bounded(n / 10, digits - 1);
    }
}

/// Decimal ASCII byte values, including a leading minus sign when needed.
#[logic(open)]
pub fn signed_decimal_values(n: Int) -> Seq<Int> {
    if n < 0 {
        pearlite! { Seq::singleton(45).concat(decimal_values(-n)) }
    } else {
        decimal_values(n)
    }
}

/// Exact whole-buffer effect of the recursive digit writer.
#[logic(open)]
#[requires(n >= 0)]
#[requires(end <= values.len())]
#[requires(decimal_values(n).len() <= end)]
#[variant(n)]
pub fn write_model(values: Seq<Int>, n: Int, end: Int) -> Seq<Int> {
    if n < 10 {
        values.set(end - 1, 48 + n)
    } else {
        write_model(values, n / 10, end - 1).set(end - 1, 48 + n % 10)
    }
}

#[logic]
#[requires(n >= 0)]
#[requires(end <= values.len())]
#[requires(decimal_values(n).len() <= end)]
#[ensures(write_model(values, n, end) == if n < 10 {
    values.set(end - 1, 48 + n)
} else {
    write_model(values, n / 10, end - 1).set(end - 1, 48 + n % 10)
})]
fn write_model_unfold(values: Seq<Int>, n: Int, end: Int) {}

#[logic]
#[requires(n >= 0)]
#[requires(end <= values.len())]
#[requires(decimal_values(n).len() <= end)]
#[ensures(write_model(values, n, end).len() == values.len())]
#[variant(n)]
fn write_model_preserves_len(values: Seq<Int>, n: Int, end: Int) {
    if n >= 10 {
        decimal_values_unfold(n);
        write_model_preserves_len(values, n / 10, end - 1);
    }
}

/// Representation lemma connecting the update-oriented model to canonical
/// most-significant-first decimal bytes.
#[logic]
#[requires(n >= 0)]
#[requires(end <= values.len())]
#[requires(decimal_values(n).len() <= end)]
#[ensures(write_model(values, n, end)
    .subsequence(end - decimal_values(n).len(), end) == decimal_values(n))]
#[variant(n)]
fn write_model_is_decimal(values: Seq<Int>, n: Int, end: Int) {
    if n >= 10 {
        decimal_values_unfold(n);
        write_model_preserves_len(values, n, end);
        write_model_preserves_len(values, n / 10, end - 1);
        write_model_is_decimal(values, n / 10, end - 1);
        let prefix = decimal_values(n / 10);
        let digit = 48 + n % 10;
        let start = end - prefix.len() - 1;
        let previous = write_model(values, n / 10, end - 1);
        let output = write_model(values, n, end);
        proof_assert!(decimal_values(n) == prefix.push_back(digit));
        proof_assert!(decimal_values(n).len() == prefix.len() + 1);
        proof_assert!(prefix.len() + 1 <= end);
        proof_assert!(0 <= start);
        proof_assert!(output.len() == values.len());
        proof_assert!(previous.len() == values.len());
        proof_assert!(end <= output.len());
        proof_assert!(output == previous.set(end - 1, digit));
        proof_assert!(start + prefix.len() == end - 1);
        proof_assert!(output.subsequence(start, end).len() == prefix.len() + 1);
        proof_assert!(forall<i: Int> 0 <= i && i < prefix.len() ==>
            start + i < end - 1);
        proof_assert!(forall<i: Int> 0 <= i && i < prefix.len() ==>
            output[start + i] == previous[start + i]);
        proof_assert!(forall<i: Int> 0 <= i && i < prefix.len() ==>
            previous.subsequence(start, end - 1)[i] == prefix[i]);
        proof_assert!(forall<i: Int> 0 <= i && i < prefix.len() ==>
            output.subsequence(start, end)[i] == prefix[i]);
        proof_assert!(output[end - 1] == digit);
        proof_assert!(output.subsequence(start, end)[prefix.len()] == output[start + prefix.len()]);
        proof_assert!(output.subsequence(start, end)[prefix.len()] == digit);
        proof_assert!(output.subsequence(start, end) == prefix.push_back(digit));
    } else {
        decimal_values_unfold(n);
        write_model_preserves_len(values, n, end);
        let output = write_model(values, n, end);
        proof_assert!(n < 10);
        proof_assert!(decimal_values(n) == Seq::singleton(48 + n));
        proof_assert!(decimal_values(n).len() == 1);
        proof_assert!(1 <= end);
        proof_assert!(0 <= end - 1);
        proof_assert!(output.len() == values.len());
        proof_assert!(end <= output.len());
        proof_assert!(output == values.set(end - 1, 48 + n));
        proof_assert!(output[end - 1] == 48 + n);
        proof_assert!(output.subsequence(end - 1, end).len() == 1);
        proof_assert!(output.subsequence(end - 1, end)[0] == output[end - 1]);
        proof_assert!(output.subsequence(end - 1, end)[0] == 48 + n);
        proof_assert!(output.subsequence(end - 1, end) == Seq::singleton(48 + n));
    }
}

mod private {
    use super::*;

    pub trait Sealed: Copy {
        #[logic]
        fn value(self) -> Int;

        #[ensures(result@ == if Self::value(self) < 0 {
            -Self::value(self)
        } else {
            Self::value(self)
        })]
        fn magnitude(self) -> u128;

        #[ensures(result == (Self::value(self) < 0))]
        fn is_negative(self) -> bool;
    }
}

/// An integer that can be written into a [`Buffer`].
///
/// This trait is sealed and has the same implementations as upstream itoa.
pub trait Integer: private::Sealed {
    /// Maximum decimal representation length for this type.
    const MAX_STR_LEN: usize;
}

/// Exact mathematical integer value of an [`Integer`].
#[logic(open)]
pub fn integer_value<I: Integer>(value: I) -> Int {
    private::Sealed::value(value)
}

/// Exact decimal ASCII model used by [`Buffer::format`].
#[logic(open)]
pub fn integer_decimal_values<I: Integer>(value: I) -> Seq<Int> {
    signed_decimal_values(integer_value(value))
}

macro_rules! impl_unsigned {
    ($($ty:ty => $len:expr),* $(,)?) => {$($crate::verification::impl_unsigned!(@one $ty, $len);)*};
    (@one $ty:ty, $len:expr) => {
        impl Integer for $ty {
            const MAX_STR_LEN: usize = $len;
        }

        impl private::Sealed for $ty {
            #[logic(open)]
            fn value(self) -> Int {
                pearlite! { self@ }
            }

            #[ensures(result@ == Self::value(self))]
            fn magnitude(self) -> u128 {
                self as u128
            }

            #[ensures(!result)]
            fn is_negative(self) -> bool {
                false
            }
        }
    };
}

// Keep this macro local while allowing its recursive arm to be addressed.
pub(crate) use impl_unsigned;

impl_unsigned! {
    u8 => 3,
    u16 => 5,
    u32 => 10,
    u64 => 20,
    u128 => 39,
    usize => 20,
}

macro_rules! impl_signed_widening {
    ($($ty:ty => $len:expr),* $(,)?) => {$($crate::verification::impl_signed_widening!(@one $ty, $len);)*};
    (@one $ty:ty, $len:expr) => {
        impl Integer for $ty {
            const MAX_STR_LEN: usize = $len;
        }

        impl private::Sealed for $ty {
            #[logic(open)]
            fn value(self) -> Int {
                pearlite! { self@ }
            }

            #[ensures(result@ == if Self::value(self) < 0 {
                -Self::value(self)
            } else {
                Self::value(self)
            })]
            fn magnitude(self) -> u128 {
                let wide = self as i128;
                if wide < 0 {
                    (-wide) as u128
                } else {
                    wide as u128
                }
            }

            #[ensures(result == (Self::value(self) < 0))]
            fn is_negative(self) -> bool {
                self < 0
            }
        }
    };
}

pub(crate) use impl_signed_widening;

impl_signed_widening! {
    i8 => 4,
    i16 => 6,
    i32 => 11,
    i64 => 20,
    isize => 20,
}

impl Integer for i128 {
    const MAX_STR_LEN: usize = 40;
}

impl private::Sealed for i128 {
    #[logic(open)]
    fn value(self) -> Int {
        pearlite! { self@ }
    }

    #[ensures(result@ == if Self::value(self) < 0 {
        -Self::value(self)
    } else {
        Self::value(self)
    })]
    fn magnitude(self) -> u128 {
        if self == i128::MIN {
            (i128::MAX as u128) + 1
        } else if self < 0 {
            (-self) as u128
        } else {
            self as u128
        }
    }

    #[ensures(result == (Self::value(self) < 0))]
    fn is_negative(self) -> bool {
        self < 0
    }
}

/// Verification-facing representation of itoa's fixed stack buffer.
///
/// Runtime builds retain upstream's `MaybeUninit` representation and optimized
/// lookup-table algorithm. This initialized representation makes the byte
/// recurrence independently provable without adding raw-memory facts to it.
pub struct Buffer {
    bytes: [u8; 40],
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer { bytes: [0; 40] }
    }

    /// Print an integer and return its exact signed decimal representation.
    #[ensures(result@.to_bytes().map(|byte: u8| byte@) == integer_decimal_values(i))]
    pub fn format<I: Integer>(&mut self, i: I) -> &str {
        let magnitude = private::Sealed::magnitude(i);
        let negative = private::Sealed::is_negative(i);
        proof_assert!(0 <= magnitude@);
        proof_assert! {
            let _ = power_of_ten_39();
            magnitude@ < power_of_ten(39)
        };
        proof_assert! {
            let _ = decimal_len_bounded(magnitude@, 39);
            decimal_values(magnitude@).len() <= 39
        };
        let mut start = write_unsigned(magnitude, &mut self.bytes, 40);
        let digits_start = start;
        if negative {
            start -= 1;
            self.bytes[start] = b'-';
            proof_assert!(integer_value(i) < 0);
            proof_assert!(magnitude@ == -integer_value(i));
            proof_assert!(self.bytes@.subsequence(digits_start@, 40)
                .map(|byte: u8| byte@) == decimal_values(magnitude@));
            proof_assert!(self.bytes@.subsequence(start@, 40)
                .map(|byte: u8| byte@).len() == decimal_values(magnitude@).len() + 1);
            proof_assert!(self.bytes@.subsequence(start@, 40)
                .map(|byte: u8| byte@)[0] == 45);
            proof_assert!(forall<j: Int> 0 <= j && j < decimal_values(magnitude@).len() ==>
                self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@)[j + 1]
                    == decimal_values(magnitude@)[j]);
            proof_assert!(self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@)
                == Seq::singleton(45).concat(decimal_values(magnitude@)));
            proof_assert!(self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@)
                == signed_decimal_values(integer_value(i)));
        } else {
            proof_assert!(0 <= integer_value(i));
            proof_assert!(magnitude@ == integer_value(i));
            proof_assert!(self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@)
                == signed_decimal_values(integer_value(i)));
        }
        // SAFETY: write_unsigned emits only ASCII digits, and the optional byte
        // immediately before them is the ASCII minus sign.
        let output = unsafe { decimal_slice_to_str(&self.bytes, start) };
        proof_assert!(output@.to_bytes().map(|byte: u8| byte@)
            == self.bytes@.subsequence(start@, 40).map(|byte: u8| byte@));
        proof_assert!(output@.to_bytes().map(|byte: u8| byte@)
            == integer_decimal_values(i));
        output
    }
}

impl Default for Buffer {
    fn default() -> Buffer {
        Buffer::new()
    }
}

impl Copy for Buffer {}

#[allow(clippy::non_canonical_clone_impl)]
impl Clone for Buffer {
    fn clone(&self) -> Self {
        Buffer::new()
    }
}

/// One byte update expressed directly in the integer-valued vocabulary used
/// by the decimal writer model.
#[requires(index@ < buf@.len())]
#[ensures((^buf)@.map(|value: u8| value@)
    == buf@.map(|value: u8| value@).set(index@, byte@))]
#[check(terminates)]
fn write_byte(buf: &mut [u8], index: usize, byte: u8) {
    buf[index] = byte;
    proof_assert!((^buf)@.map(|value: u8| value@).len()
        == buf@.map(|value: u8| value@).len());
    proof_assert!(forall<i: Int> 0 <= i && i < buf@.len() ==>
        (^buf)@.map(|value: u8| value@)[i]
            == buf@.map(|value: u8| value@).set(index@, byte@)[i]);
}

/// Write the canonical unsigned decimal representation into the suffix ending
/// at `end`. Recursion has one progress measure: the remaining magnitude.
#[requires(end@ <= buf@.len())]
#[requires(decimal_values(n@).len() <= end@)]
#[ensures((^buf)@.map(|byte: u8| byte@)
    == write_model(buf@.map(|byte: u8| byte@), n@, end@))]
#[ensures(result@ + decimal_values(n@).len() == end@)]
#[ensures((^buf)@.subsequence(result@, end@).map(|byte: u8| byte@) == decimal_values(n@))]
#[check(terminates)]
#[variant(n)]
fn write_unsigned(n: u128, buf: &mut [u8], end: usize) -> usize {
    let before = snapshot!(buf@);
    proof_assert!(0 <= n@);
    let result = if n < 10 {
        proof_assert!(n@ < 10);
        proof_assert! {
            let _ = decimal_values_one_digit(n@);
            decimal_values(n@) == Seq::singleton(48 + n@)
        };
        proof_assert!(decimal_values(n@).len() == 1);
        proof_assert!(1 <= end@);
        let start = end - 1;
        write_byte(buf, start, b'0' + n as u8);
        proof_assert!(start@ + decimal_values(n@).len() == end@);
        start
    } else {
        proof_assert!(10 <= n@);
        proof_assert!(decimal_values(n@)
            == decimal_values(n@ / 10).push_back(48 + n@ % 10));
        proof_assert!(decimal_values(n@).len() == decimal_values(n@ / 10).len() + 1);
        proof_assert!(decimal_values(n@ / 10).len() + 1 <= end@);
        proof_assert!(decimal_values(n@ / 10).len() <= end@ - 1);
        let digit = (n % 10) as u8;
        let start = write_unsigned(n / 10, buf, end - 1);
        write_byte(buf, end - 1, b'0' + digit);
        proof_assert!(start@ + decimal_values(n@).len() == end@);
        start
    };
    proof_assert! {
        let _ = decimal_values_unfold(n@);
        result@ + decimal_values(n@).len() == end@
    };
    proof_assert! {
        let _ = write_model_unfold((*before).map(|byte: u8| byte@), n@, end@);
        (^buf)@.map(|byte: u8| byte@)
            == write_model((*before).map(|byte: u8| byte@), n@, end@)
    };
    proof_assert! {
        let _ = write_model_is_decimal((*before).map(|byte: u8| byte@), n@, end@);
        (^buf)@.map(|byte: u8| byte@).subsequence(result@, end@)
            == decimal_values(n@)
    };
    proof_assert!((^buf)@.subsequence(result@, end@).map(|byte: u8| byte@).len()
        == decimal_values(n@).len());
    proof_assert!(forall<i: Int> 0 <= i && i < decimal_values(n@).len() ==>
        (^buf)@.subsequence(result@, end@).map(|byte: u8| byte@)[i]
            == (^buf)@.map(|byte: u8| byte@).subsequence(result@, end@)[i]);
    proof_assert!((^buf)@.subsequence(result@, end@).map(|byte: u8| byte@)
        == decimal_values(n@));
    result
}

/// UTF-8/reference construction boundary. The caller proves the complete byte
/// suffix; this leaf only exposes the corresponding `str` byte sequence.
// TODO: remove `trusted` once Creusot can derive ASCII UTF-8 validity and model
// the slice-to-str reference conversion without raw representation casts.
#[trusted]
#[requires(start@ <= buf@.len())]
#[ensures(result@.to_bytes() == buf@.subsequence(start@, buf@.len()))]
unsafe fn decimal_slice_to_str(buf: &[u8], start: usize) -> &str {
    unsafe { str::from_utf8_unchecked(&buf[start..]) }
}

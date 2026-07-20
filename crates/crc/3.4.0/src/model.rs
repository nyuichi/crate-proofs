//! Mathematical CRC models used by the public contracts.
//!
//! The model deliberately describes the byte-at-a-time recurrence.  The
//! lookup-table and slicing-by-16 implementations must refine this model, so
//! clients never need to reason about the chosen implementation strategy.

#[allow(unused_imports)]
use creusot_std::prelude::{logic, pearlite, requires, variant, Int, Seq};

macro_rules! define_crc_model {
    (
        $word:ty, $bits:expr, $bits_u8:expr, $top_shift:expr, $byte_shift:expr,
        $zero:expr, $one:expr, $byte_mask:expr,
        $valid:ident, $round:ident, $rounds:ident, $reverse_acc:ident, $reverse:ident,
        $byte:ident, $step:ident, $update:ident, $init:ident,
        $finalize:ident, $checksum:ident, $table_valid:ident, $lane:ident
    ) => {
        /// Whether an algorithm width fits the selected machine register.
        #[logic(open)]
        pub fn $valid(width: u8) -> bool {
            pearlite! { 1 <= width@ && width@ <= $bits }
        }

        /// One polynomial feedback round on an already aligned register.
        #[logic(open)]
        pub fn $round(poly: $word, reflect: bool, value: $word) -> $word {
            pearlite! {
                if reflect {
                    (value >> 1u8) ^ ((value & $one) * poly)
                } else {
                    (value << 1u8) ^ (((value >> $top_shift) & $one) * poly)
                }
            }
        }

        /// Repeated polynomial feedback rounds.
        #[logic]
        #[requires(0 <= count)]
        #[variant(count)]
        pub fn $rounds(poly: $word, reflect: bool, value: $word, count: Int) -> $word {
            if count == 0 {
                value
            } else {
                $rounds(poly, reflect, $round(poly, reflect, value), count - 1)
            }
        }

        /// Accumulator form of bit reversal across a fixed number of bits.
        #[logic]
        #[requires(0 <= count)]
        #[variant(count)]
        pub fn $reverse_acc(value: $word, accumulator: $word, count: Int) -> $word {
            if count == 0 {
                accumulator
            } else {
                $reverse_acc(
                    value >> 1u8,
                    (accumulator << 1u8) | (value & $one),
                    count - 1,
                )
            }
        }

        /// Bit reversal across the complete machine register.
        #[logic(open)]
        pub fn $reverse(value: $word) -> $word {
            pearlite! { $reverse_acc(value, $zero, $bits) }
        }

        /// The table-independent contribution of one input byte.
        #[logic(open)]
        pub fn $byte(poly: $word, reflect: bool, value: $word) -> $word {
            pearlite! {
                if reflect {
                    $rounds(poly, true, value, 8)
                } else {
                    $rounds(poly, false, value << $byte_shift, 8)
                }
            }
        }

        /// One byte update of the internal, aligned CRC register.
        #[logic(open)]
        #[requires($valid(width))]
        pub fn $step(width: u8, poly: $word, reflect: bool, crc: $word, input: u8) -> $word {
            pearlite! {
                let aligned_poly = if reflect {
                    $reverse(poly) >> ($bits_u8 - width)
                } else {
                    poly << ($bits_u8 - width)
                };
                if reflect {
                    $byte(aligned_poly, true, (crc ^ (input as $word)) & $byte_mask)
                        ^ (crc >> 8u8)
                } else {
                    $byte(
                        aligned_poly,
                        false,
                        ((crc >> $byte_shift) ^ (input as $word)) & $byte_mask,
                    ) ^ (crc << 8u8)
                }
            }
        }

        /// Fold the byte recurrence over a complete byte sequence.
        #[logic]
        #[requires($valid(width))]
        #[variant(bytes.len())]
        pub fn $update(width: u8, poly: $word, reflect: bool, crc: $word, bytes: Seq<u8>) -> $word {
            if bytes.len() == 0 {
                crc
            } else {
                let prefix = bytes.subsequence(0, bytes.len() - 1);
                $step(
                    width,
                    poly,
                    reflect,
                    $update(width, poly, reflect, crc, prefix),
                    bytes[bytes.len() - 1],
                )
            }
        }

        /// Align an externally supplied initial value to the runtime register.
        #[logic(open)]
        #[requires($valid(width))]
        pub fn $init(width: u8, reflect: bool, initial: $word) -> $word {
            pearlite! {
                if reflect {
                    $reverse(initial) >> ($bits_u8 - width)
                } else {
                    initial << ($bits_u8 - width)
                }
            }
        }

        /// Apply reflection, width alignment, and the final XOR.
        #[logic(open)]
        #[requires($valid(width))]
        pub fn $finalize(width: u8, refin: bool, refout: bool, xorout: $word, crc: $word) -> $word {
            pearlite! {
                let reflected = if refin != refout { $reverse(crc) } else { crc };
                let aligned = if refout {
                    reflected
                } else {
                    reflected >> ($bits_u8 - width)
                };
                aligned ^ xorout
            }
        }

        /// Complete CRC result for a byte sequence and algorithm parameters.
        #[logic(open)]
        #[requires($valid(width))]
        pub fn $checksum(
            width: u8,
            poly: $word,
            initial: $word,
            refin: bool,
            refout: bool,
            xorout: $word,
            bytes: Seq<u8>,
        ) -> $word {
            pearlite! {
                $finalize(
                    width,
                    refin,
                    refout,
                    xorout,
                    $update(width, poly, refin, $init(width, refin, initial), bytes),
                )
            }
        }

        /// Representation relation between generated lookup lanes and the
        /// table-independent byte recurrence.
        #[logic(open)]
        #[requires($valid(width))]
        pub fn $table_valid<const L: usize>(
            width: u8,
            poly: $word,
            reflect: bool,
            data: [[$word; 256]; L],
        ) -> bool {
            pearlite! {
                let aligned_poly = if reflect {
                    $reverse(poly) >> ($bits_u8 - width)
                } else {
                    poly << ($bits_u8 - width)
                };
                (forall<i: u8> 0 < data@.len() ==>
                    data[0][i@] == $byte(aligned_poly, reflect, i as $word))
                && (forall<e: Int, i: u8>
                    1 <= e && e < data@.len() ==>
                        data[e][i@] == $lane(data[0]@, data[e - 1][i@], reflect))
            }
        }
    };
}

#[logic(open)]
pub fn crc8_lane_model(table: Seq<u8>, previous: u8, _reflect: bool) -> u8 {
    pearlite! { table[previous@] }
}

// Reflected and non-reflected tables use different lane recurrences, so the
// relation is selected explicitly by the algorithm's `refin` flag below.
#[logic(open)]
pub fn crc16_lane_model(table: Seq<u16>, previous: u16, reflect: bool) -> u16 {
    pearlite! {
        if reflect {
            (previous >> 8u8) ^ table[(previous & 0xffu16)@]
        } else {
            (previous << 8u8) ^ table[((previous >> 8u8) & 0xffu16)@]
        }
    }
}

#[logic(open)]
pub fn crc32_lane_model(table: Seq<u32>, previous: u32, reflect: bool) -> u32 {
    pearlite! {
        if reflect {
            (previous >> 8u8) ^ table[(previous & 0xffu32)@]
        } else {
            (previous << 8u8) ^ table[((previous >> 24u8) & 0xffu32)@]
        }
    }
}

#[logic(open)]
pub fn crc64_lane_model(table: Seq<u64>, previous: u64, reflect: bool) -> u64 {
    pearlite! {
        if reflect {
            (previous >> 8u8) ^ table[(previous & 0xffu64)@]
        } else {
            (previous << 8u8) ^ table[((previous >> 56u8) & 0xffu64)@]
        }
    }
}

#[logic(open)]
pub fn crc128_lane_model(table: Seq<u128>, previous: u128, reflect: bool) -> u128 {
    pearlite! {
        if reflect {
            (previous >> 8u8) ^ table[(previous & 0xffu128)@]
        } else {
            (previous << 8u8) ^ table[((previous >> 120u8) & 0xffu128)@]
        }
    }
}

define_crc_model!(
    u8,
    8,
    8u8,
    7u8,
    0u8,
    0u8,
    1u8,
    0xffu8,
    valid_width_8,
    crc8_round_model,
    crc8_rounds_model,
    reverse_u8_acc_model,
    reverse_u8_model,
    crc8_byte_model,
    crc8_step_model,
    crc8_update_model,
    crc8_init_model,
    crc8_finalize_model,
    crc8_checksum_model,
    crc8_table_valid,
    crc8_lane_model
);
define_crc_model!(
    u16,
    16,
    16u8,
    15u8,
    8u8,
    0u16,
    1u16,
    0xffu16,
    valid_width_16,
    crc16_round_model,
    crc16_rounds_model,
    reverse_u16_acc_model,
    reverse_u16_model,
    crc16_byte_model,
    crc16_step_model,
    crc16_update_model,
    crc16_init_model,
    crc16_finalize_model,
    crc16_checksum_model,
    crc16_table_valid,
    crc16_lane_model
);
define_crc_model!(
    u32,
    32,
    32u8,
    31u8,
    24u8,
    0u32,
    1u32,
    0xffu32,
    valid_width_32,
    crc32_round_model,
    crc32_rounds_model,
    reverse_u32_acc_model,
    reverse_u32_model,
    crc32_byte_model,
    crc32_step_model,
    crc32_update_model,
    crc32_init_model,
    crc32_finalize_model,
    crc32_checksum_model,
    crc32_table_valid,
    crc32_lane_model
);
define_crc_model!(
    u64,
    64,
    64u8,
    63u8,
    56u8,
    0u64,
    1u64,
    0xffu64,
    valid_width_64,
    crc64_round_model,
    crc64_rounds_model,
    reverse_u64_acc_model,
    reverse_u64_model,
    crc64_byte_model,
    crc64_step_model,
    crc64_update_model,
    crc64_init_model,
    crc64_finalize_model,
    crc64_checksum_model,
    crc64_table_valid,
    crc64_lane_model
);
define_crc_model!(
    u128,
    128,
    128u8,
    127u8,
    120u8,
    0u128,
    1u128,
    0xffu128,
    valid_width_128,
    crc128_round_model,
    crc128_rounds_model,
    reverse_u128_acc_model,
    reverse_u128_model,
    crc128_byte_model,
    crc128_step_model,
    crc128_update_model,
    crc128_init_model,
    crc128_finalize_model,
    crc128_checksum_model,
    crc128_table_valid,
    crc128_lane_model
);

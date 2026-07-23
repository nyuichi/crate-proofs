#![allow(missing_docs)]

use crate::Uuid;
use creusot_std::prelude::trusted;

#[allow(missing_debug_implementations)]
#[derive(Clone, Copy)]
pub struct Timestamp;

pub trait ClockSequence {
    type Output;
}

pub mod context {
    #[allow(missing_debug_implementations)]
    #[derive(Clone, Copy)]
    pub struct NoContext;
}

impl Timestamp {
    #[trusted]
    pub const fn from_gregorian_time(_ticks: u64, _counter: u16) -> Self {
        Timestamp
    }

    #[trusted]
    pub const fn from_unix_time(
        _seconds: u64,
        _subsec_nanos: u32,
        _counter: u128,
        _usable_counter_bits: u8,
    ) -> Self {
        Timestamp
    }
}

#[trusted]
pub(crate) const fn encode_gregorian_timestamp(
    _ticks: u64,
    _counter: u16,
    _node_id: &[u8; 6],
) -> Uuid {
    Uuid::nil()
}

#[trusted]
pub(crate) const fn encode_sorted_gregorian_timestamp(
    _ticks: u64,
    _counter: u16,
    _node_id: &[u8; 6],
) -> Uuid {
    Uuid::nil()
}

#[trusted]
pub(crate) const fn encode_unix_timestamp_millis(
    _millis: u64,
    _counter_random_bytes: &[u8; 10],
) -> Uuid {
    Uuid::nil()
}

#[trusted]
pub(crate) const fn decode_gregorian_timestamp(_uuid: &Uuid) -> (u64, u16) {
    (0, 0)
}

#[trusted]
pub(crate) const fn decode_sorted_gregorian_timestamp(_uuid: &Uuid) -> (u64, u16) {
    (0, 0)
}

#[trusted]
pub(crate) const fn decode_unix_timestamp_millis(_uuid: &Uuid) -> u64 {
    0
}

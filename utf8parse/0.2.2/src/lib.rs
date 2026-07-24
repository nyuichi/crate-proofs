//! A table-driven UTF-8 Parser
//!
//! This module implements a table-driven UTF-8 parser which should
//! theoretically contain the minimal number of branches (1). The only branch is
//! on the `Action` returned from unpacking a transition.
#![deny(clippy::all, clippy::if_not_else, clippy::enum_glob_use)]
#![cfg_attr(all(feature = "nightly", test), feature(test))]
#![no_std]

extern crate creusot_std;

#[allow(unused_imports)]
use creusot_std::prelude::{
    bitwise_proof, ensures, extern_spec, logic, pearlite, requires, trusted, DeepModel, Int,
    Invariant, View,
};

use core::char;

mod types;

#[cfg(creusot)]
use types::transition;
use types::{Action, State};

/// Handles codepoint and invalid sequence events from the parser.
pub trait Receiver {
    /// Called whenever a codepoint is parsed successfully
    fn codepoint(&mut self, _: char);

    /// Called when an invalid_sequence is detected
    fn invalid_sequence(&mut self);
}

/// A parser for Utf8 Characters
///
/// Repeatedly call `advance` with bytes to emit Utf8 characters
#[cfg_attr(not(creusot), derive(Clone, Default, PartialEq, Eq, Debug))]
pub struct Parser {
    #[cfg(creusot)]
    pub point: u32,
    #[cfg(not(creusot))]
    point: u32,
    #[cfg(creusot)]
    pub state: State,
    #[cfg(not(creusot))]
    state: State,
}

#[cfg(creusot)]
impl Clone for Parser {
    #[ensures(result@ == self@)]
    fn clone(&self) -> Self {
        Self {
            point: self.point,
            state: self.state,
        }
    }
}

#[cfg(creusot)]
impl Default for Parser {
    #[ensures(result@ == (0u32, 0))]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(creusot)]
impl PartialEq for Parser {
    // Verification-only replacement for the unsupported derived equality VC.
    #[trusted]
    #[ensures(result == (self.deep_model() == other.deep_model()))]
    fn eq(&self, other: &Self) -> bool {
        self.point == other.point && self.state == other.state
    }
}

#[cfg(creusot)]
impl Eq for Parser {}

impl View for Parser {
    type ViewTy = (u32, Int);

    #[logic(open)]
    fn view(self) -> Self::ViewTy {
        pearlite! { (self.point, self.state@) }
    }
}

impl DeepModel for Parser {
    type DeepModelTy = (Int, Int);

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { (self.point@, self.state@) }
    }
}

impl Invariant for Parser {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! { parser_invariant_model((self.point, self.state@)) }
    }
}

/// Reachable range constraints for each table state.
#[logic(open)]
pub fn parser_invariant_model(parser: (u32, Int)) -> bool {
    pearlite! {
        (parser.1 == 0 ==> parser.0@ == 0)
        && (parser.1 == 4 ==> parser.0@ == 0)
        && (parser.1 == 5 ==> parser.0@ == 0xd000)
        && (parser.1 == 6 ==> parser.0@ == 0)
        && (parser.1 == 7 ==> parser.0@ == 0x100000)
        && (parser.1 == 1 ==> 0x40000 <= parser.0@ && parser.0@ <= 0xc0000)
        && (parser.1 == 2 ==> (0x1000 <= parser.0@ && parser.0@ <= 0xc000)
            || (0xe000 <= parser.0@ && parser.0@ <= 0xf000)
            || (0x10000 <= parser.0@ && parser.0@ <= 0x10f000))
        && (parser.1 == 3 ==> (0x80 <= parser.0@ && parser.0@ <= 0xcfc0)
            || (0xd000 <= parser.0@ && parser.0@ <= 0xd7c0)
            || (0xe000 <= parser.0@ && parser.0@ <= 0x10ffc0))
    }
}

/// Continuation bytes are masked with this value.
const CONTINUATION_MASK: u8 = 0b0011_1111;

extern_spec! {
    mod core {
        mod char {
            #[requires(point@ <= 0x10ffff)]
            #[requires(point@ < 0xd800 || 0xdfff < point@)]
            #[ensures(result@ == point@)]
            unsafe fn from_u32_unchecked(point: u32) -> char;
        }
    }
}

/// Exact accumulator update associated with an action tag.
#[logic(open)]
pub fn point_after(point: u32, byte: u8, action: Int) -> u32 {
    if action == 0 || action == 2 {
        0u32
    } else if action == 1 {
        point
    } else if action == 3 {
        pearlite! { point | (((byte & 0b0011_1111u8) as u32) << 6u8) }
    } else if action == 4 {
        pearlite! { point | (((byte & 0b0001_1111u8) as u32) << 6u8) }
    } else if action == 5 {
        pearlite! { point | (((byte & 0b0011_1111u8) as u32) << 12u8) }
    } else if action == 6 {
        pearlite! { point | (((byte & 0b0000_1111u8) as u32) << 12u8) }
    } else {
        pearlite! { point | (((byte & 0b0000_0111u8) as u32) << 18u8) }
    }
}

/// Scalar value completed by the final continuation byte.
#[logic(open)]
pub fn completed_point(point: u32, byte: u8) -> u32 {
    pearlite! { point | ((byte & 0b0011_1111u8) as u32) }
}

/// Exact parser-state update for one byte, independent of receiver effects.
#[logic(open)]
pub fn parser_step(parser: (u32, Int), byte: u8) -> (u32, Int) {
    pearlite! {
        let next = transition(parser.1, byte@);
        (point_after(parser.0, byte, next.1), next.0)
    }
}

#[cfg(creusot)]
#[bitwise_proof]
#[requires(parser_invariant_model((point, state@)))]
#[ensures(parser_invariant_model(parser_step((point, state@), byte)))]
fn parser_step_preserves(point: u32, state: State, byte: u8) {
    match state {
        State::Ground
        | State::Tail3
        | State::Tail2
        | State::Tail1
        | State::U3_2_e0
        | State::U3_2_ed
        | State::Utf8_4_3_f0
        | State::Utf8_4_3_f4 => {}
    }
}

impl Parser {
    /// Create a new Parser
    #[ensures(result@ == (0u32, 0))]
    pub fn new() -> Parser {
        Parser {
            point: 0,
            state: State::Ground,
        }
    }

    /// Advance the parser
    ///
    /// The provider receiver will be called whenever a codepoint is completed or an invalid
    /// sequence is detected.
    #[bitwise_proof]
    #[ensures((^self)@ == parser_step((*self)@, byte))]
    pub fn advance<R>(&mut self, receiver: &mut R, byte: u8)
    where
        R: Receiver,
    {
        #[cfg(creusot)]
        parser_step_preserves(self.point, self.state, byte);
        let (state, action) = self.state.advance(byte);
        Self::perform_action(&mut self.point, receiver, byte, action);
        self.state = state;
    }

    #[requires(action@ != 2 || completed_point(*point, byte)@ <= 0x10ffff)]
    #[requires(action@ != 2 || completed_point(*point, byte)@ < 0xd800
        || 0xdfff < completed_point(*point, byte)@)]
    #[bitwise_proof]
    #[ensures(^point == point_after(*point, byte, action@))]
    fn perform_action<R>(point: &mut u32, receiver: &mut R, byte: u8, action: Action)
    where
        R: Receiver,
    {
        match action {
            Action::InvalidSequence => {
                *point = 0;
                receiver.invalid_sequence();
            }
            Action::EmitByte => {
                receiver.codepoint(byte as char);
            }
            Action::SetByte1 => {
                let completed = *point | ((byte & CONTINUATION_MASK) as u32);
                let c = unsafe { char::from_u32_unchecked(completed) };
                *point = 0;

                receiver.codepoint(c);
            }
            Action::SetByte2 => {
                *point |= ((byte & CONTINUATION_MASK) as u32) << 6;
            }
            Action::SetByte2Top => {
                *point |= ((byte & 0b0001_1111) as u32) << 6;
            }
            Action::SetByte3 => {
                *point |= ((byte & CONTINUATION_MASK) as u32) << 12;
            }
            Action::SetByte3Top => {
                *point |= ((byte & 0b0000_1111) as u32) << 12;
            }
            Action::SetByte4 => {
                *point |= ((byte & 0b0000_0111) as u32) << 18;
            }
        }
    }
}

#[cfg(all(feature = "nightly", test))]
mod benches {
    extern crate std;
    extern crate test;

    use super::{Parser, Receiver};

    use self::test::{black_box, Bencher};

    static UTF8_DEMO: &[u8] = include_bytes!("../tests/UTF-8-demo.txt");

    impl Receiver for () {
        fn codepoint(&mut self, c: char) {
            black_box(c);
        }

        fn invalid_sequence(&mut self) {}
    }

    #[bench]
    fn parse_bench_utf8_demo(b: &mut Bencher) {
        let mut parser = Parser::new();

        b.iter(|| {
            for byte in UTF8_DEMO {
                parser.advance(&mut (), *byte);
            }
        })
    }

    #[bench]
    fn std_string_parse_utf8(b: &mut Bencher) {
        b.iter(|| {
            for c in std::str::from_utf8(UTF8_DEMO).unwrap().chars() {
                black_box(c);
            }
        });
    }
}

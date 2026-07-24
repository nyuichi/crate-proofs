//! Types supporting the UTF-8 parser

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, trusted, DeepModel, Int, View};

/// Action to take when receiving a byte
#[cfg_attr(not(creusot), derive(Debug, Copy, Clone))]
pub enum Action {
    /// Unexpected byte; sequence is invalid
    InvalidSequence = 0,
    /// Received valid 7-bit ASCII byte which can be directly emitted.
    EmitByte = 1,
    /// Set the bottom continuation byte
    SetByte1 = 2,
    /// Set the 2nd-from-last continuation byte
    SetByte2 = 3,
    /// Set the 2nd-from-last byte which is part of a two byte sequence
    SetByte2Top = 4,
    /// Set the 3rd-from-last continuation byte
    SetByte3 = 5,
    /// Set the 3rd-from-last byte which is part of a three byte sequence
    SetByte3Top = 6,
    /// Set the top byte of a four byte sequence.
    SetByte4 = 7,
}

#[cfg(creusot)]
impl Clone for Action {
    #[ensures(result@ == self@)]
    fn clone(&self) -> Self {
        *self
    }
}

#[cfg(creusot)]
impl Copy for Action {}

impl View for Action {
    type ViewTy = Int;

    #[logic(open)]
    fn view(self) -> Int {
        pearlite! { match self {
            Self::InvalidSequence => 0,
            Self::EmitByte => 1,
            Self::SetByte1 => 2,
            Self::SetByte2 => 3,
            Self::SetByte2Top => 4,
            Self::SetByte3 => 5,
            Self::SetByte3Top => 6,
            Self::SetByte4 => 7,
        } }
    }
}

/// States the parser can be in.
///
/// There is a state for each initial input of the 3 and 4 byte sequences since
/// the following bytes are subject to different conditions than a tail byte.
#[allow(non_camel_case_types)]
#[cfg_attr(not(creusot), derive(Debug, Default, Copy, Clone, PartialEq, Eq))]
pub enum State {
    /// Ground state; expect anything
    #[cfg_attr(not(creusot), default)]
    Ground = 0,
    /// 3 tail bytes
    Tail3 = 1,
    /// 2 tail bytes
    Tail2 = 2,
    /// 1 tail byte
    Tail1 = 3,
    /// UTF8-3 starting with E0
    U3_2_e0 = 4,
    /// UTF8-3 starting with ED
    U3_2_ed = 5,
    /// UTF8-4 starting with F0
    Utf8_4_3_f0 = 6,
    /// UTF8-4 starting with F4
    Utf8_4_3_f4 = 7,
}

#[cfg(creusot)]
impl Clone for State {
    #[ensures(result@ == self@)]
    fn clone(&self) -> Self {
        *self
    }
}

#[cfg(creusot)]
impl Copy for State {}

#[cfg(creusot)]
impl Default for State {
    #[ensures(result@ == 0)]
    fn default() -> Self {
        Self::Ground
    }
}

#[cfg(creusot)]
impl PartialEq for State {
    // Verification-only replacement for the unsupported derived equality VC.
    #[trusted]
    #[ensures(result == (self.deep_model() == other.deep_model()))]
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Ground, Self::Ground)
                | (Self::Tail3, Self::Tail3)
                | (Self::Tail2, Self::Tail2)
                | (Self::Tail1, Self::Tail1)
                | (Self::U3_2_e0, Self::U3_2_e0)
                | (Self::U3_2_ed, Self::U3_2_ed)
                | (Self::Utf8_4_3_f0, Self::Utf8_4_3_f0)
                | (Self::Utf8_4_3_f4, Self::Utf8_4_3_f4)
        )
    }
}

#[cfg(creusot)]
impl Eq for State {}

impl View for State {
    type ViewTy = Int;

    #[logic(open)]
    fn view(self) -> Int {
        pearlite! { match self {
            Self::Ground => 0,
            Self::Tail3 => 1,
            Self::Tail2 => 2,
            Self::Tail1 => 3,
            Self::U3_2_e0 => 4,
            Self::U3_2_ed => 5,
            Self::Utf8_4_3_f0 => 6,
            Self::Utf8_4_3_f4 => 7,
        } }
    }
}

impl DeepModel for State {
    type DeepModelTy = Int;

    #[logic]
    fn deep_model(self) -> Int {
        pearlite! { self@ }
    }
}

/// Exact table transition, represented as `(next state, action)` tags.
#[logic(open)]
pub fn transition(state: Int, byte: Int) -> (Int, Int) {
    if state == 0 {
        if byte <= 0x7f {
            (0, 1)
        } else if 0xc2 <= byte && byte <= 0xdf {
            (3, 4)
        } else if byte == 0xe0 {
            (4, 6)
        } else if 0xe1 <= byte && byte <= 0xec {
            (2, 6)
        } else if byte == 0xed {
            (5, 6)
        } else if 0xee <= byte && byte <= 0xef {
            (2, 6)
        } else if byte == 0xf0 {
            (6, 7)
        } else if 0xf1 <= byte && byte <= 0xf3 {
            (1, 7)
        } else if byte == 0xf4 {
            (7, 7)
        } else {
            (0, 0)
        }
    } else if state == 4 {
        if 0xa0 <= byte && byte <= 0xbf {
            (3, 3)
        } else {
            (0, 0)
        }
    } else if state == 5 {
        if 0x80 <= byte && byte <= 0x9f {
            (3, 3)
        } else {
            (0, 0)
        }
    } else if state == 6 {
        if 0x90 <= byte && byte <= 0xbf {
            (2, 5)
        } else {
            (0, 0)
        }
    } else if state == 7 {
        if 0x80 <= byte && byte <= 0x8f {
            (2, 5)
        } else {
            (0, 0)
        }
    } else if state == 1 {
        if 0x80 <= byte && byte <= 0xbf {
            (2, 5)
        } else {
            (0, 0)
        }
    } else if state == 2 {
        if 0x80 <= byte && byte <= 0xbf {
            (3, 3)
        } else {
            (0, 0)
        }
    } else {
        if 0x80 <= byte && byte <= 0xbf {
            (0, 2)
        } else {
            (0, 0)
        }
    }
}

impl State {
    /// Advance the parser state.
    ///
    /// This takes the current state and input byte into consideration, to determine the next state
    /// and any action that should be taken.
    #[inline]
    #[ensures((result.0@, result.1@) == transition(self@, byte@))]
    pub fn advance(self, byte: u8) -> (State, Action) {
        match self {
            State::Ground => match byte {
                0x00..=0x7f => (State::Ground, Action::EmitByte),
                0xc2..=0xdf => (State::Tail1, Action::SetByte2Top),
                0xe0 => (State::U3_2_e0, Action::SetByte3Top),
                0xe1..=0xec => (State::Tail2, Action::SetByte3Top),
                0xed => (State::U3_2_ed, Action::SetByte3Top),
                0xee..=0xef => (State::Tail2, Action::SetByte3Top),
                0xf0 => (State::Utf8_4_3_f0, Action::SetByte4),
                0xf1..=0xf3 => (State::Tail3, Action::SetByte4),
                0xf4 => (State::Utf8_4_3_f4, Action::SetByte4),
                _ => (State::Ground, Action::InvalidSequence),
            },
            State::U3_2_e0 => match byte {
                0xa0..=0xbf => (State::Tail1, Action::SetByte2),
                _ => (State::Ground, Action::InvalidSequence),
            },
            State::U3_2_ed => match byte {
                0x80..=0x9f => (State::Tail1, Action::SetByte2),
                _ => (State::Ground, Action::InvalidSequence),
            },
            State::Utf8_4_3_f0 => match byte {
                0x90..=0xbf => (State::Tail2, Action::SetByte3),
                _ => (State::Ground, Action::InvalidSequence),
            },
            State::Utf8_4_3_f4 => match byte {
                0x80..=0x8f => (State::Tail2, Action::SetByte3),
                _ => (State::Ground, Action::InvalidSequence),
            },
            State::Tail3 => match byte {
                0x80..=0xbf => (State::Tail2, Action::SetByte3),
                _ => (State::Ground, Action::InvalidSequence),
            },
            State::Tail2 => match byte {
                0x80..=0xbf => (State::Tail1, Action::SetByte2),
                _ => (State::Ground, Action::InvalidSequence),
            },
            State::Tail1 => match byte {
                0x80..=0xbf => (State::Ground, Action::SetByte1),
                _ => (State::Ground, Action::InvalidSequence),
            },
        }
    }
}

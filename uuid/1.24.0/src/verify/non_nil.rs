use crate::Uuid;
use creusot_std::prelude::{ensures, logic, pearlite, Invariant, Seq, View};

/// A UUID statically known not to be nil.
#[derive(Copy, Clone)]
pub struct NonNilUuid(Uuid);

impl View for NonNilUuid {
    type ViewTy = Seq<u8>;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { self.0@ }
    }
}

impl Invariant for NonNilUuid {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! { self@.len() == 16 }
    }
}

impl NonNilUuid {
    /// Creates a non-nil UUID when the input is not nil.
    pub const fn new(uuid: Uuid) -> Option<Self> {
        if uuid.is_nil() {
            None
        } else {
            Some(Self(uuid))
        }
    }

    /// Returns the wrapped UUID.
    #[ensures(result@ == self@)]
    pub const fn get(self) -> Uuid {
        self.0
    }
}

impl From<NonNilUuid> for Uuid {
    fn from(value: NonNilUuid) -> Self {
        value.get()
    }
}

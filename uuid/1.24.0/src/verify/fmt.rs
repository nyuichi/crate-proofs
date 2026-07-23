#![allow(missing_docs)]

use crate::Uuid;
use creusot_std::prelude::{ensures, logic, DeepModel, Int, Seq};

macro_rules! format_adapter {
    ($name:ident, $length:expr) => {
        #[derive(Clone, Copy)]
        #[repr(transparent)]
        pub struct $name(Uuid);

        impl $name {
            pub const LENGTH: usize = $length;

            #[ensures(result.deep_model() == uuid.deep_model())]
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            #[ensures(result.deep_model() == self.deep_model())]
            pub const fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            #[ensures(result.deep_model() == self.deep_model())]
            pub const fn into_uuid(self) -> Uuid {
                self.0
            }
        }

        impl DeepModel for $name {
            type DeepModelTy = Seq<Int>;

            #[logic]
            fn deep_model(self) -> Self::DeepModelTy {
                self.0.deep_model()
            }
        }
    };
}

format_adapter!(Hyphenated, 36);
format_adapter!(Simple, 32);
format_adapter!(Urn, 45);
format_adapter!(Braced, 38);

impl Uuid {
    #[ensures(result.deep_model() == self.deep_model())]
    pub const fn hyphenated(self) -> Hyphenated {
        Hyphenated(self)
    }

    #[ensures(result.deep_model() == self.deep_model())]
    pub const fn simple(self) -> Simple {
        Simple(self)
    }

    #[ensures(result.deep_model() == self.deep_model())]
    pub const fn urn(self) -> Urn {
        Urn(self)
    }

    #[ensures(result.deep_model() == self.deep_model())]
    pub const fn braced(self) -> Braced {
        Braced(self)
    }
}

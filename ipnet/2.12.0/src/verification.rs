//! Creusot-facing structural model.
//!
//! The pinned Creusot library has no logical model for `core::net` address
//! values.  This module therefore verifies the prefix-length state machine
//! without claiming an address-content or mask-arithmetic proof.  The normal
//! build continues to compile the unmodified upstream implementation.

use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use creusot_std::prelude::{ensures, logic, requires, DeepModel, Int, Invariant, View};

pub struct PrefixLenError;

pub enum IpNet {
    V4(Ipv4Net),
    V6(Ipv6Net),
}

pub struct Ipv4Net {
    addr: Ipv4Addr,
    prefix_len: u8,
}

pub struct Ipv6Net {
    addr: Ipv6Addr,
    prefix_len: u8,
}

impl View for Ipv4Net {
    type ViewTy = Int;

    #[logic]
    fn view(self) -> Int {
        self.prefix_len.deep_model()
    }
}

impl Invariant for Ipv4Net {
    #[logic(open, prophetic)]
    fn invariant(self) -> bool {
        pearlite! { self@ <= 32 }
    }
}

impl View for Ipv6Net {
    type ViewTy = Int;

    #[logic]
    fn view(self) -> Int {
        self.prefix_len.deep_model()
    }
}

impl Invariant for Ipv6Net {
    #[logic(open, prophetic)]
    fn invariant(self) -> bool {
        pearlite! { self@ <= 128 }
    }
}

impl Invariant for IpNet {
    #[logic(open, prophetic)]
    fn invariant(self) -> bool {
        match self {
            IpNet::V4(net) => net.invariant(),
            IpNet::V6(net) => net.invariant(),
        }
    }
}

impl Ipv4Net {
    #[ensures(match result { Ok(net) => net@ == prefix_len@, Err(_) => prefix_len@ > 32 })]
    pub fn new(addr: Ipv4Addr, prefix_len: u8) -> Result<Self, PrefixLenError> {
        if prefix_len > 32 {
            Err(PrefixLenError)
        } else {
            Ok(Self { addr, prefix_len })
        }
    }

    #[ensures(result@ == prefix_len@)]
    #[requires(prefix_len@ <= 32)]
    pub fn new_assert(addr: Ipv4Addr, prefix_len: u8) -> Self {
        assert!(prefix_len <= 32);
        Self { addr, prefix_len }
    }

    pub fn addr(&self) -> Ipv4Addr {
        self.addr
    }

    #[ensures(result@ == self@)]
    pub fn prefix_len(&self) -> u8 {
        self.prefix_len
    }

    #[ensures(result@ == 32)]
    pub fn max_prefix_len(&self) -> u8 {
        32
    }
}

impl Ipv6Net {
    #[ensures(match result { Ok(net) => net@ == prefix_len@, Err(_) => prefix_len@ > 128 })]
    pub fn new(addr: Ipv6Addr, prefix_len: u8) -> Result<Self, PrefixLenError> {
        if prefix_len > 128 {
            Err(PrefixLenError)
        } else {
            Ok(Self { addr, prefix_len })
        }
    }

    #[ensures(result@ == prefix_len@)]
    #[requires(prefix_len@ <= 128)]
    pub fn new_assert(addr: Ipv6Addr, prefix_len: u8) -> Self {
        assert!(prefix_len <= 128);
        Self { addr, prefix_len }
    }

    pub fn addr(&self) -> Ipv6Addr {
        self.addr
    }

    #[ensures(result@ == self@)]
    pub fn prefix_len(&self) -> u8 {
        self.prefix_len
    }

    #[ensures(result@ == 128)]
    pub fn max_prefix_len(&self) -> u8 {
        128
    }
}

impl IpNet {
    #[ensures(match result {
        Ok(IpNet::V4(net)) => net@ == prefix_len@,
        Ok(IpNet::V6(net)) => net@ == prefix_len@,
        Err(_) => prefix_len@ > 32,
    })]
    pub fn new(addr: IpAddr, prefix_len: u8) -> Result<Self, PrefixLenError> {
        match addr {
            IpAddr::V4(addr) => match Ipv4Net::new(addr, prefix_len) {
                Ok(net) => Ok(IpNet::V4(net)),
                Err(err) => Err(err),
            },
            IpAddr::V6(addr) => match Ipv6Net::new(addr, prefix_len) {
                Ok(net) => Ok(IpNet::V6(net)),
                Err(err) => Err(err),
            },
        }
    }

    #[ensures(match result {
        IpNet::V4(net) => net@ == prefix_len@,
        IpNet::V6(net) => net@ == prefix_len@,
    })]
    #[requires(match addr {
        IpAddr::V4(_) => prefix_len@ <= 32,
        IpAddr::V6(_) => prefix_len@ <= 128,
    })]
    pub fn new_assert(addr: IpAddr, prefix_len: u8) -> Self {
        match addr {
            IpAddr::V4(addr) => IpNet::V4(Ipv4Net::new_assert(addr, prefix_len)),
            IpAddr::V6(addr) => IpNet::V6(Ipv6Net::new_assert(addr, prefix_len)),
        }
    }

    #[ensures(match (self, result) {
        (IpNet::V4(_), IpAddr::V4(_)) => true,
        (IpNet::V6(_), IpAddr::V6(_)) => true,
        _ => false,
    })]
    pub fn addr(&self) -> IpAddr {
        match self {
            IpNet::V4(net) => IpAddr::V4(net.addr()),
            IpNet::V6(net) => IpAddr::V6(net.addr()),
        }
    }

    #[ensures(match self {
        IpNet::V4(net) => result@ == net@,
        IpNet::V6(net) => result@ == net@,
    })]
    pub fn prefix_len(&self) -> u8 {
        match self {
            IpNet::V4(net) => net.prefix_len(),
            IpNet::V6(net) => net.prefix_len(),
        }
    }

    #[ensures(match self {
        IpNet::V4(_) => result@ == 32,
        IpNet::V6(_) => result@ == 128,
    })]
    pub fn max_prefix_len(&self) -> u8 {
        match self {
            IpNet::V4(_) => 32,
            IpNet::V6(_) => 128,
        }
    }
}

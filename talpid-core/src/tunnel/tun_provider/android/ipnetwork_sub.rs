use ipnetwork::{Ipv4Network, Ipv6Network};
use std::{
    fmt::Debug,
    ops::{Add, BitAnd, BitXor, Not, Shl, Sub},
};

pub trait AbstractIpNetwork: Clone + Copy + 'static {
    type Representation: Add<Output = Self::Representation>
        + BitAnd<Output = Self::Representation>
        + BitXor<Output = Self::Representation>
        + Clone
        + Copy
        + Debug
        + PartialEq
        + Not<Output = Self::Representation>
        + Shl<u8, Output = Self::Representation>
        + Sub<Output = Self::Representation>;

    const ZERO: Self::Representation;
    const ONE: Self::Representation;
    const MAX_PREFIX: u8;

    fn new(network: Self::Representation, prefix: u8) -> Self;
    fn mask(self) -> Self::Representation;
    fn network(self) -> Self::Representation;
    fn prefix(self) -> u8;
}

impl AbstractIpNetwork for Ipv4Network {
    type Representation = u32;

    const ZERO: Self::Representation = 0;
    const ONE: Self::Representation = 1;
    const MAX_PREFIX: u8 = 32;

    fn new(network: Self::Representation, prefix: u8) -> Self {
        Ipv4Network::new(network.into(), prefix).expect("Invalid IPv4 network prefix")
    }

    fn mask(self) -> Self::Representation {
        Ipv4Network::mask(&self).into()
    }

    fn network(self) -> Self::Representation {
        Ipv4Network::network(&self).into()
    }

    fn prefix(self) -> u8 {
        Ipv4Network::prefix(&self)
    }
}

impl AbstractIpNetwork for Ipv6Network {
    type Representation = u128;

    const ZERO: Self::Representation = 0;
    const ONE: Self::Representation = 1;
    const MAX_PREFIX: u8 = 128;

    fn new(network: Self::Representation, prefix: u8) -> Self {
        Ipv6Network::new(network.into(), prefix).expect("Invalid IPv6 network prefix")
    }

    fn mask(self) -> Self::Representation {
        Ipv6Network::mask(&self).into()
    }

    fn network(self) -> Self::Representation {
        Ipv6Network::network(&self).into()
    }

    fn prefix(self) -> u8 {
        Ipv6Network::prefix(&self)
    }
}

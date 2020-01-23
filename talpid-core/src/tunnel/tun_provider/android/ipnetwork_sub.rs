use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use std::{
    fmt::Debug,
    iter,
    marker::PhantomData,
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

#[derive(Clone, Copy, Debug)]
pub struct IpNetworkRange<T: AbstractIpNetwork> {
    network: T::Representation,
    bit_position: u8,
    max_bit_position: u8,
    _network_type: PhantomData<T>,
}

impl<T> Iterator for IpNetworkRange<T>
where
    T: AbstractIpNetwork,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bit_position < self.max_bit_position {
            let bit_mask = T::ONE << self.bit_position;
            let prefix_mask = !(bit_mask - T::ONE);
            let address = (self.network ^ bit_mask) & prefix_mask;
            let prefix = T::MAX_PREFIX - self.bit_position;

            self.bit_position += 1;

            Some(T::new(address, prefix))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum IpNetworks<T: AbstractIpNetwork> {
    Empty,
    SingleNetwork(T),
    MultipleNetworks(IpNetworkRange<T>),
}

impl<T> Iterator for IpNetworks<T>
where
    T: AbstractIpNetwork,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IpNetworks::Empty => None,
            &mut IpNetworks::SingleNetwork(network) => {
                *self = IpNetworks::Empty;
                Some(network)
            }
            IpNetworks::MultipleNetworks(range) => {
                if let Some(item) = range.next() {
                    Some(item)
                } else {
                    *self = IpNetworks::Empty;
                    None
                }
            }
        }
    }
}

pub trait IpNetworkSub: Copy + Sized + 'static {
    type Output: Iterator<Item = Self>;

    fn sub(self, other: Self) -> Self::Output;

    fn sub_all(self, others: impl IntoIterator<Item = Self>) -> Box<dyn Iterator<Item = Self>> {
        let mut result: Box<dyn Iterator<Item = Self>> = Box::new(iter::once(self));

        for other in others {
            result = Box::new(result.flat_map(move |network| network.sub(other)));
        }

        result
    }
}

impl<T> IpNetworkSub for T
where
    T: AbstractIpNetwork,
{
    type Output = IpNetworks<T>;

    fn sub(self, other: Self) -> Self::Output {
        let subtrahend = self.network();
        let minuend = other.network();
        let mask = self.mask();

        if minuend & mask == subtrahend {
            let max_bit_position = T::MAX_PREFIX - self.prefix();
            let bit_position = T::MAX_PREFIX - other.prefix();

            IpNetworks::MultipleNetworks(IpNetworkRange {
                network: minuend,
                bit_position,
                max_bit_position,
                _network_type: PhantomData,
            })
        } else {
            let other_mask = other.mask();

            if subtrahend & other_mask == minuend {
                IpNetworks::Empty
            } else {
                IpNetworks::SingleNetwork(self)
            }
        }
    }
}

#[derive(Debug)]
pub enum IpNetworkIterator {
    V4(IpNetworks<Ipv4Network>),
    V6(IpNetworks<Ipv6Network>),
}

impl Iterator for IpNetworkIterator {
    type Item = IpNetwork;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IpNetworkIterator::V4(iterator) => iterator.next().map(IpNetwork::V4),
            IpNetworkIterator::V6(iterator) => iterator.next().map(IpNetwork::V6),
        }
    }
}

impl IpNetworkSub for IpNetwork {
    type Output = IpNetworkIterator;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (IpNetwork::V4(self_v4), IpNetwork::V4(other_v4)) => {
                IpNetworkIterator::V4(self_v4.sub(other_v4))
            }
            (IpNetwork::V6(self_v6), IpNetwork::V6(other_v6)) => {
                IpNetworkIterator::V6(self_v6.sub(other_v6))
            }
            (IpNetwork::V4(_), IpNetwork::V6(_)) => {
                panic!("Can't remove IPv6 network from IPv4 network")
            }
            (IpNetwork::V6(_), IpNetwork::V4(_)) => {
                panic!("Can't remove IPv4 network from IPv6 network")
            }
        }
    }
}

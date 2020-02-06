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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::HashSet,
        net::{IpAddr, Ipv4Addr},
    };

    #[test]
    fn subtract_out_of_range() {
        let minuend = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(25, 0, 0, 0)), 8).unwrap();
        let subtrahend = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(125, 92, 4, 0)), 24).unwrap();

        let difference: Vec<_> = minuend.sub(subtrahend).collect();

        let expected = vec![minuend];

        assert_eq!(difference, expected);
    }

    #[test]
    fn subtract_whole_range() {
        let minuend = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(25, 0, 0, 0)), 8).unwrap();
        let subtrahend = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(16, 0, 0, 0)), 4).unwrap();

        let difference: Vec<_> = minuend.sub(subtrahend).collect();

        assert!(difference.is_empty());
    }

    #[test]
    fn subtract_inner_range() {
        let minuend = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)), 8).unwrap();
        let subtrahend = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(10, 10, 10, 0)), 24).unwrap();

        let difference: HashSet<_> = minuend.sub(subtrahend).collect();

        let expected = vec![
            ([10, 0, 0, 0], 13),
            ([10, 8, 0, 0], 15),
            ([10, 10, 0, 0], 21),
            ([10, 10, 8, 0], 23),
            ([10, 10, 11, 0], 24),
            ([10, 10, 12, 0], 22),
            ([10, 10, 16, 0], 20),
            ([10, 10, 32, 0], 19),
            ([10, 10, 64, 0], 18),
            ([10, 10, 128, 0], 17),
            ([10, 11, 0, 0], 16),
            ([10, 12, 0, 0], 14),
            ([10, 16, 0, 0], 12),
            ([10, 32, 0, 0], 11),
            ([10, 64, 0, 0], 10),
            ([10, 128, 0, 0], 9),
        ];

        let expected: HashSet<_> = expected
            .into_iter()
            .map(|(octets, prefix)| IpNetwork::new(IpAddr::V4(octets.into()), prefix).unwrap())
            .collect();

        assert_eq!(difference, expected);
    }

    #[test]
    fn subtract_single_address() {
        let minuend = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(10, 64, 0, 0)), 10).unwrap();
        let subtrahend = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(10, 64, 0, 0)), 32).unwrap();

        let difference: HashSet<_> = minuend.sub(subtrahend).collect();

        let expected = vec![
            ([10, 64, 0, 1], 32),
            ([10, 64, 0, 2], 31),
            ([10, 64, 0, 4], 30),
            ([10, 64, 0, 8], 29),
            ([10, 64, 0, 16], 28),
            ([10, 64, 0, 32], 27),
            ([10, 64, 0, 64], 26),
            ([10, 64, 0, 128], 25),
            ([10, 64, 1, 0], 24),
            ([10, 64, 2, 0], 23),
            ([10, 64, 4, 0], 22),
            ([10, 64, 8, 0], 21),
            ([10, 64, 16, 0], 20),
            ([10, 64, 32, 0], 19),
            ([10, 64, 64, 0], 18),
            ([10, 64, 128, 0], 17),
            ([10, 65, 0, 0], 16),
            ([10, 66, 0, 0], 15),
            ([10, 68, 0, 0], 14),
            ([10, 72, 0, 0], 13),
            ([10, 80, 0, 0], 12),
            ([10, 96, 0, 0], 11),
        ];

        let expected: HashSet<_> = expected
            .into_iter()
            .map(|(octets, prefix)| IpNetwork::new(IpAddr::V4(octets.into()), prefix).unwrap())
            .collect();

        assert_eq!(difference, expected);
    }

    #[test]
    fn subtract_multiple() {
        let minuend = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0).unwrap();
        let subtrahend_1 = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)), 8).unwrap();
        let subtrahend_2 = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 0)), 12).unwrap();
        let subtrahend_3 = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0)), 16).unwrap();
        let subtrahend_4 = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(169, 254, 0, 0)), 16).unwrap();
        let subtrahend_5 = IpNetwork::new(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 0)), 24).unwrap();
        let subtrahend_6 =
            IpNetwork::new(IpAddr::V4(Ipv4Addr::new(239, 255, 255, 250)), 32).unwrap();
        let subtrahend_7 =
            IpNetwork::new(IpAddr::V4(Ipv4Addr::new(239, 255, 255, 251)), 32).unwrap();

        let difference: HashSet<_> = minuend
            .sub_all(vec![
                subtrahend_1,
                subtrahend_2,
                subtrahend_3,
                subtrahend_4,
                subtrahend_5,
                subtrahend_6,
                subtrahend_7,
            ])
            .collect();

        let expected = vec![
            ([0, 0, 0, 0], 5),
            ([8, 0, 0, 0], 7),
            ([11, 0, 0, 0], 8),
            ([12, 0, 0, 0], 6),
            ([16, 0, 0, 0], 4),
            ([32, 0, 0, 0], 3),
            ([64, 0, 0, 0], 2),
            ([128, 0, 0, 0], 3),
            ([160, 0, 0, 0], 5),
            ([168, 0, 0, 0], 8),
            ([169, 0, 0, 0], 9),
            ([169, 128, 0, 0], 10),
            ([169, 192, 0, 0], 11),
            ([169, 224, 0, 0], 12),
            ([169, 240, 0, 0], 13),
            ([169, 248, 0, 0], 14),
            ([169, 252, 0, 0], 15),
            ([169, 255, 0, 0], 16),
            ([170, 0, 0, 0], 7),
            ([172, 0, 0, 0], 12),
            ([172, 32, 0, 0], 11),
            ([172, 64, 0, 0], 10),
            ([172, 128, 0, 0], 9),
            ([173, 0, 0, 0], 8),
            ([174, 0, 0, 0], 7),
            ([176, 0, 0, 0], 4),
            ([192, 0, 0, 0], 9),
            ([192, 128, 0, 0], 11),
            ([192, 160, 0, 0], 13),
            ([192, 169, 0, 0], 16),
            ([192, 170, 0, 0], 15),
            ([192, 172, 0, 0], 14),
            ([192, 176, 0, 0], 12),
            ([192, 192, 0, 0], 10),
            ([193, 0, 0, 0], 8),
            ([194, 0, 0, 0], 7),
            ([196, 0, 0, 0], 6),
            ([200, 0, 0, 0], 5),
            ([208, 0, 0, 0], 4),
            ([224, 0, 1, 0], 24),
            ([224, 0, 2, 0], 23),
            ([224, 0, 4, 0], 22),
            ([224, 0, 8, 0], 21),
            ([224, 0, 16, 0], 20),
            ([224, 0, 32, 0], 19),
            ([224, 0, 64, 0], 18),
            ([224, 0, 128, 0], 17),
            ([224, 1, 0, 0], 16),
            ([224, 2, 0, 0], 15),
            ([224, 4, 0, 0], 14),
            ([224, 8, 0, 0], 13),
            ([224, 16, 0, 0], 12),
            ([224, 32, 0, 0], 11),
            ([224, 64, 0, 0], 10),
            ([224, 128, 0, 0], 9),
            ([225, 0, 0, 0], 8),
            ([226, 0, 0, 0], 7),
            ([228, 0, 0, 0], 6),
            ([232, 0, 0, 0], 6),
            ([236, 0, 0, 0], 7),
            ([238, 0, 0, 0], 8),
            ([239, 0, 0, 0], 9),
            ([239, 128, 0, 0], 10),
            ([239, 192, 0, 0], 11),
            ([239, 224, 0, 0], 12),
            ([239, 240, 0, 0], 13),
            ([239, 248, 0, 0], 14),
            ([239, 252, 0, 0], 15),
            ([239, 254, 0, 0], 16),
            ([239, 255, 0, 0], 17),
            ([239, 255, 128, 0], 18),
            ([239, 255, 192, 0], 19),
            ([239, 255, 224, 0], 20),
            ([239, 255, 240, 0], 21),
            ([239, 255, 248, 0], 22),
            ([239, 255, 252, 0], 23),
            ([239, 255, 254, 0], 24),
            ([239, 255, 255, 0], 25),
            ([239, 255, 255, 128], 26),
            ([239, 255, 255, 192], 27),
            ([239, 255, 255, 224], 28),
            ([239, 255, 255, 240], 29),
            ([239, 255, 255, 248], 31),
            ([239, 255, 255, 252], 30),
            ([240, 0, 0, 0], 4),
        ];

        let expected: HashSet<_> = expected
            .into_iter()
            .map(|(octets, prefix)| IpNetwork::new(IpAddr::V4(octets.into()), prefix).unwrap())
            .collect();

        assert_eq!(difference, expected);
    }
}

//! Parse and use various proxy configurations as they are retrieved via AAAA records, hopefully
//! served by DoH resolvers.
use std::{
    io::Cursor,
    net::{Ipv6Addr, SocketAddrV4},
};

use byteorder::{LittleEndian, ReadBytesExt};

mod plain;
mod xor;
pub use plain::Plain;
pub use xor::Xor;

/// An error that happens when parsing IPv6 addresses into proxy configurations.
#[derive(Debug)]
pub enum Error {
    /// IP address representing a Xor proxy was not valid
    InvalidXor(xor::Error),
    /// IP address representing the plain proxy was not valid
    InvalidPlain(plain::Error),
    /// IP addresses did not contain any valid proxy configuration
    NoProxies,
}

/// If a given IPv6 address does not contain a valid value for the proxy version, this error type
/// will contain the unrecognized value.
#[derive(Debug)]
pub struct ErrorUnknownType(u16);

/// Type of a proxy configuration. Derived from the 2nd hextet of an IPv6 address in network byte
/// order. E.g. an IPv6 address such as `7f7f:2323::`  would have a proxy type value of `0x2323`.
#[derive(PartialEq, Debug)]
#[repr(u16)]
enum ProxyType {
    Plain = 0x01,
    XorV1 = 0x02,
    XorV2 = 0x03,
}

impl TryFrom<Ipv6Addr> for ProxyType {
    type Error = ErrorUnknownType;

    fn try_from(value: Ipv6Addr) -> Result<Self, Self::Error> {
        let mut data = Cursor::new(value.octets());
        // skip the first 2 bytes since it's just padding to make the IP look more like a legit
        // IPv6 address.

        data.set_position(2);
        match data
            .read_u16::<LittleEndian>()
            .expect("IPv6 must have at least 16 bytes")
        {
            0x01 => Ok(Self::Plain),
            0x02 => Ok(Self::XorV1),
            0x03 => Ok(Self::XorV2),
            unknown => Err(ErrorUnknownType(unknown)),
        }
    }
}

/// Contains valid proxy configurations as derived from a set of IPv6 addresses.
pub struct AvailableProxies {
    /// Plain proxies just forward traffic without any obfuscation.
    pub plain: Vec<Plain>,
    /// Xor proxies xor a pre-shared key with all the traffic.
    pub xor: Vec<Xor>,
}

impl TryFrom<Vec<Ipv6Addr>> for AvailableProxies {
    type Error = Error;

    fn try_from(ips: Vec<Ipv6Addr>) -> Result<Self, Self::Error> {
        let mut proxies = AvailableProxies {
            plain: vec![],
            xor: vec![],
        };

        for ip in ips {
            match ProxyType::try_from(ip) {
                Ok(ProxyType::Plain) => {
                    proxies
                        .plain
                        .push(Plain::try_from(ip).map_err(Error::InvalidPlain)?);
                }
                Ok(ProxyType::XorV2) => {
                    proxies
                        .xor
                        .push(Xor::try_from(ip).map_err(Error::InvalidXor)?);
                }

                // V1 types are ignored and so are errors
                Ok(ProxyType::XorV1) => continue,

                Err(ErrorUnknownType(unknown_proxy_type)) => {
                    log::error!("Unknown proxy type {unknown_proxy_type}");
                }
            }
        }
        if proxies.plain.is_empty() && proxies.xor.is_empty() {
            return Err(Error::NoProxies);
        }

        Ok(proxies)
    }
}

/// A trait that can be used by a forwarder to forward traffic.
pub trait Obfuscator: Send {
    /// Provides the endpoint for the proxy. This address must be connected and all traffic to it
    /// should first be obfuscated with `Obfuscator::obfuscate`.
    fn addr(&self) -> SocketAddrV4;
    /// Applies obfuscation to a given buffer of bytes.
    fn obfuscate(&mut self, buffer: &mut [u8]);
    /// Constructs a new obfuscator of the same type and configuration, with it's internal state
    /// reset.
    fn clone(&self) -> Box<dyn Obfuscator>;
}

#[test]
fn wrong_proxy_type() {
    let addr: Ipv6Addr = "ffff:2345::".parse().unwrap();
    match ProxyType::try_from(addr) {
        Err(ErrorUnknownType(0x4523)) => (),
        anything_else => panic!("Expected unknown type 0x33, got {anything_else:x?}"),
    }
}

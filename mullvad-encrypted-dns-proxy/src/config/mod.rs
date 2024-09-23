//! Parse and use various proxy configurations as they are retrieved via AAAA records, hopefully
//! served by DoH resolvers.

use core::fmt;
use std::net::{Ipv6Addr, SocketAddrV4};

mod plain;
mod xor;

pub use xor::XorKey;

/// All the errors that can happen during deserialization of a [`ProxyConfig`].
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// The proxy type field has a value this library is not compatible with
    UnknownProxyType(u16),
    /// The XorV1 proxy type is deprecated and not supported
    XorV1Unsupported,
    /// The port is not valid
    InvalidPort(u16),
    /// The key to use for XOR obfuscation was empty (all zeros)
    EmptyXorKey,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownProxyType(t) => write!(f, "Unknown type of proxy: {t:#x}"),
            Self::XorV1Unsupported => write!(f, "XorV1 proxy types are not supported"),
            Self::InvalidPort(port) => write!(f, "Port {port} is not valid for remote endpoint"),
            Self::EmptyXorKey => write!(f, "The key material for XOR obfuscation is empty"),
        }
    }
}

impl std::error::Error for Error {}

/// Type of a proxy configuration. Derived from the 2nd hextet of an IPv6 address in network byte
/// order. E.g. an IPv6 address such as `7f7f:2323::`  would have a proxy type value of `0x2323`.
#[derive(PartialEq, Debug)]
enum ProxyType {
    /// Plain proxy type
    Plain,
    /// XorV1 - deprecated
    XorV1,
    /// XorV2
    XorV2,
}

impl TryFrom<[u8; 2]> for ProxyType {
    type Error = Error;

    fn try_from(bytes: [u8; 2]) -> Result<Self, Self::Error> {
        match u16::from_le_bytes(bytes) {
            0x01 => Ok(Self::Plain),
            0x02 => Ok(Self::XorV1),
            0x03 => Ok(Self::XorV2),
            unknown => Err(Error::UnknownProxyType(unknown)),
        }
    }
}

pub trait Obfuscator: Send {
    /// Applies obfuscation to a given buffer of bytes. Changes the data in place.
    fn obfuscate(&mut self, buffer: &mut [u8]);
}

/// Represents a Mullvad Encrypted DNS proxy configuration. Created by parsing
/// the config out of an IPv6 address resolved over DoH.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ProxyConfig {
    /// The remote address to connect to the proxy over. This is the address
    /// on the internet where the proxy is listening.
    pub addr: SocketAddrV4,
    /// If the proxy requires some obfuscation of the data sent to/received from it,
    /// it's represented by an obfuscation config here.
    pub obfuscation: Option<ObfuscationConfig>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ObfuscationConfig {
    XorV2(xor::XorKey),
}

impl ObfuscationConfig {
    /// Instantiate an obfuscator from the given obfuscation config.
    pub fn create_obfuscator(&self) -> Box<dyn Obfuscator> {
        match self {
            Self::XorV2(key) => Box::new(xor::XorObfuscator::new(*key)),
        }
    }
}

impl TryFrom<Ipv6Addr> for ProxyConfig {
    type Error = Error;

    fn try_from(ip: Ipv6Addr) -> Result<Self, Self::Error> {
        let data = ip.octets();

        let proxy_type_bytes = <[u8; 2]>::try_from(&data[2..4]).unwrap();
        let proxy_config_payload = <[u8; 12]>::try_from(&data[4..16]).unwrap();

        let proxy_type = ProxyType::try_from(proxy_type_bytes)?;

        match proxy_type {
            ProxyType::Plain => plain::parse_plain(proxy_config_payload),
            ProxyType::XorV1 => Err(Error::XorV1Unsupported),
            ProxyType::XorV2 => xor::parse_xor(proxy_config_payload),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv6Addr;

    use super::{Error, ProxyConfig};

    #[test]
    fn wrong_proxy_type() {
        let addr: Ipv6Addr = "ffff:2345::".parse().unwrap();
        match ProxyConfig::try_from(addr) {
            Err(Error::UnknownProxyType(0x4523)) => (),
            anything_else => panic!("Unexpected proxy config parse result: {anything_else:?}"),
        }
    }
}

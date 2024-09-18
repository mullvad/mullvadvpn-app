use std::{
    io::Cursor,
    net::{Ipv6Addr, SocketAddrV4},
};

use byteorder::{LittleEndian, ReadBytesExt};

mod plain;
mod xor;
pub use plain::Plain;
pub use xor::Xor;

#[derive(Debug)]
pub enum Error {
    UnknownType(u16),
    InvalidXor(xor::Error),
    InvalidPlain(plain::Error),
}

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
    type Error = Error;

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
            unknown => Err(Error::UnknownType(unknown)),
        }
    }
}

impl TryFrom<Vec<Ipv6Addr>> for AvailableProxies {
    type Error = Error;

    fn try_from(ips: Vec<Ipv6Addr>) -> Result<Self, Self::Error> {
        let mut proxies = AvailableProxies {
            plain: vec![],
            xor: vec![],
        };

        for ip in ips {
            match ProxyType::try_from(ip)? {
                ProxyType::Plain => {
                    proxies
                        .plain
                        .push(Plain::try_from(ip).map_err(Error::InvalidPlain)?);
                }
                ProxyType::XorV2 => {
                    proxies
                        .xor
                        .push(Xor::try_from(ip).map_err(Error::InvalidXor)?);
                }
                // this type is ignored.
                ProxyType::XorV1 => continue,
            }
        }

        Ok(proxies)
    }
}

pub struct AvailableProxies {
    pub plain: Vec<Plain>,
    pub xor: Vec<Xor>,
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
        Err(Error::UnknownType(0x4523)) => (),
        anything_else => panic!("Expected unknown type 0x33, got {anything_else:x?}"),
    }
}

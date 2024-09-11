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

#[derive(PartialEq)]
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

pub trait Obfuscator: Send {
    fn addr(&self) -> SocketAddrV4;
    fn obfuscate(&mut self, buffer: &mut [u8]);
    fn clone(&self) -> Box<dyn Obfuscator>;
}

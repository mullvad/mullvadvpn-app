use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    io::{Cursor, Read},
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4},
};

/// Obfuscator that does not obfuscate. It still can circumvent censorship since it is reaching our
/// API through a different IP address.
///
/// A plain configuration is represented by proxy type ProxyType::Plain (0x01). A plain
/// configuration interprets the following bytes from a given IPv6 address:
/// bytes 4-8 - u16le - proxy type - must be 0x0001
/// bytes 8-16 - [u8; 4] - 4 bytes representing the proxy IPv4 address
/// bytes 16-18 - u16le - port on which the proxy is listening
///
/// Given the above, an IPv6 address `2001:100:b9d5:9a75:3804::` will have the second hexlet
/// (0x0100) represent the proxy type, the following 2 hexlets (0xb9d5, 0x9a75) - the IPv4 address
/// of the proxy endpoint, and the final hexlet represents the port for the proxy endpoint - the
/// remaining bytes can be ignored.
#[derive(PartialEq, Debug, Clone)]
pub struct Plain {
    pub addr: SocketAddrV4,
}

#[derive(Debug)]
pub enum Error {
    UnexpectedType(u16),
}

impl TryFrom<Ipv6Addr> for Plain {
    type Error = Error;

    fn try_from(ip: Ipv6Addr) -> Result<Self, Self::Error> {
        let mut cursor = Cursor::new(ip.octets());

        // skip the first 2 bytes since it's just padding to make the IP look more like a legit
        // IPv6 address.
        cursor.set_position(2);
        let proxy_type = cursor.read_u16::<LittleEndian>().unwrap();
        if proxy_type != super::ProxyType::Plain as u16 {
            return Err(Error::UnexpectedType(proxy_type));
        }

        let mut ipv4_bytes = [0u8; 4];
        cursor.read_exact(&mut ipv4_bytes).unwrap();
        let v4_addr = Ipv4Addr::from(ipv4_bytes);

        let port = cursor.read_u16::<LittleEndian>().unwrap();

        Ok(Self {
            addr: SocketAddrV4::new(v4_addr, port),
        })
    }
}

impl super::Obfuscator for Plain {
    // can be a noop, since this configuration is just a port forward.
    fn obfuscate(&mut self, _buffer: &mut [u8]) {}

    fn addr(&self) -> SocketAddrV4 {
        self.addr
    }

    fn clone(&self) -> Box<dyn super::Obfuscator> {
        Box::new(Clone::clone(self))
    }
}

#[test]
fn test_parsing() {
    struct Test {
        input: Ipv6Addr,
        expected: Plain,
    }
    let tests = vec![
        Test {
            input: "2001:100:7f00:1:3905::".parse::<Ipv6Addr>().unwrap(),
            expected: Plain {
                addr: "127.0.0.1:1337".parse::<SocketAddrV4>().unwrap(),
            },
        },
        Test {
            input: "2001:100:c0a8:101:bb01::".parse::<Ipv6Addr>().unwrap(),
            expected: Plain {
                addr: "192.168.1.1:443".parse::<SocketAddrV4>().unwrap(),
            },
        },
        Test {
            input: "2001:100:c0a8:101:bb01:404::".parse::<Ipv6Addr>().unwrap(),
            expected: Plain {
                addr: "192.168.1.1:443".parse::<SocketAddrV4>().unwrap(),
            },
        },
    ];

    for t in tests {
        let parsed = Plain::try_from(t.input).unwrap();
        assert_eq!(parsed, t.expected);
    }
}

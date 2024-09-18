use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    io::{Cursor, Read},
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4},
};

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

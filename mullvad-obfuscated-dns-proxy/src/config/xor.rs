use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    io::{Cursor, Read},
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4},
};

use crate::config::Obfuscator;

#[derive(PartialEq, Debug)]
pub struct Xor {
    addr: SocketAddrV4,
    // the key to be used for Xor
    xor_key: Vec<u8>,
    key_index: usize,
}

#[derive(Debug)]
pub enum Error {
    EmptyXorKey,
    UnexpectedType(u16),
}

impl TryFrom<Ipv6Addr> for Xor {
    type Error = Error;

    fn try_from(ip: Ipv6Addr) -> Result<Self, Self::Error> {
        let mut cursor = Cursor::new(ip.octets());

        let _ = cursor.read_u16::<LittleEndian>().unwrap();
        let proxy_type = cursor.read_u16::<LittleEndian>().unwrap();
        if proxy_type != super::ProxyType::XorV2 as u16 {
            return Err(Error::UnexpectedType(proxy_type));
        }

        let mut ipv4_bytes = [0u8; 4];
        cursor.read_exact(&mut ipv4_bytes).unwrap();
        let v4_addr = Ipv4Addr::from(ipv4_bytes);

        let port = cursor.read_u16::<LittleEndian>().unwrap();

        let mut key_bytes = [0u8; 6];
        cursor.read_exact(&mut key_bytes).unwrap();
        let xor_key = key_bytes
            .into_iter()
            .filter(|byte| *byte != 0x00)
            .collect::<Vec<_>>();
        if xor_key.is_empty() {
            return Err(Error::EmptyXorKey);
        }

        Ok(Self {
            addr: SocketAddrV4::new(v4_addr, port),
            xor_key,
            key_index: 0,
        })
    }
}

impl Obfuscator for Xor {
    fn addr(&self) -> SocketAddrV4 {
        self.addr
    }

    fn obfuscate(&mut self, buffer: &mut [u8]) {
        for byte in buffer.iter_mut() {
            *byte ^= self.xor_key[self.key_index % self.xor_key.len()];
            self.key_index = (self.key_index + 1) % self.xor_key.len();
        }
    }

    fn clone(&self) -> Box<dyn super::Obfuscator> {
        Box::new(Self {
            xor_key: self.xor_key.clone(),
            addr: self.addr,
            key_index: 0,
        })
    }
}

#[test]
fn test_xor_parsing() {
    struct Test {
        input: Ipv6Addr,
        expected: Xor,
    }
    let tests = vec![
        Test {
            input: "2001:300:7f00:1:3905:0102:304:506"
                .parse::<Ipv6Addr>()
                .unwrap(),
            expected: Xor {
                addr: "127.0.0.1:1337".parse::<SocketAddrV4>().unwrap(),
                xor_key: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
                key_index: 0,
            },
        },
        Test {
            input: "2001:300:7f00:1:3905:0100:304:506"
                .parse::<Ipv6Addr>()
                .unwrap(),
            expected: Xor {
                addr: "127.0.0.1:1337".parse::<SocketAddrV4>().unwrap(),
                xor_key: vec![0x01, 0x03, 0x04, 0x05, 0x06],
                key_index: 0,
            },
        },
        Test {
            input: "2001:300:c0a8:101:bb01:ff04:204:0"
                .parse::<Ipv6Addr>()
                .unwrap(),
            expected: Xor {
                addr: "192.168.1.1:443".parse::<SocketAddrV4>().unwrap(),
                xor_key: vec![0xff, 0x04, 0x02, 0x04],
                key_index: 0,
            },
        },
    ];

    for t in tests {
        let parsed = Xor::try_from(t.input).unwrap();
        assert_eq!(parsed, t.expected);
    }
}

#[test]
fn test_obfuscation() {
    let input = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut payload = input.to_vec();
    let mut xor = Xor {
        addr: "192.168.1.1:443".parse::<SocketAddrV4>().unwrap(),
        xor_key: vec![0xff, 0x04, 0x02, 0x04],
        key_index: 0,
    };
    let mut dexor = xor.clone();
    xor.obfuscate(&mut payload);
    dexor.obfuscate(&mut payload);
    assert_eq!(input, payload.as_slice());
}

// Before XOR-v2 there was XOR-v1, which is now deprecated. This test verifies that the old Xor
// config does not deserialize.
#[test]
fn test_old_xor_addr() {
    let _ = Xor::try_from(
        "2001:200:7f00:1:3905:0102:304:506"
            .parse::<Ipv6Addr>()
            .unwrap(),
    )
    .unwrap_err();
}

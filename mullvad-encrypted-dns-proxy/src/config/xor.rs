use super::ProxyType;
use core::fmt;
use std::net::{Ipv4Addr, SocketAddrV4};

/// Parse a proxy config that XORs all traffic with the given key.
///
/// A Xor configuration is represented by the proxy type `ProxyType::XorV2`. There used to be a `XorV1`, but it
/// is deprecated and should not be used.
///
/// The following bytes of an IPv6 address are interpreted to derive a Xor configuration:
/// bytes 2-4 - u16le - proxy type - must be 0x03
/// bytes 4-8 - [u8; 4] - 4 bytes representing the proxy IPv4 address
/// bytes 8-10 - u16le - port on which the proxy is listening
/// bytes 10-16 - [u8; 6] - xor key bytes. 0x00 marks a premature end of the key
/// Given the above, `2001:300:b9d5:9a75:3a04:eafd:1100:ad9e` will have the second hexlet (0x0300)
/// represent the proxy type, the next 2 hexlets (0xb9d5,0x9a75) represent the IPv4 address for the
/// proxy endpoint, the next hexlet (`3a04`) represents the port for the proxy endpoint, and
/// the final 3 hexlets `eafd:1100:ad9e` represent the xor key (0xEA, 0xFD, 0x11).
pub fn parse_xor(data: [u8; 12]) -> Result<super::ProxyConfig, super::Error> {
    let (ip_bytes, tail) = data.split_first_chunk::<4>().unwrap();
    let (port_bytes, key_bytes) = tail.split_first_chunk::<2>().unwrap();
    let key_bytes = <[u8; 6]>::try_from(key_bytes).unwrap();

    let ip = Ipv4Addr::from(*ip_bytes);
    let port = u16::from_le_bytes(*port_bytes);
    if port == 0 {
        return Err(super::Error::InvalidPort(port));
    }
    let addr = SocketAddrV4::new(ip, port);

    let key = XorKey::try_from(key_bytes)?;

    Ok(super::ProxyConfig {
        addr,
        obfuscation: Some(super::ObfuscationConfig::XorV2(key)),
        r#type: ProxyType::XorV2,
    })
}

/// A bunch of bytes, representing a "key" Simply meaning a slice of bytes that the data
/// will be XORed with.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct XorKey {
    data: [u8; 6],
    len: usize,
}

impl XorKey {
    /// Return the XOR key material. Will always have at least length 1.
    pub fn key_data(&self) -> &[u8] {
        &self.data[0..self.len]
    }
}

impl fmt::Debug for XorKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        for byte in self.key_data() {
            write!(f, "{byte:0>2x}")?;
        }
        Ok(())
    }
}

impl TryFrom<[u8; 6]> for XorKey {
    type Error = super::Error;

    fn try_from(mut key_bytes: [u8; 6]) -> Result<Self, Self::Error> {
        let key_len = key_bytes
            .iter()
            .position(|b| *b == 0x00)
            .unwrap_or(key_bytes.len());
        if key_len == 0 {
            return Err(super::Error::EmptyXorKey);
        }

        // Reset bytes after terminating null to zeros.
        // Allows simpler implementations of Eq and Hash
        key_bytes[key_len..].fill(0);

        Ok(Self {
            data: key_bytes,
            len: key_len,
        })
    }
}

#[derive(Debug)]
pub struct XorObfuscator {
    key: XorKey,
    key_index: usize,
}

impl XorObfuscator {
    pub fn new(key: XorKey) -> Self {
        Self { key, key_index: 0 }
    }
}

impl super::Obfuscator for XorObfuscator {
    fn obfuscate(&mut self, buffer: &mut [u8]) {
        let key_data = self.key.key_data();
        for byte in buffer {
            *byte ^= key_data[self.key_index % key_data.len()];
            self.key_index = (self.key_index + 1) % key_data.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv6Addr, SocketAddrV4};

    use crate::config::xor::{XorKey, XorObfuscator};
    use crate::config::{Error, ObfuscationConfig, Obfuscator, ProxyConfig, ProxyType};

    #[test]
    fn xor_parsing() {
        struct Test {
            input: Ipv6Addr,
            expected: Result<ProxyConfig, Error>,
        }
        let tests = vec![
            Test {
                input: "2001:300:7f00:1:3905:0102:304:506"
                    .parse::<Ipv6Addr>()
                    .unwrap(),
                expected: Ok(ProxyConfig {
                    addr: "127.0.0.1:1337".parse::<SocketAddrV4>().unwrap(),
                    obfuscation: Some(ObfuscationConfig::XorV2(
                        XorKey::try_from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]).unwrap(),
                    )),
                    r#type: ProxyType::XorV2,
                }),
            },
            Test {
                input: "2001:300:7f00:1:3905:0100:304:506"
                    .parse::<Ipv6Addr>()
                    .unwrap(),
                expected: Ok(ProxyConfig {
                    addr: "127.0.0.1:1337".parse::<SocketAddrV4>().unwrap(),
                    obfuscation: Some(ObfuscationConfig::XorV2(
                        XorKey::try_from([0x01, 0, 0, 0, 0, 0]).unwrap(),
                    )),
                    r#type: ProxyType::XorV2,
                }),
            },
            Test {
                input: "2001:300:c0a8:101:bb01:ff04:204:0"
                    .parse::<Ipv6Addr>()
                    .unwrap(),
                expected: Ok(ProxyConfig {
                    addr: "192.168.1.1:443".parse::<SocketAddrV4>().unwrap(),
                    obfuscation: Some(ObfuscationConfig::XorV2(
                        XorKey::try_from([0xff, 0x04, 0x02, 0x04, 0, 0]).unwrap(),
                    )),
                    r#type: ProxyType::XorV2,
                }),
            },
        ];

        for t in tests {
            let parsed = ProxyConfig::try_from(t.input);
            assert_eq!(parsed, t.expected);
        }
    }

    #[test]
    fn obfuscation() {
        const INPUT: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut payload = INPUT.to_vec();

        let xor_key = XorKey::try_from([0xff, 0x04, 0x02, 0x04, 0x00, 0x00]).unwrap();

        let mut xor_obfuscator = XorObfuscator::new(xor_key);
        let mut xor_deobfuscator = XorObfuscator::new(xor_key);

        xor_obfuscator.obfuscate(&mut payload);

        assert_eq!(
            payload,
            &[0xfe, 0x06, 0x01, 0x00, 0xfa, 0x02, 0x05, 0x0c, 0xf6, 0x0e]
        );

        xor_deobfuscator.obfuscate(&mut payload);
        assert_eq!(INPUT, payload.as_slice());
    }

    // Before XOR-v2 there was XOR-v1, which is now deprecated. This test verifies that the old Xor
    // config does not deserialize.
    #[test]
    fn old_xor_addr() {
        match ProxyConfig::try_from(
            "2001:200:7f00:1:3905:0102:304:506"
                .parse::<Ipv6Addr>()
                .unwrap(),
        ) {
            Err(Error::XorV1Unsupported) => (),
            anything_else => panic!("Unexpected proxy config parse result: {anything_else:?}"),
        }
    }

    #[test]
    fn xor_key_debug_fmt() {
        let key = XorKey::try_from([0x01, 0xff, 0x31, 0x00, 0x00, 0x00]).unwrap();
        let key_str = format!("{key:?}");
        assert_eq!(key_str, "0x01ff31");
    }
}

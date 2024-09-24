use std::net::{Ipv4Addr, SocketAddrV4};

/// Parse a proxy config that does not obfuscate. It still can circumvent censorship since it is reaching our
/// API through a different IP address.
///
/// A plain configuration is represented by proxy type [`super::ProxyType::Plain`]. Normally the
/// input to this function will come from the last 12 bytes of an IPv6 address. A plain
/// configuration interprets the following bytes from a given IPv6 address:
/// bytes 2-4 - u16le - proxy type - must be 0x01
/// bytes 4-8 - [u8; 4] - 4 bytes representing the proxy IPv4 address
/// bytes 8-10 - u16le - port on which the proxy is listening
///
/// Given the above, an IPv6 address `2001:100:b9d5:9a75:3804::` will have the second hexlet
/// (0x0100) represent the proxy type, the following 2 hexlets (0xb9d5, 0x9a75) - the IPv4 address
/// of the proxy endpoint, and the final hexlet represents the port for the proxy endpoint - the
/// remaining bytes can be ignored.
pub fn parse_plain(data: [u8; 12]) -> Result<super::ProxyConfig, super::Error> {
    let (ip_bytes, tail) = data.split_first_chunk::<4>().unwrap();
    let (port_bytes, _tail) = tail.split_first_chunk::<2>().unwrap();

    let ip = Ipv4Addr::from(*ip_bytes);
    let port = u16::from_le_bytes(*port_bytes);
    if port == 0 {
        return Err(super::Error::InvalidPort(0));
    }
    let addr = SocketAddrV4::new(ip, port);

    Ok(super::ProxyConfig {
        addr,
        obfuscation: None,
    })
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv6Addr, SocketAddrV4};

    use crate::config::{Error, ProxyConfig};

    #[test]
    fn parsing() {
        struct Test {
            input: Ipv6Addr,
            expected: Result<ProxyConfig, Error>,
        }
        let tests = vec![
            Test {
                input: "2001:100:7f00:1:3905::".parse::<Ipv6Addr>().unwrap(),
                expected: Ok(ProxyConfig {
                    addr: "127.0.0.1:1337".parse::<SocketAddrV4>().unwrap(),
                    obfuscation: None,
                }),
            },
            Test {
                input: "2001:100:c0a8:101:bb01::".parse::<Ipv6Addr>().unwrap(),
                expected: Ok(ProxyConfig {
                    addr: "192.168.1.1:443".parse::<SocketAddrV4>().unwrap(),
                    obfuscation: None,
                }),
            },
            Test {
                input: "2001:100:c0a8:101:bb01:404::".parse::<Ipv6Addr>().unwrap(),
                expected: Ok(ProxyConfig {
                    addr: "192.168.1.1:443".parse::<SocketAddrV4>().unwrap(),
                    obfuscation: None,
                }),
            },
            Test {
                input: "2001:100:c0a8:101:0000:404::".parse::<Ipv6Addr>().unwrap(),
                expected: Err(Error::InvalidPort(0)),
            },
        ];

        for t in tests {
            let parsed = ProxyConfig::try_from(t.input);
            assert_eq!(parsed, t.expected);
        }
    }
}

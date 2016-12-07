use std::error::Error;
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;

/// Representation of a TCP or UDP endpoint. The host is represented as a String since it can be
/// both a hostname/domain as well as an IP.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RemoteAddr {
    address: String,
    port: u16,
}

impl RemoteAddr {
    /// Constructs a new `RemoteAddr` from the given address and port.
    pub fn new(address: &str, port: u16) -> Self {
        RemoteAddr {
            address: address.to_owned(),
            port: port,
        }
    }

    /// Returns the address associated with this `RemoteAddr`.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Returns the port associated with this `RemoteAddr`.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl From<SocketAddr> for RemoteAddr {
    fn from(socket_addr: SocketAddr) -> Self {
        RemoteAddr {
            address: socket_addr.ip().to_string(),
            port: socket_addr.port(),
        }
    }
}

impl FromStr for RemoteAddr {
    type Err = AddrParseError;
    /// Parse a string into a `RemoteAddr`. Does not work correctly with IPv6, see tests for
    /// defined behavior
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (address_str, port_str) = split_at_colon(s).map_err(|_| AddrParseError(()))?;
        let port = u16::from_str(port_str).map_err(|_| AddrParseError(()))?;
        if address_str.len() == 0 {
            return Err(AddrParseError(()));
        }
        Ok(RemoteAddr {
            address: String::from(address_str),
            port: port,
        })
    }
}

fn split_at_colon(s: &str) -> Result<(&str, &str), ()> {
    let mut iter = s.rsplitn(2, ":");
    let port = iter.next().unwrap();
    let address = iter.next().ok_or(())?;
    Ok((address, port))
}

impl fmt::Display for RemoteAddr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.address, self.port)
    }
}

/// Representation of the errors that can happen when parsing a string into a `RemoteAddr`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddrParseError(());

impl fmt::Display for AddrParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl Error for AddrParseError {
    fn description(&self) -> &str {
        "Invalid remote address format"
    }
}


#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn remote_addr_new_and_getters() {
        let remote_addr = RemoteAddr::new("a_domain", 543);
        assert_eq!("a_domain", remote_addr.address());
        assert_eq!(543, remote_addr.port());
    }

    #[test]
    fn remote_addr_from_socket_addr() {
        let socket_addr = SocketAddr::from_str("10.0.1.1:76").unwrap();
        let remote_addr: RemoteAddr = socket_addr.into();
        assert_eq!("10.0.1.1", remote_addr.address());
        assert_eq!(76, remote_addr.port());
    }

    #[test]
    fn remote_addr_from_str() {
        let remote_addr = RemoteAddr::from_str("example.com:3333").unwrap();
        assert_eq!("example.com", remote_addr.address());
        assert_eq!(3333, remote_addr.port());
    }

    // These tests exist to show that the parsing does not work exactly as the standard suggests
    // https://tools.ietf.org/html/rfc3986#appendix-A
    mod ipv6_invalid_parsing {
        use std::str::FromStr;
        use super::super::*;

        #[test]
        fn remote_addr_from_ipv6_str_without_brackets() {
            let remote_addr = RemoteAddr::from_str("fe80::1:1337").unwrap();
            assert_eq!("fe80::1", remote_addr.address());
            assert_eq!(1337, remote_addr.port());
        }

        #[test]
        fn remote_addr_from_ipv6_str_with_brackets() {
            let remote_addr = RemoteAddr::from_str("[fe80::1]:1337").unwrap();
            assert_eq!("[fe80::1]", remote_addr.address());
            assert_eq!(1337, remote_addr.port());
        }

        #[test]
        fn remote_addr_from_ipv6_str_without_port() {
            let remote_addr = RemoteAddr::from_str("fe80::1").unwrap();
            assert_eq!("fe80:", remote_addr.address());
            assert_eq!(1, remote_addr.port());
        }
    }

    #[test]
    fn remote_addr_from_str_no_colon() {
        assert!(RemoteAddr::from_str("example.com").is_err());
    }

    #[test]
    fn remote_addr_from_str_invalid_port_large() {
        assert!(RemoteAddr::from_str("example.com:99999").is_err());
    }

    #[test]
    fn remote_addr_from_str_empty_address() {
        assert!(RemoteAddr::from_str(":100").is_err());
    }

    #[test]
    fn remote_addr_from_str_empty_port() {
        assert!(RemoteAddr::from_str("example.com:").is_err());
    }

    #[test]
    fn remote_addr_to_string() {
        let formatted_remote = "10.98.150.255:1337";
        let remote_addr = RemoteAddr::from_str(formatted_remote).unwrap();
        assert_eq!(formatted_remote, remote_addr.to_string());
    }
}

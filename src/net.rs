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
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (address_str, port_str) = split_at_colon(s).map_err(|_| AddrParseError::InvalidPort)?;
        let port = u16::from_str(port_str).map_err(|_| AddrParseError::InvalidPort)?;
        if address_str.len() == 0 {
            return Err(AddrParseError::InvalidAddress);
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

/// Representation of the errors that can happen when parsing a string into a `RemoteAddr`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddrParseError {
    /// When the address is malformed
    InvalidAddress,
    /// When no port or an invalid port
    InvalidPort,
}

impl fmt::Display for AddrParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl Error for AddrParseError {
    fn description(&self) -> &str {
        match *self {
            AddrParseError::InvalidAddress => "Invalid address",
            AddrParseError::InvalidPort => "Invalid port",
        }
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

    #[test]
    fn remote_addr_from_ipv6_str() {
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

    #[test]
    fn remote_addr_from_str_no_colon() {
        let err = RemoteAddr::from_str("example.com");
        assert_eq!(Err(AddrParseError::InvalidPort), err);
    }

    #[test]
    fn remote_addr_from_str_invalid_port_large() {
        let err = RemoteAddr::from_str("example.com:99999");
        assert_eq!(Err(AddrParseError::InvalidPort), err);
    }

    #[test]
    fn remote_addr_from_str_empty_address() {
        let err = RemoteAddr::from_str(":100");
        assert_eq!(Err(AddrParseError::InvalidAddress), err);
    }

    #[test]
    fn remote_addr_from_str_empty_port() {
        let err = RemoteAddr::from_str("example.com:");
        assert_eq!(Err(AddrParseError::InvalidPort), err);
    }
}

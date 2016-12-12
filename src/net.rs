use regex::Regex;

use std::error::Error;
use std::fmt;
use std::io;
use std::iter;
use std::net::SocketAddr;
use std::option;
use std::slice;
use std::str::FromStr;
use std::vec;

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
        let (address, port_str) = split_remote_addr_string(s).map_err(|_| AddrParseError(()))?;
        let port = u16::from_str(port_str).map_err(|_| AddrParseError(()))?;
        Ok(RemoteAddr {
            address: address.to_owned(),
            port: port,
        })
    }
}

fn split_remote_addr_string(s: &str) -> Result<(&str, &str), ()> {
    let with_brackets = Regex::new(r"^\[([^\]]+)\]:([0-9]+)$").unwrap();
    let without_brackets = Regex::new(r"^([^:]+):([0-9]+)$").unwrap();
    let captures = with_brackets.captures(s).or(without_brackets.captures(s));
    captures.map(|cs| (cs.at(1).unwrap(), cs.at(2).unwrap())).ok_or(())
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


/// A trait for objects which can be converted to one or more `RemoteAddr` values.
pub trait ToRemoteAddrs {
    /// Returned iterator over remote addresses which this type may correspond
    /// to.
    type Iter: Iterator<Item = RemoteAddr>;

    /// Converts this object to an iterator of parsed `RemoteAddr`s.
    ///
    /// # Errors
    ///
    /// Any errors encountered during parsing will be returned as an `Err`.
    fn to_remote_addrs(&self) -> io::Result<Self::Iter>;
}

impl ToRemoteAddrs for RemoteAddr {
    type Iter = option::IntoIter<RemoteAddr>;

    fn to_remote_addrs(&self) -> io::Result<Self::Iter> {
        Ok(Some(self.clone()).into_iter())
    }
}

impl<'a> ToRemoteAddrs for &'a [RemoteAddr] {
    type Iter = iter::Cloned<slice::Iter<'a, RemoteAddr>>;

    fn to_remote_addrs(&self) -> io::Result<Self::Iter> {
        Ok(self.iter().cloned())
    }
}

impl<'a> ToRemoteAddrs for &'a str {
    type Iter = option::IntoIter<RemoteAddr>;

    fn to_remote_addrs(&self) -> io::Result<Self::Iter> {
        let parsed_addr = str_to_remote_addr(self)?;
        Ok(Some(parsed_addr).into_iter())
    }
}

impl<'a> ToRemoteAddrs for &'a [&'a str] {
    type Iter = vec::IntoIter<RemoteAddr>;

    fn to_remote_addrs(&self) -> io::Result<Self::Iter> {
        let mut addrs = Vec::with_capacity(self.len());
        for addr in self.iter() {
            addrs.push(str_to_remote_addr(addr)?);
        }
        Ok(addrs.into_iter())
    }
}

fn str_to_remote_addr(s: &str) -> io::Result<RemoteAddr> {
    RemoteAddr::from_str(s)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.description()))
}

#[cfg(test)]
mod remote_addr_tests {
    use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
    use std::str::FromStr;

    use super::*;

    #[test]
    fn new_and_getters() {
        let testee = RemoteAddr::new("a_domain", 543);
        assert_eq!("a_domain", testee.address());
        assert_eq!(543, testee.port());
    }

    #[test]
    fn from_socket_addr() {
        let socket_addr = SocketAddr::from_str("10.0.1.1:76").unwrap();
        let testee: RemoteAddr = socket_addr.into();
        assert_eq!("10.0.1.1", testee.address());
        assert_eq!(76, testee.port());
    }

    #[test]
    fn from_str() {
        let testee = RemoteAddr::from_str("example.com:3333").unwrap();
        assert_eq!("example.com", testee.address());
        assert_eq!(3333, testee.port());
    }

    #[test]
    fn from_ipv6_str_without_brackets() {
        assert!(RemoteAddr::from_str("fe80::1:1337").is_err());
    }

    #[test]
    fn from_ipv6_str_with_brackets() {
        let testee = RemoteAddr::from_str("[fe80::1]:1337").unwrap();
        assert_eq!("fe80::1", testee.address());
        assert_eq!(1337, testee.port());
    }

    #[test]
    fn from_ipv6_str_without_port() {
        assert!(RemoteAddr::from_str("fe80::1").is_err());
    }

    #[test]
    fn from_str_no_colon() {
        assert!(RemoteAddr::from_str("example.com").is_err());
    }

    #[test]
    fn from_str_invalid_port_large() {
        assert!(RemoteAddr::from_str("example.com:99999").is_err());
    }

    #[test]
    fn from_str_empty_address() {
        assert!(RemoteAddr::from_str(":100").is_err());
    }

    #[test]
    fn from_str_empty_port() {
        assert!(RemoteAddr::from_str("example.com:").is_err());
    }

    #[test]
    fn to_string_domain() {
        let testee = RemoteAddr::new("example.com", 3333);
        assert_eq!("example.com:3333", testee.to_string());
    }

    #[test]
    fn to_string_ipv4() {
        let socket_addr = SocketAddr::V4(SocketAddrV4::from_str("127.1.2.3:1337").unwrap());
        let testee = RemoteAddr::from(socket_addr);
        assert_eq!("127.1.2.3:1337", testee.to_string());
    }

    #[test]
    #[ignore]
    fn to_string_ipv6() {
        let socket_addr = SocketAddr::V6(SocketAddrV6::from_str("[2001:beef::1]:9876").unwrap());
        let testee = RemoteAddr::from(socket_addr);
        assert_eq!("[2001:beef::1]:9876", testee.to_string());
    }
}

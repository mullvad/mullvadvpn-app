use std::fmt;
use std::io;
use std::iter;
use std::net::SocketAddr;
use std::option;
use std::slice;
use std::str::FromStr;
use std::vec;


error_chain! {
    errors {
        /// Error indicating parsing the address failed
        AddrParse(s: String) {
            description("Invalid address format")
            display("Unable to parse address. {}", s)
        }
    }
}


/// Representation of a TCP or UDP endpoint. The IP level address is represented by either an IP
/// directly or a hostname/domain. The IP level address together with a port becomes a socket
/// address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RemoteAddr {
    /// Endpoint represented by an IP and a port.
    SocketAddr(SocketAddr),
    /// Endpoint represented by a hostname or domain and a port.
    Domain(String, u16),
}

impl RemoteAddr {
    /// Constructs a new `RemoteAddr` from the given address (hostname or domain) and port. To
    /// construct a `RemoteAddr` based on IP rather than domain, use the From<SocketAddr> impl.
    pub fn new(address: &str, port: u16) -> Self {
        RemoteAddr::Domain(address.to_owned(), port)
    }

    /// Returns the address associated with this `RemoteAddr`. If it is backed by an IP that will
    /// be formatted as a string.
    pub fn address(&self) -> String {
        match *self {
            RemoteAddr::SocketAddr(ref addr) => addr.ip().to_string(),
            RemoteAddr::Domain(ref address, _) => address.to_owned(),
        }
    }

    /// Returns the port associated with this `RemoteAddr`.
    pub fn port(&self) -> u16 {
        match *self {
            RemoteAddr::SocketAddr(addr) => addr.port(),
            RemoteAddr::Domain(_, port) => port,
        }
    }

    fn from_domain_str(s: &str) -> Result<Self> {
        let (address, port_str) = Self::split_at_last_colon(s)?;
        let port = u16::from_str(port_str).chain_err(|| {
            ErrorKind::AddrParse(format!("Invalid port: \"{}\"", port_str))
        })?;
        if address.is_empty() || address.contains(':') {
            let msg = format!("Invalid IP or domain: \"{}\"", address);
            bail!(ErrorKind::AddrParse(msg));
        }
        Ok(RemoteAddr::Domain(address.to_owned(), port))
    }

    fn split_at_last_colon(s: &str) -> Result<(&str, &str)> {
        let mut iter = s.rsplitn(2, ':');
        let port = iter.next().unwrap();
        let address = iter.next()
            .ok_or_else(|| Error::from(ErrorKind::AddrParse("No colon".to_owned())))?;
        Ok((address, port))
    }
}

impl From<SocketAddr> for RemoteAddr {
    fn from(socket_addr: SocketAddr) -> Self {
        RemoteAddr::SocketAddr(socket_addr)
    }
}

impl FromStr for RemoteAddr {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        if let Ok(addr) = SocketAddr::from_str(s) {
            Ok(RemoteAddr::from(addr))
        } else {
            Self::from_domain_str(s)
        }
    }
}

impl fmt::Display for RemoteAddr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RemoteAddr::SocketAddr(ref addr) => addr.fmt(fmt),
            RemoteAddr::Domain(ref address, ref port) => write!(fmt, "{}:{}", address, port),
        }
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
    use super::*;

    use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
    use std::str::FromStr;

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
        let result = RemoteAddr::from_str("fe80::1:1337");
        assert_matches!(result, Err(Error(ErrorKind::AddrParse(_), _)));
    }

    #[test]
    fn from_ipv6_str_with_brackets() {
        let testee = RemoteAddr::from_str("[fe80::1]:1337").unwrap();
        assert_eq!("fe80::1", testee.address());
        assert_eq!(1337, testee.port());
    }

    #[test]
    fn from_ipv6_str_without_port() {
        let result = RemoteAddr::from_str("fe80::1");
        assert_matches!(result, Err(Error(ErrorKind::AddrParse(_), _)));
    }

    #[test]
    fn from_str_no_colon() {
        let result = RemoteAddr::from_str("example.com");
        assert_matches!(result, Err(Error(ErrorKind::AddrParse(_), _)));
    }

    #[test]
    fn from_str_invalid_port_large() {
        let result = RemoteAddr::from_str("example.com:99999");
        assert_matches!(result, Err(Error(ErrorKind::AddrParse(_), _)));
    }

    #[test]
    fn from_str_empty_address() {
        let result = RemoteAddr::from_str(":100");
        assert_matches!(result, Err(Error(ErrorKind::AddrParse(_), _)));
    }

    #[test]
    fn from_str_empty_port() {
        let result = RemoteAddr::from_str("example.com:");
        assert_matches!(result, Err(Error(ErrorKind::AddrParse(_), _)));
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
    fn to_string_ipv6() {
        let socket_addr = SocketAddr::V6(SocketAddrV6::from_str("[2001:beef::1]:9876").unwrap());
        let testee = RemoteAddr::from(socket_addr);
        assert_eq!("[2001:beef::1]:9876", testee.to_string());
    }
}

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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_addr_new_and_getters() {
        let remote_addr = RemoteAddr::new("a_domain", 543);
        assert_eq!("a_domain", remote_addr.address());
        assert_eq!(543, remote_addr.port());
    }
}

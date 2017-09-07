use std::fmt;
use talpid_types::net::TransportProtocol;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RelayEndpoint {
    pub host: String,
    pub port: u16,
    pub protocol: TransportProtocol,
}

impl fmt::Display for RelayEndpoint {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{} - {:?}", self.host, self.port, self.protocol)
    }
}

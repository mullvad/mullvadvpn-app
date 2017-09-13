use std::fmt;
use std::net::{SocketAddr, ToSocketAddrs};
use talpid_types;
use talpid_types::net::TransportProtocol;

error_chain!{
    errors {
        InvalidHost(host: String) {
            display("Invalid host: {}", host)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RelayEndpoint {
    pub host: String,
    pub port: u16,
    pub protocol: TransportProtocol,
}

impl RelayEndpoint {
    pub fn to_endpoint(&self) -> Result<talpid_types::net::Endpoint> {

        let socket_addrs = to_socket_addrs(self.host.as_str(), self.port)?;
        ensure!(
            socket_addrs.len() > 0,
            ErrorKind::InvalidHost(self.host.clone())
        );

        let socket_addr = choose_ip(&socket_addrs).unwrap();

        if socket_addrs.len() > 1 {
            info!(
                "{} resolved to more than one IP, ignoring all but {}",
                self.host,
                socket_addr.ip()
            )
        }

        Ok(talpid_types::net::Endpoint::new(socket_addr.ip(), socket_addr.port(), self.protocol),)
    }
}

/// Does a DNS lookup if the host isn't an IP.
fn to_socket_addrs(host: &str, port: u16) -> Result<Vec<SocketAddr>> {
    Ok(
        (host, port)
            .to_socket_addrs()
            .chain_err(|| ErrorKind::InvalidHost(host.to_owned()))?
            .collect(),
    )
}

fn choose_ip(socket_addrs: &Vec<SocketAddr>) -> Option<SocketAddr> {
    // We prefer IPv4 addresses, so we split the addresses into
    // IPv4 ad IPv6s and take form the IPv4 pile if any.

    let (mut ipv4, mut ipv6): (Vec<SocketAddr>, Vec<SocketAddr>) =
        socket_addrs
            .into_iter()
            .partition(|addr| addr.is_ipv4());

    // If there are many IP:s, we simply ignore the rest
    ipv4.pop().or(ipv6.pop())
}

impl fmt::Display for RelayEndpoint {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{} - {}", self.host, self.port, self.protocol)
    }
}

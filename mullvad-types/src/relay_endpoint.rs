use std::cmp::Ordering;
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

        let mut socket_addrs = to_socket_addrs(self.host.as_str(), self.port)?;
        ensure!(
            socket_addrs.len() > 0,
            ErrorKind::InvalidHost(self.host.clone())
        );

        let socket_addr = choose_ip(&mut socket_addrs);

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

fn choose_ip(socket_addrs: &mut Vec<SocketAddr>) -> SocketAddr {

    // We want to prefer IPv4 addresses so we sort the addresses putting
    // the IPv4 addresses at the start of the vector.
    socket_addrs.sort_by(
        |a, _| if a.is_ipv4() {
            Ordering::Less
        } else {
            Ordering::Equal
        },
    );

    // If there are many IP:s, we simply ignore the rest
    socket_addrs[0]
}

impl fmt::Display for RelayEndpoint {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{} - {}", self.host, self.port, self.protocol)
    }
}

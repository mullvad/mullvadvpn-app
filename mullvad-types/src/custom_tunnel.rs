use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{IpAddr, ToSocketAddrs},
};
use talpid_types::net::{TunnelEndpoint, TunnelEndpointData};

error_chain! {
    errors {
        InvalidHost(host: String) {
            display("Invalid host: {}", host)
        }
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CustomTunnelEndpoint {
    pub host: String,
    pub tunnel: TunnelEndpointData,
}

impl CustomTunnelEndpoint {
    pub fn to_tunnel_endpoint(&self) -> Result<TunnelEndpoint> {
        Ok(TunnelEndpoint {
            address: resolve_to_ip(&self.host)?,
            tunnel: self.tunnel.clone(),
        })
    }
}

impl fmt::Display for CustomTunnelEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} over {}", self.host, self.tunnel)
    }
}

/// Does a DNS lookup if the host isn't an IP.
/// Returns the first IPv4 address if one exists, otherwise the first IPv6 address.
/// Rust only provides means to resolve a socket addr, not just a host, for some reason. So
/// because of this we do the resolving with port zero and then pick out the IPs.
fn resolve_to_ip(host: &str) -> Result<IpAddr> {
    let (mut ipv4, mut ipv6): (Vec<IpAddr>, Vec<IpAddr>) = (host, 0)
        .to_socket_addrs()
        .chain_err(|| ErrorKind::InvalidHost(host.to_owned()))?
        .map(|addr| addr.ip())
        .partition(|addr| addr.is_ipv4());

    ipv4.pop()
        .or_else(|| {
            log::info!("No IPv4 for host {}", host);
            ipv6.pop()
        })
        .ok_or_else(|| ErrorKind::InvalidHost(host.to_owned()).into())
}

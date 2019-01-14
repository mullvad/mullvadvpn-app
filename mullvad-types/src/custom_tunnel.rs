use crate::settings::TunnelOptions;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
};
use talpid_types::net::{openvpn, wireguard, TunnelParameters};

error_chain! {
    errors {
        InvalidHost(host: String) {
            display("Invalid host: {}", host)
        }
        Unsupported {
            description("Tunnel type not supported")
        }
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CustomTunnelEndpoint {
    host: String,
    config: ConnectionConfig,
}

impl CustomTunnelEndpoint {
    pub fn new(host: String, config: ConnectionConfig) -> Self {
        Self { host, config }
    }

    pub fn to_tunnel_parameters(&self, tunnel_options: TunnelOptions) -> Result<TunnelParameters> {
        let ip = resolve_to_ip(&self.host)?;
        let mut config = self.config.clone();
        config.set_ip(ip);

        let parameters = match config {
            ConnectionConfig::OpenVpn(config) => openvpn::TunnelParameters {
                config,
                options: tunnel_options.openvpn.clone(),
                generic_options: tunnel_options.generic.clone(),
            }
            .into(),
            ConnectionConfig::Wireguard(connection) => wireguard::TunnelParameters {
                connection,
                options: tunnel_options.wireguard.clone(),
                generic_options: tunnel_options.generic.clone(),
            }
            .into(),
        };
        Ok(parameters)
    }
}

impl fmt::Display for CustomTunnelEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.config {
            ConnectionConfig::OpenVpn(config) => write!(
                f,
                "OpenVPN relay - {}:{} {}",
                self.host,
                config.endpoint.address.port(),
                config.endpoint.protocol
            ),
            ConnectionConfig::Wireguard(connection) => write!(
                f,
                "WireGuard relay - {} with public key {}",
                connection.peer.endpoint, connection.peer.public_key
            ),
        }
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

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename = "connection_config")]
pub enum ConnectionConfig {
    #[serde(rename = "openvpn")]
    OpenVpn(openvpn::ConnectionConfig),
    #[serde(rename = "wireguard")]
    Wireguard(wireguard::ConnectionConfig),
}

impl ConnectionConfig {
    fn set_ip(&mut self, ip: IpAddr) {
        match self {
            ConnectionConfig::OpenVpn(config) => {
                config.endpoint.address = SocketAddr::new(ip, config.endpoint.address.port());
            }
            ConnectionConfig::Wireguard(config) => {
                config.peer.endpoint = SocketAddr::new(ip, config.peer.endpoint.port())
            }
        }
    }
}

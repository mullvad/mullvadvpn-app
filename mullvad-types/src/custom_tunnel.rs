use crate::settings::TunnelOptions;
use serde::{Deserialize, Serialize};
use std::{
    fmt, io,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
};
use talpid_types::net::{openvpn, proxy::CustomProxy, wireguard, Endpoint, TunnelParameters};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid host/domain: {0}")]
    InvalidHost(String, #[source] io::Error),

    #[error("Host has no IPv4 address: {0}")]
    HostHasNoIpv4(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CustomTunnelEndpoint {
    pub host: String,
    pub config: ConnectionConfig,
}

impl CustomTunnelEndpoint {
    pub fn new(host: String, config: ConnectionConfig) -> Self {
        Self { host, config }
    }

    pub fn endpoint(&self) -> Endpoint {
        match &self.config {
            ConnectionConfig::OpenVpn(config) => config.endpoint,
            ConnectionConfig::Wireguard(config) => config.get_endpoint(),
        }
    }

    pub fn to_tunnel_parameters(
        &self,
        tunnel_options: TunnelOptions,
        proxy: Option<CustomProxy>,
    ) -> Result<TunnelParameters, Error> {
        let ip = resolve_to_ip(&self.host)?;
        let mut config = self.config.clone();
        config.set_ip(ip);

        let parameters = match config {
            ConnectionConfig::OpenVpn(config) => openvpn::TunnelParameters {
                config,
                options: tunnel_options.openvpn,
                generic_options: tunnel_options.generic,
                proxy,
                #[cfg(target_os = "linux")]
                fwmark: crate::TUNNEL_FWMARK,
            }
            .into(),
            ConnectionConfig::Wireguard(connection) => {
                let mut options = tunnel_options.wireguard.into_talpid_tunnel_options();
                if options.quantum_resistant {
                    options.quantum_resistant = false;
                    log::info!("Ignoring quantum resistant option for custom tunnel");
                }
                wireguard::TunnelParameters {
                    connection,
                    options,
                    generic_options: tunnel_options.generic,
                    obfuscation: None,
                }
                .into()
            }
        };
        Ok(parameters)
    }
}

impl fmt::Display for CustomTunnelEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
                "WireGuard relay - {}:{} with public key {}",
                self.host,
                connection.peer.endpoint.port(),
                connection.peer.public_key
            ),
        }
    }
}

/// Does a DNS lookup if the host isn't an IP.
/// Returns the first IPv4 address if one exists, otherwise the first IPv6 address.
/// Rust only provides means to resolve a socket addr, not just a host, for some reason. So
/// because of this we do the resolving with port zero and then pick out the IPs.
fn resolve_to_ip(host: &str) -> Result<IpAddr, Error> {
    let (mut ipv4, mut ipv6): (Vec<IpAddr>, Vec<IpAddr>) = (host, 0)
        .to_socket_addrs()
        .map_err(|e| Error::InvalidHost(host.to_owned(), e))?
        .map(|addr| addr.ip())
        .partition(|addr| addr.is_ipv4());

    ipv4.pop()
        .or_else(|| {
            log::info!("No IPv4 for host {}", host);
            ipv6.pop()
        })
        .ok_or_else(|| Error::HostHasNoIpv4(host.to_owned()))
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

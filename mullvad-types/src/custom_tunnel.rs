use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
};
use talpid_types::net::{
    OpenVpnConnectionConfig, OpenVpnTunnelParameters, TunnelParameters, WireguardConnectionConfig,
    WireguardTunnelParameters,
};

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
    pub config: ConnectionConfig,
}

impl CustomTunnelEndpoint {
    pub fn to_connection_config(&self) -> Result<ConnectionConfig> {
        let ip = resolve_to_ip(&self.host)?;
        let mut config = self.config.clone();
        config.set_ip(ip);

        Ok(config)
    }
}

impl fmt::Display for CustomTunnelEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tunnel_type = match &self.config {
            ConnectionConfig::OpenVpn(_) => &"OpenVpn",
            ConnectionConfig::Wireguard(_) => &"Wireguard",
        };
        write!(f, "{} over {}", self.host, tunnel_type)
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
pub enum ConnectionConfig {
    OpenVpn(OpenVpnConnectionConfig),
    Wireguard(WireguardConnectionConfig),
}

impl ConnectionConfig {
    fn set_ip(&mut self, ip: IpAddr) {
        match self {
            ConnectionConfig::OpenVpn(config) => {
                config.host = SocketAddr::new(ip, config.host.port())
            }
            ConnectionConfig::Wireguard(config) => {
                config.host = SocketAddr::new(ip, config.host.port())
            }
        }
    }

    pub fn to_tunnel_parameters(self, settings: &Settings) -> TunnelParameters {
        let tunnel_options = settings.get_tunnel_options().clone();
        match self {
            ConnectionConfig::OpenVpn(config) => OpenVpnTunnelParameters {
                config,
                options: tunnel_options.openvpn,
                generic_options: tunnel_options.generic,
            }
            .into(),

            ConnectionConfig::Wireguard(config) => WireguardTunnelParameters {
                config,
                options: tunnel_options.wireguard,
                generic_options: tunnel_options.generic,
            }
            .into(),
        }
    }
}

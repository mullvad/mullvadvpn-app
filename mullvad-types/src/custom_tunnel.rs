use crate::settings::TunnelOptions;
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};
use std::{
    fmt, io,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
};
use talpid_types::net::{openvpn, wireguard, Endpoint, TunnelParameters};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Invalid host/domain: {}", _0)]
    InvalidHost(String, #[error(source)] io::Error),

    #[error(display = "Host has no IPv4 address: {}", _0)]
    HostHasNoIpv4(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
// TODO: Remove this Java conversion once `jnix` supports skipping fields in enum tuple variants.
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[cfg_attr(target_os = "android", jnix(skip_all))]
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
        proxy: Option<openvpn::ProxySettings>,
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
            ConnectionConfig::Wireguard(connection) => wireguard::TunnelParameters {
                connection,
                options: tunnel_options.wireguard.into_talpid_tunnel_options(),
                generic_options: tunnel_options.generic,
                obfuscation: None,
            }
            .into(),
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

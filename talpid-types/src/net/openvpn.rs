use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct TunnelParameters {
    pub config: ConnectionConfig,
    pub options: TunnelOptions,
    pub generic_options: super::GenericTunnelOptions,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ConnectionConfig {
    pub host: SocketAddr,
    pub protocol: super::TransportProtocol,
    pub username: String,
}

impl ConnectionConfig {
    pub fn new(
        address: SocketAddr,
        protocol: super::TransportProtocol,
        username: String,
    ) -> ConnectionConfig {
        Self {
            host: address,
            protocol,
            username,
        }
    }
    pub fn get_endpoint(&self) -> super::Endpoint {
        super::Endpoint {
            address: self.host,
            protocol: self.protocol,
        }
    }
}

/// TunnelOptions contains options for an openvpn tunnel that should be applied
/// irrespective of the relay parameters - i.e. have nothing to do with the particular
/// OpenVPN server, but do affect the connection.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TunnelOptions {
    /// Optional argument for openvpn to try and limit TCP packet size,
    /// as discussed [here](https://openvpn.net/archive/openvpn-users/2003-11/msg00154.html)
    pub mssfix: Option<u16>,
    /// Proxy settings, for when the relay connection should be via a proxy.
    pub proxy: Option<ProxySettings>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxySettings {
    Local(LocalProxySettings),
    Remote(RemoteProxySettings),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct LocalProxySettings {
    pub port: u16,
    pub peer: SocketAddr,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct RemoteProxySettings {
    pub address: SocketAddr,
    pub auth: Option<ProxyAuth>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ProxyAuth {
    pub username: String,
    pub password: String,
}

pub struct ProxySettingsValidation;

impl ProxySettingsValidation {
    pub fn validate(proxy: &ProxySettings) -> Result<(), String> {
        match proxy {
            ProxySettings::Local(local) => {
                if local.port == 0 {
                    return Err(String::from("Invalid local port number"));
                }
                if local.peer.ip().is_loopback() {
                    return Err(String::from(
                        "localhost is not a valid peer in this context",
                    ));
                }
                if local.peer.port() == 0 {
                    return Err(String::from("Invalid remote port number"));
                }
            }
            ProxySettings::Remote(remote) => {
                if remote.address.port() == 0 {
                    return Err(String::from("Invalid port number"));
                }
                if remote.address.ip().is_loopback() {
                    return Err(String::from("localhost is not a valid remote server"));
                }
            }
        };
        Ok(())
    }
}

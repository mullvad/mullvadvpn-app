use crate::net::{
    proxy::{ProxyEndpoint, ProxyType},
    Endpoint, GenericTunnelOptions, TransportProtocol,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Information needed by `OpenVpnMonitor` to establish a tunnel connection.
/// See [`crate::net::TunnelParameters`].
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct TunnelParameters {
    pub config: ConnectionConfig,
    pub options: TunnelOptions,
    pub generic_options: GenericTunnelOptions,
    pub proxy: Option<ProxySettings>,
    #[cfg(target_os = "linux")]
    pub fwmark: u32,
}

/// Connection configuration used by [`TunnelParameters`].
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ConnectionConfig {
    pub endpoint: Endpoint,
    pub username: String,
    pub password: String,
}

impl ConnectionConfig {
    pub fn new(endpoint: Endpoint, username: String, password: String) -> ConnectionConfig {
        Self {
            endpoint,
            username,
            password,
        }
    }
}

/// `TunnelOptions` contains options for an OpenVPN tunnel that should be applied
/// irrespective of the relay parameters - i.e. have nothing to do with the particular
/// OpenVPN server, but do affect the connection.
/// Stored in [`TunnelParameters`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TunnelOptions {
    /// Optional argument for openvpn to try and limit TCP packet size,
    /// as discussed [here](https://openvpn.net/archive/openvpn-users/2003-11/msg00154.html)
    pub mssfix: Option<u16>,
}

/// Proxy server options to be used by `OpenVpnMonitor` when starting a tunnel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxySettings {
    Local(LocalProxySettings),
    Remote(RemoteProxySettings),
    Shadowsocks(ShadowsocksProxySettings),
}

impl ProxySettings {
    pub fn get_endpoint(&self) -> ProxyEndpoint {
        match self {
            ProxySettings::Local(settings) => ProxyEndpoint {
                endpoint: settings.get_endpoint(),
                proxy_type: ProxyType::Custom,
            },
            ProxySettings::Remote(settings) => ProxyEndpoint {
                endpoint: settings.get_endpoint(),
                proxy_type: ProxyType::Custom,
            },
            ProxySettings::Shadowsocks(settings) => ProxyEndpoint {
                endpoint: settings.get_endpoint(),
                proxy_type: ProxyType::Shadowsocks,
            },
        }
    }
}

/// Options for a generic proxy running on localhost.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct LocalProxySettings {
    pub port: u16,
    pub peer: SocketAddr,
}

impl LocalProxySettings {
    pub fn get_endpoint(&self) -> Endpoint {
        Endpoint {
            address: self.peer,
            protocol: TransportProtocol::Tcp,
        }
    }
}

/// Options for a generic proxy running on remote host.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct RemoteProxySettings {
    pub address: SocketAddr,
    pub auth: Option<ProxyAuth>,
}

impl RemoteProxySettings {
    pub fn get_endpoint(&self) -> Endpoint {
        Endpoint {
            address: self.address,
            protocol: TransportProtocol::Tcp,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ProxyAuth {
    pub username: String,
    pub password: String,
}

/// Options for a bundled Shadowsocks proxy.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ShadowsocksProxySettings {
    pub peer: SocketAddr,
    /// Password on peer.
    pub password: String,
    pub cipher: String,
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
}

impl ShadowsocksProxySettings {
    pub fn get_endpoint(&self) -> Endpoint {
        Endpoint {
            address: self.peer,
            protocol: TransportProtocol::Tcp,
        }
    }
}

/// Options for a bundled SOCKS5 proxy.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct SocksProxySettings {
    pub peer: SocketAddr,
}

impl SocksProxySettings {
    pub fn get_endpoint(&self) -> Endpoint {
        Endpoint {
            address: self.peer,
            protocol: TransportProtocol::Tcp,
        }
    }
}

/// List of ciphers usable by a Shadowsocks proxy.
/// Cf. [`ShadowsocksProxySettings::cipher`].
pub const SHADOWSOCKS_CIPHERS: [&str; 19] = [
    // Stream ciphers.
    "aes-128-cfb",
    "aes-128-cfb1",
    "aes-128-cfb8",
    "aes-128-cfb128",
    "aes-256-cfb",
    "aes-256-cfb1",
    "aes-256-cfb8",
    "aes-256-cfb128",
    "rc4",
    "rc4-md5",
    "chacha20",
    "salsa20",
    "chacha20-ietf",
    // AEAD ciphers.
    "aes-128-gcm",
    "aes-256-gcm",
    "chacha20-ietf-poly1305",
    "xchacha20-ietf-poly1305",
    "aes-128-pmac-siv",
    "aes-256-pmac-siv",
];

/// Checks whether the proxy settings to be used by `OpenVpnMonitor` are valid.
pub fn validate_proxy_settings(proxy: &ProxySettings) -> Result<(), String> {
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
        ProxySettings::Shadowsocks(ss) => {
            if ss.peer.ip().is_loopback() {
                return Err(String::from(
                    "localhost is not a valid peer in this context",
                ));
            }
            if ss.peer.port() == 0 {
                return Err(String::from("Invalid remote port number"));
            }
            if !SHADOWSOCKS_CIPHERS.contains(&ss.cipher.as_str()) {
                return Err(String::from("Invalid cipher"));
            }
        }
    };
    Ok(())
}

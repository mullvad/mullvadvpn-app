use crate::net::Endpoint;
use serde::{Deserialize, Serialize};
use std::{fmt, net::SocketAddr};

use super::TransportProtocol;

/// Types of bridges that can be used to proxy a connection to a tunnel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyType {
    Shadowsocks,
    Custom,
}

impl fmt::Display for ProxyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let bridge = match self {
            ProxyType::Shadowsocks => "Shadowsocks",
            ProxyType::Custom => "custom bridge",
        };
        write!(f, "{bridge}")
    }
}

/// Bridge endpoint, broadcast as part of a [`crate::net::TunnelEndpoint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProxyEndpoint {
    #[serde(flatten)]
    pub endpoint: Endpoint,
    pub proxy_type: ProxyType,
}

/// User customized proxy used for obfuscation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CustomProxy {
    Shadowsocks(Shadowsocks),
    Socks5Local(Socks5Local),
    Socks5Remote(Socks5Remote),
}

impl CustomProxy {
    pub fn get_remote_endpoint(&self) -> ProxyEndpoint {
        match self {
            CustomProxy::Socks5Local(settings) => ProxyEndpoint {
                endpoint: settings.remote_endpoint,
                proxy_type: ProxyType::Custom,
            },
            CustomProxy::Socks5Remote(settings) => ProxyEndpoint {
                endpoint: Endpoint::from_socket_address(settings.endpoint, TransportProtocol::Tcp),
                proxy_type: ProxyType::Custom,
            },
            CustomProxy::Shadowsocks(settings) => ProxyEndpoint {
                endpoint: Endpoint::from_socket_address(settings.endpoint, TransportProtocol::Tcp),
                proxy_type: ProxyType::Shadowsocks,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Shadowsocks {
    pub endpoint: SocketAddr,
    pub password: String,
    /// One of [`shadowsocks_ciphers`].
    /// Gets validated at a later stage. Is assumed to be valid.
    ///
    /// shadowsocks_ciphers: talpid_types::net::openvpn::SHADOWSOCKS_CIPHERS
    pub cipher: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Socks5Local {
    pub remote_endpoint: Endpoint,
    /// Port on localhost where the SOCKS5-proxy listens to.
    pub local_port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Socks5Remote {
    pub endpoint: SocketAddr,
    pub auth: Option<SocksAuth>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SocksAuth {
    pub username: String,
    pub password: String,
}

impl Shadowsocks {
    pub fn new<I: Into<SocketAddr>>(endpoint: I, cipher: String, password: String) -> Self {
        Shadowsocks {
            endpoint: endpoint.into(),
            password,
            cipher,
        }
    }
}

impl Socks5Local {
    pub fn new<I: Into<SocketAddr>>(remote_endpoint: I, local_port: u16) -> Self {
        let transport_protocol = TransportProtocol::Tcp;
        Self::new_with_transport_protocol(remote_endpoint, local_port, transport_protocol)
    }

    pub fn new_with_transport_protocol<I: Into<SocketAddr>>(
        remote_endpoint: I,
        local_port: u16,
        transport_protocol: TransportProtocol,
    ) -> Self {
        let remote_endpoint =
            Endpoint::from_socket_address(remote_endpoint.into(), transport_protocol);
        Self {
            remote_endpoint,
            local_port,
        }
    }
}

impl Socks5Remote {
    pub fn new<I: Into<SocketAddr>>(endpoint: I) -> Self {
        Self {
            endpoint: endpoint.into(),
            auth: None,
        }
    }

    pub fn new_with_authentication<I: Into<SocketAddr>>(
        endpoint: I,
        authentication: SocksAuth,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            auth: Some(authentication),
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

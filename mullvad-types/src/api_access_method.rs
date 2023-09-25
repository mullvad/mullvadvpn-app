use std::str::FromStr;

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

/// Daemon settings for API access methods.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub api_access_methods: Vec<AccessMethod>,
}

/// API access method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AccessMethod {
    Shadowsocks(Shadowsocks),
    Socks5(Socks5),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Shadowsocks {
    pub peer: SocketAddr,
    pub password: String, // TODO: Mask the password (using special type)?
    pub cipher: String,   // Gets validated at a later stage. Is assumed to be valid.
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Socks5 {
    Local(Socks5Local),
    Remote(Socks5Remote),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Socks5Local {
    pub peer: SocketAddr,
    /// Port on localhost where the SOCKS5-proxy listens to.
    pub port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Socks5Remote {
    pub peer: SocketAddr,
}

impl Settings {
    // TODO: Do I have to clone?
    pub fn get_access_methods(&self) -> Vec<AccessMethod> {
        self.api_access_methods.clone()
    }
}

impl Shadowsocks {
    pub fn new(peer: SocketAddr, cipher: String, password: String) -> Self {
        Shadowsocks {
            peer,
            password,
            cipher,
        }
    }

    /// Like [new()], but tries to parse `ip` and `port` into a [`std::net::SocketAddr`] for you.
    /// If `ip` or `port` are valid [`Some(Socks5Local)`] is returned, otherwise [`None`].
    pub fn from_args(ip: String, port: u16, cipher: String, password: String) -> Option<Self> {
        let peer = SocketAddrV4::new(Ipv4Addr::from_str(&ip).ok()?, port).into();
        Some(Self::new(peer, cipher, password))
    }
}

impl Socks5Local {
    pub fn new(peer: SocketAddr, port: u16) -> Self {
        Self { peer, port }
    }

    /// Like [new()], but tries to parse `ip` and `port` into a [`std::net::SocketAddr`] for you.
    /// If `ip` or `port` are valid [`Some(Socks5Local)`] is returned, otherwise [`None`].
    pub fn from_args(ip: String, port: u16, localport: u16) -> Option<Self> {
        let peer_ip = IpAddr::V4(Ipv4Addr::from_str(&ip).ok()?);
        let peer = SocketAddr::new(peer_ip, port);
        Some(Self::new(peer, localport))
    }
}

impl Socks5Remote {
    pub fn new(peer: SocketAddr) -> Self {
        Self { peer }
    }

    /// Like [new()], but tries to parse `ip` and `port` into a [`std::net::SocketAddr`] for you.
    /// If `ip` or `port` are valid [`Some(Socks5Remote)`] is returned, otherwise [`None`].
    pub fn from_args(ip: String, port: u16) -> Option<Self> {
        let peer_ip = IpAddr::V4(Ipv4Addr::from_str(&ip).ok()?);
        let peer = SocketAddr::new(peer_ip, port);
        Some(Self::new(peer))
    }
}

impl From<Shadowsocks> for AccessMethod {
    fn from(value: Shadowsocks) -> Self {
        AccessMethod::Shadowsocks(value)
    }
}

impl From<Socks5Remote> for AccessMethod {
    fn from(value: Socks5Remote) -> Self {
        AccessMethod::Socks5(value.into())
    }
}

impl From<Socks5Local> for AccessMethod {
    fn from(value: Socks5Local) -> Self {
        AccessMethod::Socks5(value.into())
    }
}

impl From<Socks5Remote> for Socks5 {
    fn from(value: Socks5Remote) -> Self {
        Socks5::Remote(value)
    }
}

impl From<Socks5Local> for Socks5 {
    fn from(value: Socks5Local) -> Self {
        Socks5::Local(value)
    }
}

/// These are just extensions to the core [`AccessMethod`] datastructure which the mullvad daemon needs.
pub mod daemon {
    use super::*;

    impl From<AccessMethod> for ApiAccessMethod {
        fn from(value: AccessMethod) -> Self {
            ApiAccessMethod {
                id: Some(uuid::Uuid::new_v4().to_string()),
                access_method: value,
            }
        }
    }

    /// TODO: Document why this is needed.
    /// Hint: Argument to protobuf rpc `ApiAccessMethodReplace`.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct ApiAccessMethodReplace {
        pub index: usize,
        pub access_method: AccessMethod,
    }
}

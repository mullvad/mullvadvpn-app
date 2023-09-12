use std::str::FromStr;

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

/// Daemon settings for API access methods.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub api_access_methods: Vec<AccessMethod>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_access_methods: vec![
                BuiltInAccessMethod::Direct.into(),
                BuiltInAccessMethod::Bridge.into(),
            ],
        }
    }
}

/// Access method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AccessMethod {
    BuiltIn(BuiltInAccessMethod),
    Custom(CustomAccessMethod),
}

/// Built-In access method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BuiltInAccessMethod {
    Direct,
    Bridge,
}

/// Custom access method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CustomAccessMethod {
    pub id: String,
    pub access_method: ObfuscationProtocol,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ObfuscationProtocol {
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

impl AccessMethod {
    pub fn is_custom(&self) -> bool {
        matches!(self, AccessMethod::Custom(..))
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self, AccessMethod::BuiltIn(..))
    }

    pub fn get_custom(&self) -> Option<&CustomAccessMethod> {
        match self {
            AccessMethod::BuiltIn(_) => None,
            AccessMethod::Custom(access_method) => Some(access_method),
        }
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

impl From<BuiltInAccessMethod> for AccessMethod {
    fn from(value: BuiltInAccessMethod) -> Self {
        AccessMethod::BuiltIn(value)
    }
}

impl From<CustomAccessMethod> for AccessMethod {
    fn from(value: CustomAccessMethod) -> Self {
        AccessMethod::Custom(value)
    }
}

impl From<ObfuscationProtocol> for AccessMethod {
    fn from(value: ObfuscationProtocol) -> Self {
        CustomAccessMethod {
            id: uuid::Uuid::new_v4().to_string(),
            access_method: value,
        }
        .into()
    }
}

impl From<Shadowsocks> for AccessMethod {
    fn from(value: Shadowsocks) -> Self {
        ObfuscationProtocol::Shadowsocks(value).into()
    }
}

impl From<Socks5> for AccessMethod {
    fn from(value: Socks5) -> Self {
        ObfuscationProtocol::Socks5(value).into()
    }
}

impl From<Socks5Remote> for AccessMethod {
    fn from(value: Socks5Remote) -> Self {
        Socks5::Remote(value).into()
    }
}

impl From<Socks5Local> for AccessMethod {
    fn from(value: Socks5Local) -> Self {
        Socks5::Local(value).into()
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
    /// TODO: Document why this is needed.
    /// Hint: Argument to protobuf rpc `ApiAccessMethodReplace`.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct ApiAccessMethodReplace {
        pub index: usize,
        pub access_method: AccessMethod,
    }
}

use std::collections::hash_map::DefaultHasher;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
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
                BuiltInAccessMethod::Direct(true).into(),
                BuiltInAccessMethod::Bridge(true).into(),
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
    Direct(bool),
    Bridge(bool),
}

/// Custom access method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CustomAccessMethod {
    pub id: String,
    pub access_method: ObfuscationProtocol,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ObfuscationProtocol {
    Shadowsocks(Shadowsocks),
    Socks5(Socks5),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Socks5 {
    Local(Socks5Local),
    Remote(Socks5Remote),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Shadowsocks {
    pub peer: SocketAddr,
    pub password: String, // TODO: Mask the password (using special type)?
    pub cipher: String,   // Gets validated at a later stage. Is assumed to be valid.
    pub enabled: bool,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Socks5Local {
    pub peer: SocketAddr,
    /// Port on localhost where the SOCKS5-proxy listens to.
    pub port: u16,
    pub enabled: bool,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Socks5Remote {
    pub peer: SocketAddr,
    pub enabled: bool,
    pub name: String,
}

impl Hash for Shadowsocks {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.peer.hash(state);
        self.password.hash(state);
        self.cipher.hash(state);
    }
}

impl Hash for Socks5Local {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.peer.hash(state);
        self.port.hash(state);
    }
}

impl Hash for Socks5Remote {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.peer.hash(state);
    }
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

    pub fn as_custom(&self) -> Option<&CustomAccessMethod> {
        match self {
            AccessMethod::BuiltIn(_) => None,
            AccessMethod::Custom(access_method) => Some(access_method),
        }
    }

    pub fn enabled(&self) -> bool {
        match self {
            AccessMethod::BuiltIn(method) => match method {
                BuiltInAccessMethod::Direct(enabled) => *enabled,
                BuiltInAccessMethod::Bridge(enabled) => *enabled,
            },
            AccessMethod::Custom(method) => match &method.access_method {
                ObfuscationProtocol::Shadowsocks(ss) => ss.enabled,
                ObfuscationProtocol::Socks5(socks) => match socks {
                    Socks5::Local(local) => local.enabled,
                    Socks5::Remote(remote) => remote.enabled,
                },
            },
        }
    }

    /// Set an access method to be either enabled or disabled.
    ///
    /// This action mutates [`self`].
    pub fn toggle(&mut self, enable: bool) -> () {
        match self {
            AccessMethod::BuiltIn(method) => match method {
                BuiltInAccessMethod::Direct(enabled) => *enabled = enable,
                BuiltInAccessMethod::Bridge(enabled) => *enabled = enable,
            },
            AccessMethod::Custom(method) => match method.access_method {
                ObfuscationProtocol::Shadowsocks(ref mut ss) => ss.enabled = enable,
                ObfuscationProtocol::Socks5(ref mut socks) => match socks {
                    Socks5::Local(local) => local.enabled = enable,
                    Socks5::Remote(remote) => remote.enabled = enable,
                },
            },
        }
    }

    /// Ad-hoc implementation of `==` from [`std::cmp::PartialEq`], where temporal member
    /// values such as `enabled` is disregarded.
    ///
    /// This is a shallow comparison.
    pub fn semantically_equals(&self, other: &Self) -> bool {
        // TODO: Implemenet this function in impl<ObfuscationProtocol>,
        // impl<ObfuscationProtocol> & impl <BuiltIn>.
        match (self, other) {
            (
                AccessMethod::BuiltIn(BuiltInAccessMethod::Bridge(_)),
                AccessMethod::BuiltIn(BuiltInAccessMethod::Bridge(_)),
            ) => true,
            (
                AccessMethod::BuiltIn(BuiltInAccessMethod::Direct(_)),
                AccessMethod::BuiltIn(BuiltInAccessMethod::Direct(_)),
            ) => true,
            (AccessMethod::Custom(ref custom_left), AccessMethod::Custom(ref custom_right)) => {
                match (&custom_left.access_method, &custom_right.access_method) {
                    (
                        ObfuscationProtocol::Shadowsocks(shadowsocks_left),
                        ObfuscationProtocol::Shadowsocks(shadowsocks_right),
                    ) => shadowsocks_left.semantically_equals(&shadowsocks_right),
                    (
                        ObfuscationProtocol::Socks5(socks_left),
                        ObfuscationProtocol::Socks5(socks_right),
                    ) => socks_left.semantically_equals(&socks_right),
                    (_, _) => false,
                }
            }
            _ => false,
        }
    }
}

impl Shadowsocks {
    pub fn new(
        peer: SocketAddr,
        cipher: String,
        password: String,
        enabled: bool,
        name: String,
    ) -> Self {
        Shadowsocks {
            peer,
            password,
            cipher,
            enabled,
            name,
        }
    }

    /// Like [new()], but tries to parse `ip` and `port` into a [`std::net::SocketAddr`] for you.
    /// If `ip` or `port` are valid [`Some(Socks5Local)`] is returned, otherwise [`None`].
    pub fn from_args(
        ip: String,
        port: u16,
        cipher: String,
        password: String,
        enabled: bool,
        name: String,
    ) -> Option<Self> {
        let peer = SocketAddrV4::new(Ipv4Addr::from_str(&ip).ok()?, port).into();
        Some(Self::new(peer, cipher, password, enabled, name))
    }

    pub fn semantically_equals(&self, other: &Self) -> bool {
        self.peer == other.peer && self.cipher == other.cipher && self.password == other.password
    }
}

impl Socks5 {
    pub fn semantically_equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Socks5::Local(left), Socks5::Local(right)) => left.semantically_equals(right),
            (Socks5::Remote(left), Socks5::Remote(right)) => left.semantically_equals(right),
            (Socks5::Remote(_), Socks5::Local(_)) => false,
            (Socks5::Local(_), Socks5::Remote(_)) => false,
        }
    }
}

impl Socks5Local {
    pub fn new(peer: SocketAddr, port: u16, enabled: bool, name: String) -> Self {
        Self {
            peer,
            port,
            enabled,
            name,
        }
    }

    /// Like [new()], but tries to parse `ip` and `port` into a [`std::net::SocketAddr`] for you.
    /// If `ip` or `port` are valid [`Some(Socks5Local)`] is returned, otherwise [`None`].
    pub fn from_args(
        ip: String,
        port: u16,
        localport: u16,
        enabled: bool,
        name: String,
    ) -> Option<Self> {
        let peer_ip = IpAddr::V4(Ipv4Addr::from_str(&ip).ok()?);
        let peer = SocketAddr::new(peer_ip, port);
        Some(Self::new(peer, localport, enabled, name))
    }

    fn semantically_equals(&self, other: &Socks5Local) -> bool {
        self.peer == other.peer && self.port == other.port
    }
}

impl Socks5Remote {
    pub fn new(peer: SocketAddr, enabled: bool, name: String) -> Self {
        Self {
            peer,
            enabled,
            name,
        }
    }

    /// Like [new()], but tries to parse `ip` and `port` into a [`std::net::SocketAddr`] for you.
    /// If `ip` or `port` are valid [`Some(Socks5Remote)`] is returned, otherwise [`None`].
    pub fn from_args(ip: String, port: u16, enabled: bool, name: String) -> Option<Self> {
        let peer_ip = IpAddr::V4(Ipv4Addr::from_str(&ip).ok()?);
        let peer = SocketAddr::new(peer_ip, port);
        Some(Self::new(peer, enabled, name))
    }

    fn semantically_equals(&self, other: &Socks5Remote) -> bool {
        self.peer == other.peer
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
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        CustomAccessMethod {
            id: hasher.finish().to_string(),
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
    /// TODO: Document why this is needed.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct ApiAccessMethodToggle {
        pub access_method: AccessMethod,
        pub enable: bool,
    }
}

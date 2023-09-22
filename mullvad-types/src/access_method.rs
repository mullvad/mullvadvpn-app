use std::collections::hash_map::DefaultHasher;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

/// Daemon settings for API access methods.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub api_access_methods: Vec<ApiAccessMethod>,
}

impl Settings {
    /// Append an [`AccessMethod`] to the end of `api_access_methods`.
    #[inline(always)]
    pub fn append(&mut self, api_access_method: ApiAccessMethod) {
        self.api_access_methods.push(api_access_method)
    }

    /// Remove a [`CustomAccessMethod`] from `api_access_methods`.
    #[inline(always)]
    pub fn remove(&mut self, custom_access_method: &CustomAccessMethod) {
        self.retain(|api_access_method| {
            api_access_method
                .access_method
                .as_custom()
                .map(|access_method| access_method.id != custom_access_method.id)
                .unwrap_or(true)
        })
    }

    /// Search for a particular [`AccessMethod`] in `api_access_methods`.
    ///
    /// If the [`AccessMethod`] is found to be part of `api_access_methods`, a
    /// mutable reference to that inner element is returned. Otherwise, `None`
    /// is returned.
    #[inline(always)]
    pub fn find_mut(&mut self, element: &ApiAccessMethod) -> Option<&mut ApiAccessMethod> {
        self.api_access_methods
            .iter_mut()
            .find(|api_access_method| {
                // TODO: Can probably replace with `element.id == api_access_method.id`
                element
                    .access_method
                    .semantically_equals(&api_access_method.access_method)
            })
    }

    /// Equivalent to [`Vec::retain`].
    #[inline(always)]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&ApiAccessMethod) -> bool,
    {
        self.api_access_methods.retain(f)
    }

    /// Removes an element from `api_access_methods` and returns it.
    /// The removed element is replaced by the last element of the vector.
    ///
    /// Equivalent to [`Vec::swap_remove`].
    #[inline(always)]
    pub fn swap_remove(&mut self, index: usize) -> ApiAccessMethod {
        self.api_access_methods.swap_remove(index)
    }

    /// Clone the content of `api_access_methods`.
    #[inline(always)]
    pub fn cloned(&self) -> Vec<ApiAccessMethod> {
        self.api_access_methods.clone()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_access_methods: vec![BuiltInAccessMethod::Direct, BuiltInAccessMethod::Bridge]
                .into_iter()
                .map(|built_in| ApiAccessMethod {
                    name: built_in.canonical_name(),
                    enabled: true,
                    access_method: AccessMethod::from(built_in),
                })
                .collect(),
        }
    }
}

/// API Access Method datastructure
///
/// Mirrors the protobuf definition
/// TODO(Create a constructor functions for this struct (?))
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiAccessMethod {
    pub name: String,
    pub enabled: bool,
    pub access_method: AccessMethod,
}

/// Access Method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AccessMethod {
    BuiltIn(BuiltInAccessMethod),
    Custom(CustomAccessMethod),
}

impl ApiAccessMethod {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn as_custom(&self) -> Option<&CustomAccessMethod> {
        self.access_method.as_custom()
    }

    /// Set an API access method to be either enabled or disabled.
    pub fn toggle(&mut self, enable: bool) {
        self.enabled = enable;
    }
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
    pub password: String,
    /// One of [`shadowsocks_ciphers`].
    /// Gets validated at a later stage. Is assumed to be valid.
    ///
    /// shadowsocks_ciphers: talpid_types::net::openvpn::SHADOWSOCKS_CIPHERS
    pub cipher: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Socks5Local {
    pub peer: SocketAddr,
    /// Port on localhost where the SOCKS5-proxy listens to.
    pub port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Socks5Remote {
    pub peer: SocketAddr,
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

impl AccessMethod {
    pub fn as_custom(&self) -> Option<&CustomAccessMethod> {
        match self {
            AccessMethod::BuiltIn(_) => None,
            AccessMethod::Custom(access_method) => Some(access_method),
        }
    }

    /// Ad-hoc implementation of `==` from [`std::cmp::PartialEq`], where temporal member
    /// values such as `enabled` are disregarded.
    pub fn semantically_equals(&self, other: &Self) -> bool {
        match (self, other) {
            (AccessMethod::BuiltIn(left), AccessMethod::BuiltIn(right)) => {
                left.semantically_equals(right)
            }
            (AccessMethod::Custom(left), AccessMethod::Custom(right)) => {
                left.access_method.semantically_equals(&right.access_method)
            }
            _ => false,
        }
    }
}

impl BuiltInAccessMethod {
    pub fn semantically_equals(&self, other: &Self) -> bool {
        match (self, other) {
            (BuiltInAccessMethod::Bridge(_), BuiltInAccessMethod::Bridge(_)) => true,
            (BuiltInAccessMethod::Direct(_), BuiltInAccessMethod::Direct(_)) => true,
            (BuiltInAccessMethod::Direct(_), BuiltInAccessMethod::Bridge(_)) => false,
            (BuiltInAccessMethod::Bridge(_), BuiltInAccessMethod::Direct(_)) => false,

    pub fn canonical_name(&self) -> String {
        match self {
            BuiltInAccessMethod::Direct => "Direct".to_string(),
            BuiltInAccessMethod::Bridge => "Mullvad Bridges".to_string(),
        }
    }
}

impl ObfuscationProtocol {
    pub fn semantically_equals(&self, other: &Self) -> bool {
        match (self, other) {
            (ObfuscationProtocol::Shadowsocks(left), ObfuscationProtocol::Shadowsocks(right)) => {
                left.semantically_equals(right)
            }
            (ObfuscationProtocol::Socks5(left), ObfuscationProtocol::Socks5(right)) => {
                left.semantically_equals(right)
            }
            (ObfuscationProtocol::Shadowsocks(_), ObfuscationProtocol::Socks5(_)) => false,
            (ObfuscationProtocol::Socks5(_), ObfuscationProtocol::Shadowsocks(_)) => false,
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

    fn semantically_equals(&self, other: &Socks5Local) -> bool {
        self.peer == other.peer && self.port == other.port
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
        AccessMethod::from(CustomAccessMethod {
            id: hasher.finish().to_string(),
            access_method: value,
        })
    }
}

impl From<Shadowsocks> for AccessMethod {
    fn from(value: Shadowsocks) -> Self {
        ObfuscationProtocol::Shadowsocks(value).into()
    }
}

impl From<Socks5> for AccessMethod {
    fn from(value: Socks5) -> Self {
        AccessMethod::from(ObfuscationProtocol::Socks5(value))
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

/// Some short-lived datastructure used in some RPC calls to the mullvad daemon.
pub mod daemon {
    use super::*;
    /// Argument to protobuf rpc `ApiAccessMethodReplace`.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct ApiAccessMethodReplace {
        pub access_method: ApiAccessMethod,
        pub index: usize,
    }
    /// Argument to protobuf rpc `ApiAccessMethodToggle`.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct ApiAccessMethodToggle {
        pub access_method: ApiAccessMethod,
        pub enable: bool,
    }
}

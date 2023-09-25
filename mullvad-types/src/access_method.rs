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

    /// Remove an [`ApiAccessMethod`] from `api_access_methods`.
    #[inline(always)]
    pub fn remove(&mut self, api_access_method: &ApiAccessMethodId) {
        self.retain(|method| method.get_id() != *api_access_method)
    }

    /// Search for a particular [`AccessMethod`] in `api_access_methods`.
    #[inline(always)]
    pub fn find(&self, element: &ApiAccessMethodId) -> Option<&ApiAccessMethod> {
        self.api_access_methods
            .iter()
            .find(|api_access_method| *element == api_access_method.get_id())
    }

    /// Search for a particular [`AccessMethod`] in `api_access_methods`.
    ///
    /// If the [`AccessMethod`] is found to be part of `api_access_methods`, a
    /// mutable reference to that inner element is returned. Otherwise, `None`
    /// is returned.
    #[inline(always)]
    pub fn find_mut(&mut self, element: &ApiAccessMethodId) -> Option<&mut ApiAccessMethod> {
        self.api_access_methods
            .iter_mut()
            .find(|api_access_method| *element == api_access_method.get_id())
    }

    /// Equivalent to [`Vec::retain`].
    #[inline(always)]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&ApiAccessMethod) -> bool,
    {
        self.api_access_methods.retain(f)
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
                .map(|built_in| {
                    ApiAccessMethod::new(
                        built_in.canonical_name(),
                        true,
                        AccessMethod::from(built_in),
                    )
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
    /// Some unique id (distinct for each `AccessMethod`).
    id: ApiAccessMethodId,
    pub name: String,
    pub enabled: bool,
    pub access_method: AccessMethod,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiAccessMethodId(String);

impl ApiAccessMethodId {
    /// It is up to the caller to make sure that the supplied String is actually
    /// a valid UUID in the context of [`ApiAccessMethod`]s.
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for ApiAccessMethodId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for ApiAccessMethodId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&AccessMethod> for ApiAccessMethodId {
    fn from(value: &AccessMethod) -> Self {
        // Generate unqiue ID for this `AccessMethod`.
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        ApiAccessMethodId(hasher.finish().to_string())
    }
}

/// Access Method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
pub enum AccessMethod {
    BuiltIn(BuiltInAccessMethod),
    Custom(CustomAccessMethod),
}

impl ApiAccessMethod {
    pub fn new(name: String, enabled: bool, access_method: AccessMethod) -> Self {
        Self {
            id: ApiAccessMethodId::from(&access_method),
            name,
            enabled,
            access_method,
        }
    }

    pub fn get_id(&self) -> ApiAccessMethodId {
        self.id.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn as_custom(&self) -> Option<&CustomAccessMethod> {
        self.access_method.as_custom()
    }

    /// Set an API access method to be enabled.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Set an API access method to be disabled.
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

/// Built-In access method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
pub enum BuiltInAccessMethod {
    Direct,
    Bridge,
}

/// Custom access method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CustomAccessMethod {
    Shadowsocks(Shadowsocks),
    Socks5(Socks5),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Socks5 {
    Local(Socks5Local),
    Remote(Socks5Remote),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Shadowsocks {
    pub peer: SocketAddr,
    pub password: String,
    /// One of [`shadowsocks_ciphers`].
    /// Gets validated at a later stage. Is assumed to be valid.
    ///
    /// shadowsocks_ciphers: talpid_types::net::openvpn::SHADOWSOCKS_CIPHERS
    pub cipher: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Socks5Local {
    pub peer: SocketAddr,
    /// Port on localhost where the SOCKS5-proxy listens to.
    pub port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Socks5Remote {
    pub peer: SocketAddr,
}

impl AccessMethod {
    pub fn as_custom(&self) -> Option<&CustomAccessMethod> {
        match self {
            AccessMethod::BuiltIn(_) => None,
            AccessMethod::Custom(access_method) => Some(access_method),
        }
    }
}

impl BuiltInAccessMethod {
    pub fn canonical_name(&self) -> String {
        match self {
            BuiltInAccessMethod::Direct => "Direct".to_string(),
            BuiltInAccessMethod::Bridge => "Mullvad Bridges".to_string(),
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

impl From<Shadowsocks> for AccessMethod {
    fn from(value: Shadowsocks) -> Self {
        CustomAccessMethod::Shadowsocks(value).into()
    }
}

impl From<Socks5> for AccessMethod {
    fn from(value: Socks5) -> Self {
        AccessMethod::from(CustomAccessMethod::Socks5(value))
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
    /// Argument to protobuf rpc `UpdateApiAccessMethod`.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct ApiAccessMethodUpdate {
        pub id: ApiAccessMethodId,
        pub access_method: ApiAccessMethod,
    }
}

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use talpid_types::net::{Endpoint, TransportProtocol};

/// Daemon settings for API access methods.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub access_method_settings: Vec<AccessMethodSetting>,
}

impl Settings {
    /// Append an [`AccessMethod`] to the end of `api_access_methods`.
    pub fn append(&mut self, api_access_method: AccessMethodSetting) {
        self.access_method_settings.push(api_access_method)
    }

    /// Remove an [`ApiAccessMethod`] from `api_access_methods`.
    pub fn remove(&mut self, api_access_method: &Id) {
        self.retain(|method| method.get_id() != *api_access_method)
    }

    /// Search for any [`AccessMethod`] in `api_access_methods` which matches `predicate`.
    pub fn find<P>(&self, predicate: P) -> Option<&AccessMethodSetting>
    where
        P: Fn(&AccessMethodSetting) -> bool,
    {
        self.access_method_settings
            .iter()
            .find(|api_access_method| predicate(api_access_method))
    }

    /// Search for any [`AccessMethod`] in `api_access_methods`.
    pub fn find_mut<P>(&mut self, predicate: P) -> Option<&mut AccessMethodSetting>
    where
        P: Fn(&AccessMethodSetting) -> bool,
    {
        self.access_method_settings
            .iter_mut()
            .find(|api_access_method| predicate(api_access_method))
    }

    /// Search for a particular [`AccessMethod`] in `api_access_methods`.
    pub fn find_by_id(&self, element: &Id) -> Option<&AccessMethodSetting> {
        self.find(|api_access_method| *element == api_access_method.get_id())
    }

    /// Search for a particular [`AccessMethod`] in `api_access_methods`.
    pub fn find_by_id_mut(&mut self, element: &Id) -> Option<&mut AccessMethodSetting> {
        self.find_mut(|api_access_method| *element == api_access_method.get_id())
    }

    /// Equivalent to [`Vec::retain`].
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&AccessMethodSetting) -> bool,
    {
        self.access_method_settings.retain(f)
    }

    /// Clone the content of `api_access_methods`.
    pub fn cloned(&self) -> Vec<AccessMethodSetting> {
        self.access_method_settings.clone()
    }

    /// Get a reference to the `Direct` access method instance of this [`Settings`].
    pub fn get_direct(&mut self) -> Option<&mut AccessMethodSetting> {
        self.find_mut(|access_method| {
            access_method.access_method == BuiltInAccessMethod::Direct.into()
        })
    }

    pub fn direct() -> AccessMethodSetting {
        let method = BuiltInAccessMethod::Direct;
        AccessMethodSetting::new(method.canonical_name(), true, AccessMethod::from(method))
    }

    pub fn mullvad_bridges() -> AccessMethodSetting {
        let method = BuiltInAccessMethod::Bridge;
        AccessMethodSetting::new(method.canonical_name(), true, AccessMethod::from(method))
    }

    /// Retrieve all [`AccessMethodSetting`]s which are enabled.
    pub fn collect_enabled(&self) -> Vec<AccessMethodSetting> {
        self.cloned()
            .into_iter()
            .filter(|access_method| access_method.enabled)
            .collect()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            access_method_settings: vec![Settings::direct(), Settings::mullvad_bridges()],
        }
    }
}

/// API Access Method datastructure
///
/// Mirrors the protobuf definition
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AccessMethodSetting {
    /// Some unique id (distinct for each `AccessMethod`).
    id: Id,
    pub name: String,
    pub enabled: bool,
    pub access_method: AccessMethod,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Id(uuid::Uuid);

impl Id {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
    /// Tries to parse a UUID from a raw String. If it is successful, an
    /// [`Id`] is instantiated.
    pub fn from_string(id: String) -> Option<Self> {
        uuid::Uuid::from_str(&id).ok().map(Self)
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Access Method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AccessMethod {
    BuiltIn(BuiltInAccessMethod),
    Custom(CustomAccessMethod),
}

impl AccessMethodSetting {
    pub fn new(name: String, enabled: bool, access_method: AccessMethod) -> Self {
        Self {
            id: Id::new(),
            name,
            enabled,
            access_method,
        }
    }

    /// Just like [`new`], [`with_id`] will create a new [`ApiAccessMethod`].
    /// But instead of automatically generating a new UUID, the id is instead
    /// passed as an argument.
    ///
    /// This is useful when converting to [`ApiAccessMethod`] from other data
    /// representations, such as protobuf.
    ///
    /// [`new`]: ApiAccessMethod::new
    /// [`with_id`]: ApiAccessMethod::with_id
    pub fn with_id(id: Id, name: String, enabled: bool, access_method: AccessMethod) -> Self {
        Self {
            id,
            name,
            enabled,
            access_method,
        }
    }

    pub fn get_id(&self) -> Id {
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

    pub fn is_builtin(&self) -> bool {
        self.as_custom().is_none()
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
#[serde(rename_all = "snake_case")]
pub enum BuiltInAccessMethod {
    Direct,
    Bridge,
}

/// Custom access method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CustomAccessMethod {
    Shadowsocks(Shadowsocks),
    Socks5(Socks5),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
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
    pub remote_endpoint: Endpoint,
    /// Port on localhost where the SOCKS5-proxy listens to.
    pub local_port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Socks5Remote {
    pub peer: SocketAddr,
    pub authentication: Option<SocksAuth>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SocksAuth {
    pub username: String,
    pub password: String,
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
    pub fn new<I: Into<SocketAddr>>(peer: I, cipher: String, password: String) -> Self {
        Shadowsocks {
            peer: peer.into(),
            password,
            cipher,
        }
    }
}

impl Socks5Local {
    pub fn new<I: Into<SocketAddr>>(remote_peer: I, local_port: u16) -> Self {
        let transport_protocol = TransportProtocol::Tcp;
        Self::new_with_transport_protocol(remote_peer, local_port, transport_protocol)
    }

    pub fn new_with_transport_protocol<I: Into<SocketAddr>>(
        remote_peer: I,
        local_port: u16,
        transport_protocol: TransportProtocol,
    ) -> Self {
        let remote_endpoint = Endpoint::from_socket_address(remote_peer.into(), transport_protocol);
        Self {
            remote_endpoint,
            local_port,
        }
    }
}

impl Socks5Remote {
    pub fn new<I: Into<SocketAddr>>(peer: I) -> Self {
        Self {
            peer: peer.into(),
            authentication: None,
        }
    }

    pub fn new_with_authentication<I: Into<SocketAddr>>(
        peer: I,
        authentication: SocksAuth,
    ) -> Self {
        Self {
            peer: peer.into(),
            authentication: Some(authentication),
        }
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

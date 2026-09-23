use serde::{Deserialize, Serialize};
use talpid_types::net::proxy::{CustomProxy, Shadowsocks, Socks5Local, Socks5Remote};

/// Access Method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AccessMethod {
    BuiltIn(BuiltInAccessMethod),
    Custom(CustomProxy),
}

impl AccessMethod {
    pub fn as_custom(&self) -> Option<&CustomProxy> {
        match self {
            AccessMethod::BuiltIn(_) => None,
            AccessMethod::Custom(access_method) => Some(access_method),
        }
    }
}

impl From<CustomProxy> for AccessMethod {
    fn from(value: CustomProxy) -> Self {
        AccessMethod::Custom(value)
    }
}

impl From<Socks5Remote> for AccessMethod {
    fn from(value: Socks5Remote) -> Self {
        CustomProxy::Socks5Remote(value).into()
    }
}

impl From<Socks5Local> for AccessMethod {
    fn from(value: Socks5Local) -> Self {
        CustomProxy::Socks5Local(value).into()
    }
}

impl From<Shadowsocks> for AccessMethod {
    fn from(value: Shadowsocks) -> Self {
        CustomProxy::Shadowsocks(value).into()
    }
}

/// Built-In access method datastructure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BuiltInAccessMethod {
    Direct,
    Bridge,
    EncryptedDnsProxy,
    DomainFronting,
}

impl BuiltInAccessMethod {
    pub fn canonical_name(&self) -> String {
        match self {
            BuiltInAccessMethod::Direct => "Direct".to_string(),
            BuiltInAccessMethod::Bridge => "Mullvad Bridges".to_string(),
            BuiltInAccessMethod::EncryptedDnsProxy => "Encrypted DNS proxy".to_string(),
            BuiltInAccessMethod::DomainFronting => "Domain fronting".to_string(),
        }
    }
}

impl std::fmt::Display for BuiltInAccessMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.canonical_name())
    }
}

impl From<BuiltInAccessMethod> for AccessMethod {
    fn from(value: BuiltInAccessMethod) -> Self {
        AccessMethod::BuiltIn(value)
    }
}

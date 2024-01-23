use serde::{Deserialize, Serialize};
use talpid_types::net::proxy::{CustomProxy, Shadowsocks, Socks5Local, Socks5Remote};

/// Settings for API access methods.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub direct: AccessMethodSetting,
    pub mullvad_bridges: AccessMethodSetting,
    /// User-defined API access methods.
    pub user_defined: Vec<AccessMethodSetting>,
}

impl Settings {
    /// Append an [`AccessMethod`] to the end of `api_access_methods`.
    pub fn append(&mut self, api_access_method: AccessMethodSetting) {
        self.user_defined.push(api_access_method)
    }

    /// Remove an [`ApiAccessMethod`] from `api_access_methods`.
    ///
    /// This function will return an error if a built-in API access is about to
    /// be removed.
    pub fn remove(&mut self, api_access_method: &Id) -> Result<(), Error> {
        let maybe_setting = self
            .user_defined
            .iter()
            .find(|setting| setting.get_id() == *api_access_method);

        match maybe_setting {
            Some(x) => match x.access_method {
                AccessMethod::BuiltIn(ref built_in) => Err(Error::RemoveBuiltin {
                    attempted: built_in.clone(),
                }),
                AccessMethod::Custom(_) => {
                    self.user_defined
                        .retain(|method| method.get_id() != *api_access_method);
                    Ok(())
                }
            },
            None => Ok(()),
        }
    }

    /// Retrieve all [`AccessMethodSetting`]s which are enabled.
    pub fn collect_enabled(&self) -> Vec<AccessMethodSetting> {
        self.iter()
            .filter(|access_method| access_method.enabled)
            .cloned()
            .collect()
    }

    /// Iterate over references of built-in & custom access methods.
    pub fn iter(&self) -> impl Iterator<Item = &AccessMethodSetting> {
        use std::iter::once;
        once(&self.direct)
            .chain(once(&self.mullvad_bridges))
            .chain(&self.user_defined)
    }

    /// Iterate over mutable references of built-in & custom access methods.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut AccessMethodSetting> {
        use std::iter::once;
        once(&mut self.direct)
            .chain(once(&mut self.mullvad_bridges))
            .chain(&mut self.user_defined)
    }

    pub fn direct() -> AccessMethodSetting {
        let method = BuiltInAccessMethod::Direct;
        AccessMethodSetting::new(method.canonical_name(), true, AccessMethod::from(method))
    }

    fn mullvad_bridges() -> AccessMethodSetting {
        let method = BuiltInAccessMethod::Bridge;
        AccessMethodSetting::new(method.canonical_name(), true, AccessMethod::from(method))
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            direct: Settings::direct(),
            mullvad_bridges: Settings::mullvad_bridges(),
            user_defined: vec![],
        }
    }
}

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Built-in access methods can not be removed
    #[error(display = "Cannot remove built-in access method {}", attempted)]
    RemoveBuiltin { attempted: BuiltInAccessMethod },
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
        use std::str::FromStr;
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
    Custom(CustomProxy),
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

    pub fn disabled(&self) -> bool {
        !self.enabled
    }

    pub fn as_custom(&self) -> Option<&CustomProxy> {
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

impl AccessMethod {
    pub fn as_custom(&self) -> Option<&CustomProxy> {
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

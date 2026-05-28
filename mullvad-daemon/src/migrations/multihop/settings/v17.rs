//! Vendored types for the settings which this migration is migrating away from.

use crate::migrations::multihop::settings::{
    AllowedIps, Constraint, IpVersion, LocationConstraint, Ownership, Providers,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct __Settings {
    pub relay_settings: __RelaySettings,
    pub tunnel_options: __TunnelOptions,
    #[serde(default)]
    pub custom_lists: __CustomListsSettings,
    /// NOTE: This field is for simplifying the migration control flow. It should never leak outside
    /// of the migration.
    //
    // TODO: Update this field dynamically by running the relay selector.
    #[serde(skip)]
    pub magic_multihop: Option<__LocationConstraint>,
}

impl Settings {
    /// Given a top-level settings blob, try to parse the [`Settings`] subset from the previous settings.
    pub fn parse(settings: Value) -> Result<Self, Error> {
        Self::deserialize(&settings).map_err(Error::Deserialize)
    }

    /// A lens to the current relay settings in the existing settings blob.
    ///
    /// If the return value is [`Some`], it can safely be cast to [`RelaySettingsInner`].
    pub fn relay_settings(settings: &mut Value) -> Option<&mut Value> {
        // relay_settings -> normal
        // Note: normal key might not exist, if the user has custom relay settings.
        settings
            .get_mut("relay_settings")
            .expect("relay_settings")
            .get_mut("normal")
    }

    /// A lens to the current wireguard / tunnel settings in the existing settings blob.
    ///
    /// This can safely be cast to [`__WireguardSettings`]
    pub fn wireguard_settings(settings: &mut Value) -> &mut Value {
        // tunnel_options -> wireguard -> daita
        settings
            .get_mut("tunnel_options")
            .expect("tunnel_options")
            .get_mut("wireguard")
            .expect("wireguard")
    }

    pub fn legacy_multihop(&self) -> bool {
        let Some(settings) = self.relay_settings.normal.as_ref() else {
            return false;
        };
        settings.wireguard_constraints.use_multihop
    }

    pub fn magic_multihop(&self) -> bool {
        self.magic_multihop.is_some()
    }

    pub fn daita(&self) -> bool {
        self.tunnel_options.wireguard.daita.enabled
    }

    pub fn direct_only(&self) -> bool {
        !self
            .tunnel_options
            .wireguard
            .daita
            .use_multihop_if_necessary
    }

    pub fn filters(&self) -> bool {
        let (providers, ownership) = self.get_filters();
        providers.is_some() || ownership.is_some()
    }

    /// Get the exit filters from the current settings object.
    pub fn get_filters(&self) -> (Option<&__Providers>, Option<__Ownership>) {
        let Some(settings) = self.relay_settings.normal.as_ref() else {
            return (None, None);
        };
        let providers = match &settings.providers {
            __Constraint::Any => None,
            __Constraint::Only(providers) => Some(providers),
        };
        let ownership = match settings.ownership {
            __Constraint::Any => None,
            __Constraint::Only(ownership) => Some(ownership),
        };
        (providers, ownership)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct __RelaySettings {
    // If the user has custom relay settings, the "normal" field will not be populated.
    normal: Option<RelaySettingsInner>,
}

impl Default for __RelaySettings {
    fn default() -> Self {
        Self {
            normal: Some(Default::default()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelaySettingsInner {
    pub location: __Constraint<__LocationConstraint>,
    #[serde(default)]
    pub providers: __Constraint<__Providers>,
    #[serde(default)]
    pub ownership: __Constraint<__Ownership>,
    #[serde(default)]
    pub wireguard_constraints: __WireguardConstraints,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct __WireguardConstraints {
    pub use_multihop: bool,
    pub entry_location: __Constraint<__LocationConstraint>,
    pub entry_providers: __Constraint<__Providers>,
    pub entry_ownership: __Constraint<__Ownership>,
    // NOTE: This migration is not concerned with the following fields - leave them untouched!
    pub ip_version: __Constraint<__IpVersion>,
    pub allowed_ips: __Constraint<__AllowedIps>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct __TunnelOptions {
    pub wireguard: __WireguardSettings,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct __WireguardSettings {
    #[serde(default)]
    pub daita: __DaitaSettings,
    // NOTE: This migration is not concerned with the following fields - leave them untouched!
    pub mtu: Value,
    pub quantum_resistant: Value,
    #[serde(default)]
    pub rotation_interval: Value,
}

/// The DAITA setting
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct __DaitaSettings {
    /// If DAITA was enabled.
    pub enabled: bool,
    /// Whether to use multihop if the selected relay is not DAITA-compatible.
    pub use_multihop_if_necessary: bool,
}

/// Helper for mocking different test-cases.
#[derive(Debug, Default)]
#[cfg(test)]
pub struct SettingsBuilder {
    multihop: bool,
    daita: bool,
    direct_only: bool,
    magic_multihop: bool,
    providers: __Constraint<__Providers>,
    ownership: __Constraint<__Ownership>,
    entry_location: __Constraint<__LocationConstraint>,
}

#[cfg(test)]
impl SettingsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn multihop(mut self, value: bool) -> Self {
        self.multihop = value;
        self
    }

    pub fn magic_multihop(mut self, value: bool) -> Self {
        self.magic_multihop = value;
        self
    }

    pub fn daita(mut self, value: bool) -> Self {
        self.daita = value;
        self
    }

    pub fn direct_only(mut self, value: bool) -> Self {
        self.direct_only = value;
        self
    }

    /// If value is `true`, add a set of default provider/ownership filters.
    /// If value is `false`, clear the current provider/ownership filters.
    pub fn filters(mut self, value: bool) -> Self {
        use std::collections::HashSet;
        match value {
            true => self
                .providers(__Providers {
                    providers: HashSet::from_iter(["31337".to_string()]),
                })
                .ownership(__Ownership::MullvadOwned),
            false => {
                self.providers = __Constraint::Any;
                self.ownership = __Constraint::Any;
                self
            }
        }
    }

    pub fn providers(mut self, value: __Providers) -> Self {
        self.providers = __Constraint::Only(value);
        self
    }

    pub fn ownership(mut self, value: __Ownership) -> Self {
        self.ownership = __Constraint::Only(value);
        self
    }

    pub fn build(self) -> __Settings {
        let mut settings = __Settings::default();
        if self.multihop {
            let constraints = &mut settings
                .relay_settings
                .normal
                .as_mut()
                .expect("Settings to have normal relay settings")
                .wireguard_constraints;
            constraints.use_multihop = true;
            constraints.entry_location = self.entry_location;
        }
        settings.magic_multihop = self
            .magic_multihop
            .then_some(__LocationConstraint::default());
        settings.tunnel_options.wireguard.daita.enabled = self.daita;
        if self.daita {
            settings
                .tunnel_options
                .wireguard
                .daita
                .use_multihop_if_necessary = !self.direct_only;
        }
        settings.relay_settings.normal.as_mut().unwrap().providers = self.providers;
        settings.relay_settings.normal.as_mut().unwrap().ownership = self.ownership;
        settings
    }
}

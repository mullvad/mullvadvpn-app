//! Vendored types for the settings which this migration is migrating away from.

use crate::migrations::Error;
use crate::migrations::multihop::settings::{
    AllowedIps, Constraint, CustomListsSettings, GeographicLocationConstraint, IpVersion,
    LocationConstraint, Ownership, Providers,
};
use crate::relay_selector::RelaySelectorIO;

use anyhow::Context;
use mullvad_relay_selector::query::builder::RelayQueryBuilder;
use mullvad_relay_selector::query::{Hops, RelayQuery};
use mullvad_relay_selector::{CustomListProvider, GetRelay, RelaySelector, WireguardConfig};
use mullvad_types::relay_selector::ResolvedLocationConstraint;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

impl CustomListProvider for CustomListsSettings {
    fn custom_lists(&self) -> mullvad_types::custom_list::CustomListsSettings {
        let custom_lists: Vec<_> = self
            .custom_lists
            .iter()
            .cloned()
            .filter_map(|current| {
                let Ok(id) = current.id.parse() else {
                    log::error!("Failed to parse custom list id {}", current.id);
                    return None;
                };
                let mut custom_list = mullvad_types::custom_list::CustomList::with_id(id);
                custom_list.name = current.name;
                custom_list.locations = current.locations.into_iter().map(From::from).collect();
                Some(custom_list)
            })
            .collect();
        mullvad_types::custom_list::CustomListsSettings::from(custom_lists)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub relay_settings: RelaySettings,
    pub tunnel_options: TunnelOptions,
    #[serde(default)]
    pub custom_lists: CustomListsSettings,
    /// NOTE: This field is for simplifying the migration control flow. It should never leak outside
    /// of the migration.
    #[serde(skip)]
    pub magic_multihop: Option<LocationConstraint>,
}

impl From<Settings> for RelayQuery {
    fn from(value: Settings) -> Self {
        let builder = RelayQueryBuilder::new();
        // relay settings
        let Some(relay_settings) = value.relay_settings.normal else {
            // If the user uses custom relay settings, it is not really necessary detect if magic
            // multihop is in use.
            return builder.build();
        };
        // If multihop is in use, it is not really necessary detect if magic multihop is in use.
        let builder = if relay_settings.wireguard_constraints.use_multihop {
            return builder.multihop().build();
        } else {
            builder
        };

        let builder = if let Constraint::Only(ownership) = relay_settings.ownership {
            builder.ownership(ownership.into())
        } else {
            builder
        };
        let builder = if let Constraint::Only(providers) = relay_settings.providers
            && let Ok(providers) = providers.try_into()
        {
            builder.providers(providers)
        } else {
            builder
        };
        let builder = builder.location(ResolvedLocationConstraint::from_constraint(
            match relay_settings.location {
                Constraint::Any => mullvad_types::constraints::Constraint::Any,
                Constraint::Only(loc) => mullvad_types::constraints::Constraint::Only(loc.into()),
            },
            &value.custom_lists.into(),
        ));

        // tunnel options
        let daita = value.tunnel_options.wireguard.daita;
        let builder = if daita.enabled {
            builder.daita()
        } else {
            builder
        };
        let builder = if daita.use_multihop_if_necessary {
            builder.autohop()
        } else {
            builder
        };

        builder.build()
    }
}

impl Settings {
    /// Given a top-level settings blob, try to parse the [`Settings`] subset from the previous settings.
    pub fn parse(settings: Value) -> Result<Self, Error> {
        Self::deserialize(&settings).map_err(Error::Deserialize)
    }

    /// Run the relay selector to find out if "Magic multihop" is required to connect or not.
    pub fn check_magic_mulithop(mut self) -> anyhow::Result<Self> {
        let relay_selector = RelaySelectorIO::load(self.custom_lists.clone())
            .context("Failed to initialize relay selector. Skipping migration.")?;
        // Query the relay selector for entry relay
        // If an entry relay needs to be selected even though multihop is not explicitly enabled, the
        // entry might be needed to unblock the user post-migration.
        let query = RelayQuery::from(self.clone());
        if !matches!(query.hops, Hops::Auto(_)) {
            return Ok(self);
        }

        if let Ok(GetRelay {
            inner: WireguardConfig::Multihop { entry, .. },
            ..
        }) = relay_selector.get_relay_by_query(query)
        {
            // There is atleast one relay in the country of the relay which was automatically
            // selected. Set it as the new entry relay constraint.
            let entry = LocationConstraint::Location(GeographicLocationConstraint::Country(
                entry.inner.location.country_code,
            ));
            self.magic_multihop = Some(entry);
        };

        Ok(self)
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

    /// A lens to the current wireguard constraints in the existing settings blob.
    ///
    /// This can safely be cast to [`WireguardConstraints`]
    pub fn wireguard_constraints(settings: &mut Value) -> Option<&mut Value> {
        // relay_settings -> normal -> wireguard_constraints
        Self::relay_settings(settings)?.get_mut("wireguard_constraints")
    }

    /// A lens to the current wireguard / tunnel settings in the existing settings blob.
    ///
    /// This can safely be cast to [`WireguardSettings`]
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
    pub fn get_filters(&self) -> (Option<&Providers>, Option<&Ownership>) {
        let Some(settings) = self.relay_settings.normal.as_ref() else {
            return (None, None);
        };
        let providers = match &settings.providers {
            Constraint::Any => None,
            Constraint::Only(providers) => Some(providers),
        };
        let ownership = match &settings.ownership {
            Constraint::Any => None,
            Constraint::Only(ownership) => Some(ownership),
        };
        (providers, ownership)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelaySettings {
    // If the user has custom relay settings, the "normal" field will not be populated.
    normal: Option<RelaySettingsInner>,
}

impl Default for RelaySettings {
    fn default() -> Self {
        Self {
            normal: Some(Default::default()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelaySettingsInner {
    pub location: Constraint<LocationConstraint>,
    #[serde(default)]
    pub providers: Constraint<Providers>,
    #[serde(default)]
    pub ownership: Constraint<Ownership>,
    #[serde(default)]
    pub wireguard_constraints: WireguardConstraints,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireguardConstraints {
    pub use_multihop: bool,
    pub entry_location: Constraint<LocationConstraint>,
    pub entry_providers: Constraint<Providers>,
    pub entry_ownership: Constraint<Ownership>,
    // NOTE: This migration is not concerned with the following fields - leave them untouched!
    pub ip_version: Constraint<IpVersion>,
    pub allowed_ips: Constraint<AllowedIps>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TunnelOptions {
    pub wireguard: WireguardSettings,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireguardSettings {
    #[serde(default)]
    pub daita: DaitaSettings,
    // NOTE: This migration is not concerned with the following fields - leave them untouched!
    pub mtu: Value,
    pub quantum_resistant: Value,
    #[serde(default)]
    pub rotation_interval: Value,
}

/// The DAITA setting
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaitaSettings {
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
    providers: Constraint<Providers>,
    ownership: Constraint<Ownership>,
    entry_location: Constraint<LocationConstraint>,
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
        match value {
            true => self
                .providers(Providers(HashSet::from_iter(["31337".to_string()])))
                .ownership(Ownership::MullvadOwned),
            false => {
                self.providers = Constraint::Any;
                self.ownership = Constraint::Any;
                self
            }
        }
    }

    pub fn providers(mut self, value: Providers) -> Self {
        self.providers = Constraint::Only(value);
        self
    }

    pub fn ownership(mut self, value: Ownership) -> Self {
        self.ownership = Constraint::Only(value);
        self
    }

    pub fn build(self) -> Settings {
        let mut settings = Settings::default();
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
        settings.magic_multihop = self.magic_multihop.then_some(LocationConstraint::default());
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

//! Vendored types for the settings which this migration is migrating to.

use crate::migrations::Error;
use crate::migrations::multihop::settings::{
    AllowedIps, Constraint, IpVersion, LocationConstraint, Ownership, Providers, v17,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub relay_settings: RelaySettings,
    pub tunnel_options: TunnelOptions,
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
    location: Constraint<LocationConstraint>,
    providers: Constraint<Providers>,
    ownership: Constraint<Ownership>,
    wireguard_constraints: WireguardConstraints,
}

impl RelaySettingsInner {
    /// Update the multihop value of an existing settings blob to the new [`Multihop`] kind.
    ///
    /// If `filters` is true, copy the exit filters to the entry filters. To keep the legacy behavior where there was only one
    /// set of filters, but they applied for both entry and exit relays.
    ///
    /// If `automatic_entry` is true, then the entry relay location is overriden to [`Constraint::Any`].
    pub fn migrate(
        from: v17::RelaySettingsInner,
        multihop: Multihop,
        filters: bool,
        automatic_entry: bool,
    ) -> Self {
        let v17::RelaySettingsInner {
            location,
            providers,
            ownership,
            wireguard_constraints,
        } = from;

        let wireguard_constraints = {
            let v17::WireguardConstraints {
                use_multihop,
                entry_location,
                entry_providers,
                entry_ownership,
                ip_version,
                allowed_ips,
            } = wireguard_constraints;
            let (entry_providers, entry_ownership) = if filters {
                // Copy filters to entry.
                (providers.clone(), ownership.clone())
            } else {
                (entry_providers, entry_ownership)
            };
            let entry_location = if automatic_entry {
                Constraint::Any
            } else {
                entry_location
            };
            WireguardConstraints {
                multihop,
                entry_location,
                entry_providers,
                entry_ownership,
                ip_version,
                allowed_ips,
            }
        };

        Self {
            providers,
            ownership,
            wireguard_constraints,
            location,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireguardConstraints {
    pub multihop: Multihop,
    pub entry_location: Constraint<LocationConstraint>,
    pub entry_providers: Constraint<Providers>,
    pub entry_ownership: Constraint<Ownership>,
    // NOTE: This migration is not concerned with the following fields - leave them untouched!
    pub ip_version: Constraint<IpVersion>,
    pub allowed_ips: Constraint<AllowedIps>,
}

impl WireguardConstraints {
    pub fn migrate(from: v17::WireguardConstraints, multihop: Multihop, filters: bool) -> Self {
        Self {
            multihop,
            entry_location: from.entry_location,
            ip_version: from.ip_version,
            allowed_ips: from.allowed_ips,
            entry_providers: from.entry_providers,
            entry_ownership: from.entry_ownership,
        }
    }
}

/// New multihop setting.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Multihop {
    #[serde(rename = "always")]
    Always,
    #[serde(rename = "never")]
    Never,
    #[default]
    #[serde(rename = "auto")]
    WhenNeeded,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TunnelOptions {
    pub wireguard: WireguardSettings,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireguardSettings {
    pub daita: bool,
    // NOTE: This migration is not concerned with the following fields - leave them untouched!
    pub mtu: Value,
    pub quantum_resistant: Value,
    pub rotation_interval: Value,
}

impl WireguardSettings {
    pub fn migrate(from: v17::WireguardSettings) -> Self {
        Self {
            daita: from.daita.enabled,
            mtu: from.mtu,
            quantum_resistant: from.quantum_resistant,
            rotation_interval: from.rotation_interval,
        }
    }
}

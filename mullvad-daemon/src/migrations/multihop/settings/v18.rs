//! Vendored types for the settings which this migration is migrating to.

use crate::migrations::multihop::settings::{
    AllowedIps, Constraint, IpVersion, LocationConstraint, Ownership, Providers, v17,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
        duplicate_exit_filters: bool,
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
                use_multihop: _, // Simply override the previous value with `multihop`.
                entry_location,
                entry_providers,
                entry_ownership,
                ip_version,
                allowed_ips,
            } = wireguard_constraints;
            let (entry_providers, entry_ownership) = if duplicate_exit_filters {
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

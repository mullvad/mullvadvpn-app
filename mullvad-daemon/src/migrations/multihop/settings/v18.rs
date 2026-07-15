//! Vendored types for the settings which this migration is migrating to.

use crate::migrations::multihop::settings::{
    __AllowedIps, __Constraint, __IpVersion, __LocationConstraint, __Ownership, __Providers, v17,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct __RelaySettings {
    location: __Constraint<__LocationConstraint>,
    providers: __Constraint<__Providers>,
    ownership: __Constraint<__Ownership>,
    wireguard_constraints: __WireguardConstraints,
}

pub(crate) enum __Entry {
    Automatic(bool),
    LastKnownWorking(__LocationConstraint),
}

impl __RelaySettings {
    /// Update the multihop value of an existing settings blob to the new [`__Multihop`] kind.
    ///
    /// If `filters` is true, copy the exit filters to the entry filters. To keep the legacy behavior where there was only one
    /// set of filters, but they applied for both entry and exit relays.
    ///
    /// If `automatic_entry` is true, then the entry relay location is overriden to [`__Constraint::Any`].
    pub fn migrate(
        from: v17::RelaySettingsInner,
        multihop: __Multihop,
        duplicate_exit_filters: bool,
        automatic_entry: __Entry,
    ) -> Self {
        let v17::RelaySettingsInner {
            location,
            providers,
            ownership,
            wireguard_constraints,
        } = from;

        let wireguard_constraints = {
            let v17::__WireguardConstraints {
                use_multihop: _, // Simply override the previous value with `multihop`.
                entry_location,
                entry_providers,
                entry_ownership,
                ip_version,
                allowed_ips,
            } = wireguard_constraints;
            let (entry_providers, entry_ownership) = if duplicate_exit_filters {
                // Copy filters to entry.
                (providers.clone(), ownership)
            } else {
                (entry_providers, entry_ownership)
            };
            // TODO:  Grab this from magic_multihop
            let entry_location = match automatic_entry {
                __Entry::Automatic(true) => __Constraint::Any,
                __Entry::Automatic(false) => entry_location,
                __Entry::LastKnownWorking(this_entry) => __Constraint::Only(this_entry),
            };

            __WireguardConstraints {
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
pub struct __WireguardConstraints {
    pub multihop: __Multihop,
    pub entry_location: __Constraint<__LocationConstraint>,
    pub entry_providers: __Constraint<__Providers>,
    pub entry_ownership: __Constraint<__Ownership>,
    // NOTE: This migration is not concerned with the following fields - leave them untouched!
    pub ip_version: __Constraint<__IpVersion>,
    pub allowed_ips: __Constraint<__AllowedIps>,
}

/// New multihop setting.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum __Multihop {
    #[serde(rename = "always")]
    Always,
    #[serde(rename = "never")]
    Never,
    #[default]
    #[serde(rename = "auto")]
    WhenNeeded,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct __WireguardSettings {
    pub daita: bool,
    // NOTE: This migration is not concerned with the following fields - leave them untouched!
    pub mtu: Value,
    pub quantum_resistant: Value,
    pub rotation_interval: Value,
}

impl __WireguardSettings {
    pub fn migrate(from: v17::__WireguardSettings) -> Self {
        Self {
            daita: from.daita.enabled,
            mtu: from.mtu,
            quantum_resistant: from.quantum_resistant,
            rotation_interval: from.rotation_interval,
        }
    }
}

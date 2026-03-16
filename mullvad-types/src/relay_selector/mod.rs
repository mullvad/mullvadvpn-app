//! Types of `mullvad-relay-selector`.
//!
//! Most types in this module are equivalent to the ones in `mullvad-management-interface\proto\management_interface.proto`.
//! See the proto file for more documentation.

use talpid_types::net::IpVersion;

use crate::{
    constraints::Constraint,
    relay_constraints::{
        GeographicLocationConstraint, LocationConstraint, ObfuscationSettings, Ownership, Providers,
    },
    relay_list::WireguardRelay,
    wireguard::DaitaSettings,
};

/// Specify the constraints that should be applied when selecting relays,
/// along with a context that may affect the selection behavior.
#[derive(Debug, Clone)]
pub enum Predicate {
    Singlehop(EntryConstraints),
    Autohop(EntryConstraints),
    // Multihop-only
    Entry(MultihopConstraints),
    Exit(MultihopConstraints),
}

#[derive(Debug, Default, Clone)]
pub struct EntryConstraints {
    pub general: ExitConstraints,
    // Entry-specific constraints.
    pub obfuscation_settings: Constraint<ObfuscationSettings>,
    pub daita: Constraint<DaitaSettings>,
    pub ip_version: Constraint<IpVersion>,
}

#[derive(Debug, Default, Clone)]
pub struct ExitConstraints {
    pub location: Constraint<LocationConstraint>,
    pub providers: Constraint<Providers>,
    pub ownership: Constraint<Ownership>,
}

#[derive(Debug, Default, Clone)]
pub struct MultihopConstraints {
    pub entry: EntryConstraints,
    pub exit: ExitConstraints,
}

#[derive(Debug, Default, PartialEq)]
pub struct RelayPartitions {
    pub matches: Vec<WireguardRelay>,
    pub discards: Vec<(WireguardRelay, Vec<Reason>)>,
}

/// All possible reasons why a relay was filtered out for a particular query.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reason {
    /// This relay is already used for the other hop (entry/exit).
    Conflict,
    /// The relay does not run DAITA.
    Daita,
    /// The relay is currently offline.
    Inactive,
    /// The relay cannot be connected to with the requested ip version.
    IpVersion,
    /// The relay does not reside in the given location.
    Location,
    /// The requested obfuscation method is not available.
    Obfuscation,
    /// The relay ownership does not match.
    Ownership,
    /// The relay cannot be connected to via the requested port.
    Port,
    /// The relay is not hosted by the given provider.
    Providers,
}

// TODO: Should these be builders insteads?

impl EntryConstraints {
    pub fn daita(mut self, enabled: bool) -> Self {
        self.daita = Constraint::Only(DaitaSettings {
            enabled,
            // TODO: Remove `use_multihop_if_necessary` now when autohop exists?
            // Unused for partition relays, overridden by "Autohop" predicate.
            use_multihop_if_necessary: false,
        });
        self
    }

    pub fn general(mut self, general: ExitConstraints) -> Self {
        self.general = general;
        self
    }

    pub fn providers(mut self, providers: Providers) -> Self {
        self.general.providers = Constraint::Only(providers);
        self
    }

    pub fn ownership(mut self, ownership: Ownership) -> Self {
        self.general.ownership = Constraint::Only(ownership);
        self
    }

    pub fn obfuscation(mut self, obfuscation_settings: ObfuscationSettings) -> Self {
        self.obfuscation_settings = Constraint::Only(obfuscation_settings);
        self
    }

    pub fn ip_version(mut self, ip_version: IpVersion) -> Self {
        self.ip_version = Constraint::Only(ip_version);
        self
    }
}

impl ExitConstraints {
    pub fn location(mut self, location: impl Into<LocationConstraint>) -> Self {
        self.location = Constraint::Only(location.into());
        self
    }

    pub fn city(mut self, country: impl Into<String>, city: impl Into<String>) -> Self {
        self.location = Constraint::Only(GeographicLocationConstraint::city(country, city).into());
        self
    }

    pub fn country(mut self, country: impl Into<String>) -> Self {
        self.location = Constraint::Only(GeographicLocationConstraint::country(country).into());
        self
    }
}

impl MultihopConstraints {
    pub fn entry(mut self, entry: EntryConstraints) -> Self {
        self.entry = entry;
        self
    }

    pub fn exit(mut self, exit: ExitConstraints) -> Self {
        self.exit = exit;
        self
    }

    /// TODO
    pub fn exit_providers(mut self, providers: Providers) -> Self {
        self.exit.providers = Constraint::Only(providers);
        self
    }

    /// TODO
    pub fn exit_ownership(mut self, ownership: Ownership) -> Self {
        self.exit.ownership = Constraint::Only(ownership);
        self
    }
}

//! Types of `mullvad-relay-selector`.
//!
//! Most types in this module are equivalent to the ones in `mullvad-management-interface\proto\management_interface.proto`.
//! See the proto file for more documentation.

use talpid_types::net::IpVersion;

use crate::{
    constraints::Constraint,
    relay_constraints::{LocationConstraint, ObfuscationSettings, Ownership, Providers},
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
    pub providers: Constraint<Providers>,
    pub ownership: Constraint<Ownership>,
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

//! This module provides a flexible way to specify 'queries' for relays.
//!
//! A query is a set of constraints that the [`crate::RelaySelector`] uses when filtering relays
//! that the daemon should connect to. The query carries:
//!
//! - the **hop count** the user wants ([`Hops::Single`], [`Hops::Auto`], or
//!   [`Hops::Multi`]) along with the entry/exit constraints meaningful to that count;
//! - **connection-level** constraints (`allowed_ips`, `quantum_resistant`) that apply regardless of
//!   the count.
//!
//! [`RelayQuery`] is built either from the user's [`Settings`] (via `From`) or with the
//! [`builder::RelayQueryBuilder`] fluent API used in tests.

pub use mullvad_types::relay_constraints::{
    ObfuscationMode, obfuscation_constraint_from_settings, obfuscation_to_settings,
};
use mullvad_types::{
    Intersection,
    constraints::Constraint,
    relay_constraints::{
        AllowedIps, ObfuscationSettings, RelayConstraints, RelaySettings, WireguardConstraints,
    },
    relay_selector::{
        EntryConstraints, EntrySpecificConstraints, ExitConstraints, MultihopConstraints,
    },
    settings::Settings,
    wireguard::QuantumResistantState,
};
use talpid_types::net::{IpAvailability, IpVersion};

use crate::Error;

/// A query for a relay.
///
/// A [`RelayQuery`] is best constructed via the fluent builder API exposed by
/// [`builder::RelayQueryBuilder`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayQuery {
    /// Constraints depending on hop variant.
    /// Only this field affects relay selection.
    pub hops: Hops,
    pub allowed_ips: Constraint<AllowedIps>,
    pub quantum_resistant: Constraint<QuantumResistantState>,
}

/// The multihop variant and corresponding constraints on each hop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hops {
    /// Use a single relay satisfying both entry and exit criteria.
    Single(EntryConstraints),
    /// Singlehop preferred; falls back to multihop with an auto-selected entry.
    Auto(EntryConstraints),
    /// Multihop, i.e. two hops with distinct entry and exit constraints.
    Multi(MultihopConstraints),
}

impl RelayQuery {
    /// Tighten `self`'s entry-specific constraints by intersecting them with `retry`.
    ///
    /// Returns `None` when the retry's constraints are incompatible with the user's
    /// — e.g., the user pinned ip_version=v4 and the retry wants v6. The retry order
    /// only ever modifies obfuscation and ip_version, both of which live in
    /// [`EntrySpecificConstraints`], so this is the only kind of merging needed.
    pub fn merge_retry(mut self, retry: EntrySpecificConstraints) -> Option<Self> {
        let merged = self.entry_specific().clone().intersection(retry)?;
        *self.entry_specific_mut() = merged;
        Some(self)
    }

    // ---------------------------------------------------------------------------
    // Accessors — uniform regardless of variant
    // ---------------------------------------------------------------------------

    /// The constraints on the relay the client connects to first. For singlehop and
    /// autohop, this is the single (or singlehop-attempt) relay; for multihop, it's
    /// the entry relay.
    pub fn entry(&self) -> &EntryConstraints {
        match &self.hops {
            Hops::Single(e) | Hops::Auto(e) => e,
            Hops::Multi(MultihopConstraints { entry, .. }) => entry,
        }
    }

    pub fn entry_mut(&mut self) -> &mut EntryConstraints {
        match &mut self.hops {
            Hops::Single(e) | Hops::Auto(e) => e,
            Hops::Multi(MultihopConstraints { entry, .. }) => entry,
        }
    }

    pub fn entry_specific(&self) -> &EntrySpecificConstraints {
        &self.entry().entry_specific
    }

    pub fn entry_specific_mut(&mut self) -> &mut EntrySpecificConstraints {
        &mut self.entry_mut().entry_specific
    }

    /// The constraints on the relay traffic exits the Mullvad network through.
    /// For singlehop/autohop, that's the same relay the client connects to; for
    /// multihop, it's the exit hop.
    pub fn exit(&self) -> &ExitConstraints {
        match &self.hops {
            Hops::Single(EntryConstraints { general, .. })
            | Hops::Auto(EntryConstraints { general, .. }) => general,
            Hops::Multi(MultihopConstraints { exit, .. }) => exit,
        }
    }

    pub fn exit_mut(&mut self) -> &mut ExitConstraints {
        match &mut self.hops {
            Hops::Single(EntryConstraints { general, .. })
            | Hops::Auto(EntryConstraints { general, .. }) => general,
            Hops::Multi(MultihopConstraints { exit, .. }) => exit,
        }
    }

    pub fn apply_ip_availability(
        &mut self,
        runtime_ip_availability: IpAvailability,
    ) -> Result<(), Error> {
        let runtime_ip = match runtime_ip_availability {
            IpAvailability::Ipv4 => Constraint::Only(IpVersion::V4),
            IpAvailability::Ipv6 => Constraint::Only(IpVersion::V6),
            IpAvailability::Ipv4AndIpv6 => Constraint::Any,
        };

        let entry_specific = self.entry_specific_mut();
        let merged = entry_specific
            .ip_version
            .intersection(runtime_ip)
            .ok_or_else(|| {
                // intersection returns None only when both sides are `Constraint::Only`
                // and disagree, so unwrapping here is safe.
                Error::IpVersionUnavailable {
                    family: entry_specific.ip_version.unwrap(),
                }
            })?;
        entry_specific.ip_version = merged;
        Ok(())
    }
}

impl From<&Settings> for RelayQuery {
    fn from(settings: &Settings) -> Self {
        let RelaySettings::Normal(relay_settings) = &settings.relay_settings else {
            // Custom tunnel endpoints bypass the relay selector entirely — return a
            // dormant default. Callers that care about custom endpoints check
            // `relay_settings` themselves before consulting the relay selector.
            return Self::default();
        };

        #[cfg(daita)]
        let (daita, daita_use_multihop_if_necessary) = (
            settings.tunnel_options.wireguard.daita.enabled,
            settings
                .tunnel_options
                .wireguard
                .daita
                .use_multihop_if_necessary,
        );
        #[cfg(not(daita))]
        let (daita, daita_use_multihop_if_necessary) = (false, false);

        let wg = &relay_settings.wireguard_constraints;

        let entry_specific = EntrySpecificConstraints {
            obfuscation: obfuscation_constraint_from_settings(
                settings.obfuscation_settings.clone(),
            ),
            daita: Constraint::Only(daita),
            ip_version: wg.ip_version,
        };
        let exit = ExitConstraints {
            location: relay_settings.location.clone(),
            providers: relay_settings.providers.clone(),
            ownership: relay_settings.ownership,
        };

        let singlehop = |entry_specific, exit| EntryConstraints {
            entry_specific,
            general: exit,
        };

        // Currently, the `location`, `providers`, and `ownership` fields in the relay settings refer to the exit in a multihop configuration,
        // and the entry its corresponding settings from the entry_* fields. However, the entry specific constraints (obfuscation, ip_version, daita)
        // still refer to the entry. Thus we, need to shuffle the fields around to construct the multihop query.
        // TODO: When changing to dedicated multihop variants in settings, we should restructure the settings to match the query structure.
        let multihop_constraints = |entry_specific, exit| MultihopConstraints {
            entry: EntryConstraints {
                entry_specific,
                general: ExitConstraints {
                    location: wg.entry_location.clone(),
                    providers: wg.entry_providers.clone(),
                    ownership: wg.entry_ownership,
                },
            },
            exit,
        };

        // Currently, the autohop functionality is exclusive to DAITA, which is bound to change in the near future.
        // For now, encode the autohop preference as a combination of `daita = true` and `use_multihop_if_necessary = true`.
        // If multihop itself is disabled, this because the "when needed" mode. If it's enabled, we map it to multihop on
        // with an auto-picked entry.
        let autohop = daita && daita_use_multihop_if_necessary;

        let hops = match (wg.use_multihop, autohop) {
            (false, false) => Hops::Single(singlehop(entry_specific, exit)),
            // Multihop "when needed" (preference for singlehop)
            (false, true) => Hops::Auto(singlehop(entry_specific, exit)),
            // User-configured multihop
            (true, false) => Hops::Multi(multihop_constraints(entry_specific, exit)),
            // Multihop with auto entry
            (true, true) => Hops::Multi(singlehop(entry_specific, exit).into_autohop()),
        };

        RelayQuery {
            hops,
            allowed_ips: wg.allowed_ips.clone(),
            quantum_resistant: Constraint::Only(
                settings.tunnel_options.wireguard.quantum_resistant,
            ),
        }
    }
}

impl RelayQuery {
    /// Convert the query into RelayConstraints and ObfuscationSettings.
    /// Currently only used by the e2e tests via gRPC.
    pub fn into_settings(self) -> (RelayConstraints, ObfuscationSettings) {
        let RelayQuery {
            hops,
            allowed_ips,
            quantum_resistant: _,
        } = self;

        match hops {
            Hops::Single(entry) | Hops::Auto(entry) => {
                let constraints = RelayConstraints {
                    location: entry.general.location,
                    providers: entry.general.providers,
                    ownership: entry.general.ownership,
                    wireguard_constraints: WireguardConstraints {
                        ip_version: entry.entry_specific.ip_version,
                        allowed_ips,
                        use_multihop: false,
                        entry_location: Constraint::Any,
                        entry_providers: Constraint::Any,
                        entry_ownership: Constraint::Any,
                    },
                };
                let obfuscation = obfuscation_to_settings(entry.entry_specific.obfuscation);
                (constraints, obfuscation)
            }
            Hops::Multi(MultihopConstraints { entry, exit }) => {
                let constraints = RelayConstraints {
                    location: exit.location,
                    providers: exit.providers,
                    ownership: exit.ownership,
                    wireguard_constraints: WireguardConstraints {
                        ip_version: entry.entry_specific.ip_version,
                        allowed_ips,
                        use_multihop: true,
                        entry_location: entry.general.location,
                        entry_providers: entry.general.providers,
                        entry_ownership: entry.general.ownership,
                    },
                };
                let obfuscation = obfuscation_to_settings(entry.entry_specific.obfuscation);
                (constraints, obfuscation)
            }
        }
    }
}

impl Default for RelayQuery {
    /// "Singlehop, no constraints." This matches every relay with any configuration
    /// by setting each constraint field to [`Constraint::Any`].
    fn default() -> Self {
        Self {
            hops: Hops::Single(EntryConstraints::default()),
            allowed_ips: Constraint::Any,
            quantum_resistant: Constraint::Any,
        }
    }
}

pub mod builder {
    //! Strongly typed builder for [`RelayQuery`] using the typestate pattern.
    //!
    //! The typestate gates context-sensitive setters: `Multihop` gates `.entry_*`,
    //! `Obfuscation` gates `.udp2tcp_port()` / `.shadowsocks_port()`.
    use std::marker::PhantomData;

    use mullvad_types::{
        constraints::Constraint,
        relay_constraints::{
            LocationConstraint, LwoSettings, ShadowsocksSettings, Udp2TcpObfuscationSettings,
            WireguardPortSettings,
        },
        relay_selector::{
            EntryConstraints, EntrySpecificConstraints, ExitConstraints, MultihopConstraints,
        },
        wireguard::QuantumResistantState,
    };

    use super::{Hops, ObfuscationMode, RelayQuery};

    // Re-exports
    pub use mullvad_types::relay_constraints::{
        AllowedIps, GeographicLocationConstraint, Ownership, Providers,
    };
    pub use talpid_types::net::{IpVersion, TransportProtocol};

    /// `Any` is the typestate for "no choice made yet" on a given axis.
    pub struct Any;

    /// QUIC obfuscation typestate (no user-configurable parameters).
    pub struct Quic;

    /// LWO obfuscation typestate (no user-configurable parameters at builder level).
    pub struct Lwo;

    // TODO: Convert to type parameter and make builder typed on Hop count?
    #[derive(Default)]
    enum HopChoice {
        #[default]
        Singlehop,
        Autohop,
        Multihop,
    }

    /// Builder for [`RelayQuery`].
    ///
    /// - `Multihop` — `Any` until [`Self::multihop`] is called, then `bool` (gates
    ///   `.entry_*` setters).
    /// - `Obfuscation` — `Any` until an obfuscation method is chosen, then a type
    ///   carrying the in-flight settings so port setters can mutate them.
    pub struct RelayQueryBuilder<Multihop = Any, Obfuscation = Any> {
        entry_specific: EntrySpecificConstraints,
        exit: ExitConstraints,
        /// Only meaningful when `hop_choice == Multihop`.
        multihop_entry: ExitConstraints,
        hop_choice: HopChoice,
        allowed_ips: Constraint<AllowedIps>,
        quantum_resistant: Constraint<QuantumResistantState>,
        obfuscation_state: Obfuscation,
        _phantom: PhantomData<Multihop>,
    }

    impl RelayQueryBuilder<Any, Any> {
        pub fn new() -> Self {
            Self {
                entry_specific: EntrySpecificConstraints::default(),
                exit: ExitConstraints::default(),
                multihop_entry: ExitConstraints::default(),
                hop_choice: HopChoice::default(),
                allowed_ips: Constraint::Any,
                quantum_resistant: Constraint::Any,
                obfuscation_state: Any,
                _phantom: PhantomData,
            }
        }
    }

    impl Default for RelayQueryBuilder<Any, Any> {
        fn default() -> Self {
            Self::new()
        }
    }

    // Methods available regardless of typestate.
    impl<Multihop, Obfuscation> RelayQueryBuilder<Multihop, Obfuscation> {
        /// Configure the exit relay's location. (For singlehop/autohop, the exit is
        /// the only relay; for multihop, the exit is the second hop.)
        pub fn location(mut self, location: impl Into<LocationConstraint>) -> Self {
            self.exit.location = Constraint::Only(location.into());
            self
        }

        pub const fn ownership(mut self, ownership: Ownership) -> Self {
            self.exit.ownership = Constraint::Only(ownership);
            self
        }

        pub fn providers(mut self, providers: Providers) -> Self {
            self.exit.providers = Constraint::Only(providers);
            self
        }

        pub const fn ip_version(mut self, ip_version: IpVersion) -> Self {
            self.entry_specific.ip_version = Constraint::Only(ip_version);
            self
        }

        pub const fn daita(mut self) -> Self {
            self.entry_specific.daita = Constraint::Only(true);
            self
        }

        pub fn quantum_resistant(mut self) -> Self {
            self.quantum_resistant = Constraint::Only(QuantumResistantState::On);
            self
        }

        /// Switch to the autohop. Falls back from singlehop to multihop when
        /// no singlehop relay matches the constraints.
        ///
        /// Under the current temporary encoding this implies `daita = true`. When the
        /// standalone autohop setting lands, drop the daita coupling.
        pub fn autohop(mut self) -> Self {
            self.hop_choice = HopChoice::Autohop;
            self.entry_specific.daita = Constraint::Only(true);
            self
        }

        /// Assemble the final [`RelayQuery`].
        pub fn build(self) -> RelayQuery {
            let hops = match self.hop_choice {
                HopChoice::Singlehop => Hops::Single(EntryConstraints {
                    general: self.exit,
                    entry_specific: self.entry_specific,
                }),
                HopChoice::Autohop => Hops::Auto(EntryConstraints {
                    general: self.exit,
                    entry_specific: self.entry_specific,
                }),
                HopChoice::Multihop => Hops::Multi(MultihopConstraints {
                    entry: EntryConstraints {
                        general: self.multihop_entry,
                        entry_specific: self.entry_specific,
                    },
                    exit: self.exit,
                }),
            };
            RelayQuery {
                hops,
                allowed_ips: self.allowed_ips,
                quantum_resistant: self.quantum_resistant,
            }
        }
    }

    // `.multihop()` transitions the typestate so `.entry_*` becomes available.
    impl<Obfuscation> RelayQueryBuilder<Any, Obfuscation> {
        pub fn multihop(mut self) -> RelayQueryBuilder<bool, Obfuscation> {
            self.hop_choice = HopChoice::Multihop;
            RelayQueryBuilder {
                entry_specific: self.entry_specific,
                exit: self.exit,
                multihop_entry: self.multihop_entry,
                hop_choice: self.hop_choice,
                allowed_ips: self.allowed_ips,
                quantum_resistant: self.quantum_resistant,
                obfuscation_state: self.obfuscation_state,
                _phantom: PhantomData,
            }
        }
    }

    // `.entry_*` only available after `.multihop()`.
    impl<Obfuscation> RelayQueryBuilder<bool, Obfuscation> {
        pub fn entry(mut self, location: impl Into<LocationConstraint>) -> Self {
            self.multihop_entry.location = Constraint::Only(location.into());
            self
        }
        pub fn entry_providers(mut self, providers: Providers) -> Self {
            self.multihop_entry.providers = Constraint::Only(providers);
            self
        }
        pub const fn entry_ownership(mut self, ownership: Ownership) -> Self {
            self.multihop_entry.ownership = Constraint::Only(ownership);
            self
        }
    }

    // Obfuscation-mode setters: each transitions the typestate so the matching port
    // setter (if any) becomes available.
    impl<Multihop> RelayQueryBuilder<Multihop, Any> {
        /// Pin the WireGuard port via the `Port` obfuscation mode (no actual
        /// obfuscation, just port pinning).
        pub fn port(mut self, port: u16) -> RelayQueryBuilder<Multihop, WireguardPortSettings> {
            let port = WireguardPortSettings::from(port);
            self.entry_specific.obfuscation = Constraint::Only(ObfuscationMode::Port(port));
            RelayQueryBuilder {
                entry_specific: self.entry_specific,
                exit: self.exit,
                multihop_entry: self.multihop_entry,
                hop_choice: self.hop_choice,
                allowed_ips: self.allowed_ips,
                quantum_resistant: self.quantum_resistant,
                obfuscation_state: port,
                _phantom: PhantomData,
            }
        }

        pub fn shadowsocks(mut self) -> RelayQueryBuilder<Multihop, ShadowsocksSettings> {
            let settings = ShadowsocksSettings::default();
            self.entry_specific.obfuscation =
                Constraint::Only(ObfuscationMode::Shadowsocks(settings.clone()));
            RelayQueryBuilder {
                entry_specific: self.entry_specific,
                exit: self.exit,
                multihop_entry: self.multihop_entry,
                hop_choice: self.hop_choice,
                allowed_ips: self.allowed_ips,
                quantum_resistant: self.quantum_resistant,
                obfuscation_state: settings,
                _phantom: PhantomData,
            }
        }

        pub fn udp2tcp(mut self) -> RelayQueryBuilder<Multihop, Udp2TcpObfuscationSettings> {
            let settings = Udp2TcpObfuscationSettings::default();
            self.entry_specific.obfuscation =
                Constraint::Only(ObfuscationMode::Udp2tcp(settings.clone()));
            RelayQueryBuilder {
                entry_specific: self.entry_specific,
                exit: self.exit,
                multihop_entry: self.multihop_entry,
                hop_choice: self.hop_choice,
                allowed_ips: self.allowed_ips,
                quantum_resistant: self.quantum_resistant,
                obfuscation_state: settings,
                _phantom: PhantomData,
            }
        }

        pub fn quic(mut self) -> RelayQueryBuilder<Multihop, Quic> {
            self.entry_specific.obfuscation = Constraint::Only(ObfuscationMode::Quic);
            RelayQueryBuilder {
                entry_specific: self.entry_specific,
                exit: self.exit,
                multihop_entry: self.multihop_entry,
                hop_choice: self.hop_choice,
                allowed_ips: self.allowed_ips,
                quantum_resistant: self.quantum_resistant,
                obfuscation_state: Quic,
                _phantom: PhantomData,
            }
        }

        pub fn lwo(mut self) -> RelayQueryBuilder<Multihop, Lwo> {
            self.entry_specific.obfuscation =
                Constraint::Only(ObfuscationMode::Lwo(LwoSettings::default()));
            RelayQueryBuilder {
                entry_specific: self.entry_specific,
                exit: self.exit,
                multihop_entry: self.multihop_entry,
                hop_choice: self.hop_choice,
                allowed_ips: self.allowed_ips,
                quantum_resistant: self.quantum_resistant,
                obfuscation_state: Lwo,
                _phantom: PhantomData,
            }
        }
    }

    impl<Multihop> RelayQueryBuilder<Multihop, Udp2TcpObfuscationSettings> {
        pub fn udp2tcp_port(mut self, port: u16) -> Self {
            self.obfuscation_state.port = Constraint::Only(port);
            self.entry_specific.obfuscation =
                Constraint::Only(ObfuscationMode::Udp2tcp(self.obfuscation_state.clone()));
            self
        }
    }
}

#[cfg(test)]
mod test {
    use mullvad_types::{
        constraints::Constraint,
        relay_constraints::{
            LwoSettings, ObfuscationSettings, SelectedObfuscation, ShadowsocksSettings,
            Udp2TcpObfuscationSettings,
        },
    };
    use proptest::prelude::*;

    use super::Intersection;

    // Define proptest combinators for the `Constraint` type.

    pub fn constraint<T>(
        base_strategy: impl Strategy<Value = T> + 'static,
    ) -> impl Strategy<Value = Constraint<T>>
    where
        T: core::fmt::Debug + std::clone::Clone + 'static,
    {
        prop_oneof![any(), only(base_strategy),]
    }

    pub fn only<T>(
        base_strategy: impl Strategy<Value = T> + 'static,
    ) -> impl Strategy<Value = Constraint<T>>
    where
        T: core::fmt::Debug + std::clone::Clone + 'static,
    {
        base_strategy.prop_map(Constraint::Only)
    }

    pub fn any<T>() -> impl Strategy<Value = Constraint<T>>
    where
        T: core::fmt::Debug + std::clone::Clone + 'static,
    {
        Just(Constraint::Any)
    }

    proptest! {
        #[test]
        fn identity(x in only(proptest::arbitrary::any::<bool>())) {
            // Identity laws
            //  x ∩ identity = x
            //  identity ∩ x = x

            // The identity element
            let identity = Constraint::Any;
            prop_assert_eq!(x.intersection(identity), x.into());
            prop_assert_eq!(identity.intersection(x), x.into());
        }

        #[test]
        fn idempotency (x in constraint(proptest::arbitrary::any::<bool>())) {
            // Idempotency law
            //  x ∩ x = x
            prop_assert_eq!(x.intersection(x), x.into()) // lift x to the return type of `intersection`
        }

        #[test]
        fn commutativity(x in constraint(proptest::arbitrary::any::<bool>()),
                         y in constraint(proptest::arbitrary::any::<bool>())) {
            // Commutativity law
            //  x ∩ y = y ∩ x
            prop_assert_eq!(x.intersection(y), y.intersection(x))
        }

        #[test]
        fn associativity(x in constraint(proptest::arbitrary::any::<bool>()),
                         y in constraint(proptest::arbitrary::any::<bool>()),
                         z in constraint(proptest::arbitrary::any::<bool>()))
        {
            // Associativity law
            //  (x ∩ y) ∩ z = x ∩ (y ∩ z)
            let left: Option<_> = {
                x.intersection(y).and_then(|xy| xy.intersection(z))
            };
            let right: Option<_> = {
                // It is fine to rewrite the order of the application from
                //  x ∩ (y ∩ z)
                // to
                //  (y ∩ z) ∩ x
                // due to the commutative property of intersection
                (y.intersection(z)).and_then(|yz| yz.intersection(x))
            };
            prop_assert_eq!(left, right);
        }

        /// When obfuscation is set to automatic in [`ObfuscationSettings`], the query should
        /// convert to `Constraint::Any`.
        #[test]
        fn test_auto_obfuscation_settings(port1 in constraint(proptest::arbitrary::any::<u16>()), port2 in constraint(proptest::arbitrary::any::<u16>())) {
            let query = super::obfuscation_constraint_from_settings(ObfuscationSettings {
                selected_obfuscation: SelectedObfuscation::Auto,
                udp2tcp: Udp2TcpObfuscationSettings {
                    port: port1,
                },
                shadowsocks: ShadowsocksSettings {
                    port: port2,
                },
                wireguard_port: port1.into(),
                lwo: LwoSettings { port: port1 },
            });
            assert_eq!(query, Constraint::Any);
        }
    }
}

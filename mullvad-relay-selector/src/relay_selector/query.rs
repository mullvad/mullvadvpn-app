//! This module provides a flexible way to specify 'queries' for relays.
//!
//! A query is a set of constraints that the [`crate::RelaySelector`] will use when filtering out
//! potential relays that the daemon should connect to. It supports filtering relays by geographic
//! location, provider, ownership, and tunnel protocol, along with protocol-specific settings for
//! WireGuard.
//!
//! The main components of this module include:
//!
//! - [`RelayQuery`]: The core struct for specifying a query to select relay servers. It aggregates
//!   constraints on location, providers, ownership, tunnel protocol, and protocol-specific
//!   constraints for WireGuard.
//! - [`WireguardRelayQuery`]: Struct that defines protocol-specific constraints for selecting
//!   WireGuard relays.
//! - [`Intersection`]: A trait implemented by the different query types that support intersection
//!   logic, which allows for combining two queries into a single query that represents the common
//!   constraints of both.
//! - [Builder patterns][builder]: The module also provides builder patterns for creating instances
//!   of `RelayQuery`, and `WireguardRelayQuery` with a fluent API.
//!
//! ## Design
//!
//! This module has been built in such a way that it should be easy to reason about,
//! while providing a flexible and easy-to-use API. The `Intersection` trait provides
//! a robust framework for combining and refining queries based on multiple criteria.
//!
//! The builder patterns included in the module simplify the process of constructing
//! queries and ensure that queries are built in a type-safe manner, reducing the risk
//! of runtime errors and improving code readability.

use mullvad_types::{
    Intersection,
    constraints::Constraint,
    relay_constraints::{
        BridgeConstraints, LocationConstraint, ObfuscationSettings, Ownership, Providers,
        RelayConstraints, RelaySettings, SelectedObfuscation, ShadowsocksSettings,
        Udp2TcpObfuscationSettings, WireguardConstraints, allowed_ip::AllowedIps,
    },
    wireguard::QuantumResistantState,
};
use talpid_types::net::{IpVersion, proxy::CustomProxy};

/// Represents a query for a relay based on various constraints.
///
/// This struct contains constraints for the location, providers, ownership,
/// tunnel protocol, and additional protocol-specific constraints for WireGuard.
/// These constraints are used by the [`crate::RelaySelector`] to
/// filter and select suitable relay servers that match the specified criteria.
///
/// A [`RelayQuery`] is best constructed via the fluent builder API exposed by
/// [`builder::RelayQueryBuilder`].
///
/// # Examples
///
/// Creating a basic `RelayQuery` to filter relays by location, ownership and tunnel protocol:
///
/// ```rust
/// // Create a query for a Wireguard relay that is owned by Mullvad and located in Norway.
/// // The endpoint should specify port 443.
/// use mullvad_relay_selector::query::RelayQuery;
/// use mullvad_relay_selector::query::builder::RelayQueryBuilder;
/// use mullvad_relay_selector::query::builder::{Ownership, GeographicLocationConstraint};
///
/// let query: RelayQuery = RelayQueryBuilder::new()            // Specify the tunnel protocol
///     .location(GeographicLocationConstraint::country("no"))  // Specify the country as Norway
///     .ownership(Ownership::MullvadOwned)                     // Specify that the relay must be owned by Mullvad
///     .port(443)                                              // Specify the port to use when connecting to the relay
///     .build();                                               // Construct the query
/// ```
///
/// This example demonstrates creating a `RelayQuery` which can then be passed
/// to the [`crate::RelaySelector`] to find a relay that matches the criteria.
/// See [`builder`] for more info on how to construct queries.
#[derive(Debug, Clone, Eq, PartialEq, Intersection)]
pub struct RelayQuery {
    location: Constraint<LocationConstraint>,
    providers: Constraint<Providers>,
    ownership: Constraint<Ownership>,
    wireguard_constraints: WireguardRelayQuery,
}

impl RelayQuery {
    /// Create a new [`RelayQuery`].
    pub fn new(
        location: Constraint<LocationConstraint>,
        providers: Constraint<Providers>,
        ownership: Constraint<Ownership>,
        wireguard_constraints: WireguardRelayQuery,
    ) -> RelayQuery {
        RelayQuery {
            location,
            providers,
            ownership,
            wireguard_constraints,
        }
    }

    pub fn location(&self) -> &Constraint<LocationConstraint> {
        &self.location
    }

    pub fn set_location(&mut self, location: Constraint<LocationConstraint>) {
        self.location = location;
    }

    pub fn providers(&self) -> &Constraint<Providers> {
        &self.providers
    }

    pub fn set_providers(&mut self, providers: Constraint<Providers>) {
        self.providers = providers;
    }

    pub fn ownership(&self) -> Constraint<Ownership> {
        self.ownership
    }

    pub fn set_ownership(&mut self, ownership: Constraint<Ownership>) {
        self.ownership = ownership;
    }

    pub fn wireguard_constraints(&self) -> &WireguardRelayQuery {
        &self.wireguard_constraints
    }

    pub fn into_wireguard_constraints(self) -> WireguardRelayQuery {
        self.wireguard_constraints
    }

    pub fn set_wireguard_constraints(&mut self, wireguard_constraints: WireguardRelayQuery) {
        self.wireguard_constraints = wireguard_constraints;
    }

    /// The mapping from [`RelayQuery`] to all underlying settings types.
    ///
    /// Useful in contexts where you cannot use the query directly but
    /// still want use of the builder for convenience. For example in
    /// end to end tests where you must use the management interface
    /// to apply settings to the daemon.
    pub fn into_settings(self) -> (RelayConstraints, ObfuscationSettings) {
        let obfuscation = self
            .wireguard_constraints
            .obfuscation
            .clone()
            .into_settings();
        let constraints = RelayConstraints {
            location: self.location,
            providers: self.providers,
            ownership: self.ownership,
            wireguard_constraints: self.wireguard_constraints.into_constraints(),
        };

        (constraints, obfuscation)
    }
}

impl Default for RelayQuery {
    /// Create a new [`RelayQuery`] with no opinionated defaults. This query matches every relay
    /// with any configuration by setting each of its fields to [`Constraint::Any`].
    ///
    /// Note that the following identity applies for any `other_query`:
    /// ```rust
    /// # use mullvad_relay_selector::query::RelayQuery;
    /// # use mullvad_types::Intersection;
    ///
    /// # let other_query = RelayQuery::default();
    /// assert_eq!(RelayQuery::default().intersection(other_query.clone()), Some(other_query));
    /// # let other_query = RelayQuery::default();
    /// assert_eq!(other_query.clone().intersection(RelayQuery::default()), Some(other_query));
    /// ```
    fn default() -> Self {
        RelayQuery {
            location: Constraint::Any,
            providers: Constraint::Any,
            ownership: Constraint::Any,
            wireguard_constraints: WireguardRelayQuery::new(),
        }
    }
}

impl From<RelayQuery> for RelaySettings {
    fn from(query: RelayQuery) -> Self {
        let (relay_constraints, ..) = query.into_settings();
        RelaySettings::from(relay_constraints)
    }
}

/// A query for a relay with Wireguard-specific properties, such as `multihop` and [wireguard
/// obfuscation][`SelectedObfuscation`].
///
/// This struct may look a lot like [`WireguardConstraints`], and that is the point!
/// This struct is meant to be that type in the "universe of relay queries". The difference
/// between them may seem subtle, but in a [`WireguardRelayQuery`] every field is represented
/// as a [`Constraint`], which allow us to implement [`Intersection`] in a straight forward manner.
/// Notice that [obfuscation][`SelectedObfuscation`] is not a [`Constraint`], but it is trivial
/// to define [`Intersection`] on it, so it is fine.
#[derive(Debug, Clone, Eq, PartialEq, Intersection)]
pub struct WireguardRelayQuery {
    pub port: Constraint<u16>,
    pub ip_version: Constraint<IpVersion>,
    pub allowed_ips: Constraint<AllowedIps>,
    pub use_multihop: Constraint<bool>,
    pub entry_location: Constraint<LocationConstraint>,
    pub entry_providers: Constraint<Providers>,
    pub entry_ownership: Constraint<Ownership>,
    pub obfuscation: ObfuscationQuery,
    pub daita: Constraint<bool>,
    pub daita_use_multihop_if_necessary: Constraint<bool>,
    pub quantum_resistant: Constraint<QuantumResistantState>,
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum ObfuscationQuery {
    Off,
    #[default]
    Auto,
    Port, // If this is enabled, we should respect the port contstraint.
    Udp2tcp(Udp2TcpObfuscationSettings),
    Shadowsocks(ShadowsocksSettings),
    Quic,
    Lwo,
}

impl ObfuscationQuery {
    fn into_settings(self) -> ObfuscationSettings {
        let selected_obfuscation = match self {
            ObfuscationQuery::Off => SelectedObfuscation::Off,
            ObfuscationQuery::Auto => SelectedObfuscation::Auto,
            ObfuscationQuery::Port => SelectedObfuscation::Port,
            ObfuscationQuery::Quic => SelectedObfuscation::Quic,
            ObfuscationQuery::Lwo => SelectedObfuscation::Lwo,
            ObfuscationQuery::Udp2tcp(settings) => {
                return ObfuscationSettings {
                    selected_obfuscation: SelectedObfuscation::Udp2Tcp,
                    udp2tcp: settings,
                    ..Default::default()
                };
            }
            ObfuscationQuery::Shadowsocks(settings) => {
                return ObfuscationSettings {
                    selected_obfuscation: SelectedObfuscation::Shadowsocks,
                    shadowsocks: settings,
                    ..Default::default()
                };
            }
        };
        ObfuscationSettings {
            selected_obfuscation,
            ..Default::default()
        }
    }
}

impl From<ObfuscationSettings> for ObfuscationQuery {
    /// A query for obfuscation settings.
    ///
    /// Note that this drops obfuscation protocol specific constraints from [`ObfuscationSettings`]
    /// when the selected obfuscation type is auto.
    fn from(obfuscation: ObfuscationSettings) -> Self {
        use SelectedObfuscation::*;
        match obfuscation.selected_obfuscation {
            Off => ObfuscationQuery::Off,
            Auto => ObfuscationQuery::Auto,
            Port => ObfuscationQuery::Port,
            Udp2Tcp => ObfuscationQuery::Udp2tcp(obfuscation.udp2tcp),
            Shadowsocks => ObfuscationQuery::Shadowsocks(obfuscation.shadowsocks),
            Quic => ObfuscationQuery::Quic,
            Lwo => ObfuscationQuery::Lwo,
        }
    }
}

impl Intersection for ObfuscationQuery {
    fn intersection(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ObfuscationQuery::Off, _) | (_, ObfuscationQuery::Off) => Some(ObfuscationQuery::Off),
            (ObfuscationQuery::Auto, other) | (other, ObfuscationQuery::Auto) => Some(other),
            (ObfuscationQuery::Udp2tcp(a), ObfuscationQuery::Udp2tcp(b)) => {
                Some(ObfuscationQuery::Udp2tcp(a.intersection(b)?))
            }
            (ObfuscationQuery::Shadowsocks(a), ObfuscationQuery::Shadowsocks(b)) => {
                Some(ObfuscationQuery::Shadowsocks(a.intersection(b)?))
            }
            _ => None,
        }
    }
}

impl WireguardRelayQuery {
    pub fn multihop(&self) -> bool {
        matches!(self.use_multihop, Constraint::Only(true))
    }
}

impl WireguardRelayQuery {
    pub fn new() -> WireguardRelayQuery {
        WireguardRelayQuery {
            port: Constraint::Any,
            ip_version: Constraint::Any,
            allowed_ips: Constraint::Any,
            use_multihop: Constraint::Any,
            entry_location: Constraint::Any,
            entry_providers: Constraint::Any,
            entry_ownership: Constraint::Any,
            obfuscation: ObfuscationQuery::Auto,
            daita: Constraint::Any,
            daita_use_multihop_if_necessary: Constraint::Any,
            quantum_resistant: Constraint::Any,
        }
    }

    /// The mapping from [`WireguardRelayQuery`] to [`WireguardConstraints`].
    fn into_constraints(self) -> WireguardConstraints {
        WireguardConstraints {
            port: self.port,
            ip_version: self.ip_version,
            allowed_ips: self.allowed_ips,
            entry_location: self.entry_location,
            entry_providers: self.entry_providers,
            entry_ownership: self.entry_ownership,
            use_multihop: self.use_multihop.unwrap_or(false),
        }
    }
}

impl Default for WireguardRelayQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl From<WireguardRelayQuery> for WireguardConstraints {
    /// The mapping from [`WireguardRelayQuery`] to [`WireguardConstraints`].
    fn from(value: WireguardRelayQuery) -> Self {
        WireguardConstraints {
            port: value.port,
            ip_version: value.ip_version,
            allowed_ips: value.allowed_ips,
            entry_location: value.entry_location,
            entry_providers: value.entry_providers,
            entry_ownership: value.entry_ownership,
            use_multihop: value.use_multihop.unwrap_or(false),
        }
    }
}

/// This is the reflection of [`BridgeState`] + [`BridgeSettings`] in the "universe of relay
/// queries".
///
/// [`BridgeState`]: mullvad_types::relay_constraints::BridgeState
/// [`BridgeSettings`]: mullvad_types::relay_constraints::BridgeSettings
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum BridgeQuery {
    /// Bridges should not be used.
    Off,
    /// Don't care, let the relay selector choose!
    ///
    /// If this variant is intersected with another [`BridgeQuery`] `bq`,
    /// `bq` is always preferred.
    #[default]
    Auto,
    /// Bridges should be used.
    Normal(BridgeConstraints),
    /// Bridges should be used.
    Custom(Option<CustomProxy>),
}

impl BridgeQuery {
    /// `settings` will be used to decide if bridges should be used. See [`BridgeQuery`]
    /// for more details, but the algorithm beaks down to this:
    ///
    /// * `BridgeQuery::Off`: bridges will not be used
    /// * otherwise: bridges should be used
    pub const fn should_use_bridge(settings: &BridgeQuery) -> bool {
        match settings {
            BridgeQuery::Normal(_) | BridgeQuery::Custom(_) => true,
            BridgeQuery::Off | BridgeQuery::Auto => false,
        }
    }
}

impl Intersection for BridgeQuery {
    fn intersection(self, other: Self) -> Option<Self>
    where
        Self: PartialEq,
        Self: Sized,
    {
        match (self, other) {
            (BridgeQuery::Normal(left), BridgeQuery::Normal(right)) => {
                Some(BridgeQuery::Normal(left.intersection(right)?))
            }
            (BridgeQuery::Auto, right) => Some(right),
            (left, BridgeQuery::Auto) => Some(left),
            (left, right) if left == right => Some(left),
            _ => None,
        }
    }
}

#[allow(unused)]
pub mod builder {
    //! Strongly typed Builder pattern for of relay constraints though the use of the Typestate
    //! pattern.
    use mullvad_types::{
        constraints::Constraint,
        relay_constraints::{
            BridgeConstraints, LocationConstraint, RelayConstraints, SelectedObfuscation,
            ShadowsocksSettings, TransportPort, Udp2TcpObfuscationSettings,
        },
        wireguard::QuantumResistantState,
    };

    use super::{BridgeQuery, ObfuscationQuery, RelayQuery};

    // Re-exports
    pub use mullvad_types::relay_constraints::{
        GeographicLocationConstraint, Ownership, Providers,
    };
    pub use talpid_types::net::{IpVersion, TransportProtocol};

    /// Internal builder state for a [`RelayQuery`] parameterized over the
    /// type of VPN tunnel protocol. Some [`RelayQuery`] options are
    /// generic over the VPN protocol, while some options are protocol-specific.
    ///
    /// - The type parameter `VpnProtocol` keeps track of which VPN protocol that is being
    ///   configured. Different instantiations of `VpnProtocol` will expose different functions for
    ///   configuring a [`RelayQueryBuilder`] further.
    pub struct RelayQueryBuilder<Multihop, Obfuscation, Daita, QuantumResistant> {
        query: RelayQuery,
        settings: Settings<Multihop, Obfuscation, Daita, QuantumResistant>,
    }

    ///  The `Any` type is equivalent to the `Constraint::Any` value. If a
    ///  type-parameter is of type `Any`, it means that the corresponding value
    ///  in the final `RelayQuery` is `Constraint::Any`.
    pub struct Any;

    // This impl-block is quantified over all configurations, e.g. [`Any`],
    // or [`WireguardRelayQuery`]
    impl<Multihop, Obfuscation, Daita, QuantumResistant>
        RelayQueryBuilder<Multihop, Obfuscation, Daita, QuantumResistant>
    {
        /// Configure the [`LocationConstraint`] to use.
        pub fn location(mut self, location: impl Into<LocationConstraint>) -> Self {
            self.query.location = Constraint::Only(location.into());
            self
        }

        /// Configure which [`Ownership`] to use.
        pub const fn ownership(mut self, ownership: Ownership) -> Self {
            self.query.ownership = Constraint::Only(ownership);
            self
        }

        /// Configure which [`Providers`] to use.
        pub fn providers(mut self, providers: Providers) -> Self {
            self.query.providers = Constraint::Only(providers);
            self
        }

        /// Assemble the final [`RelayQuery`] that has been configured
        /// through `self`.
        pub fn build(mut self) -> RelayQuery {
            self.query
        }
    }

    impl RelayQueryBuilder<Any, Any, Any, Any> {
        /// Create a new [`RelayQueryBuilder`].
        ///
        /// Call [`Self::build`] to convert the builder into a [`RelayQuery`],
        /// which is used to guide the [`RelaySelector`]
        ///
        /// [`RelaySelector`]: crate::RelaySelector
        pub fn new() -> RelayQueryBuilder<Any, Any, Any, Any> {
            RelayQueryBuilder {
                query: RelayQuery::default(),
                settings: Settings::default(),
            }
        }
    }

    impl Default for RelayQueryBuilder<Any, Any, Any, Any> {
        fn default() -> Self {
            Self::new()
        }
    }

    // Type-safe builder for Wireguard relay constraints.

    /// Internal builder state for a [`WireguardRelayQuery`] configuration.
    ///
    /// - The type parameter `Multihop` keeps track of the state of multihop. If multihop has been
    ///   enabled, the builder should expose an option to select entry point.
    ///
    /// [`WireguardRelayQuery`]: super::WireguardRelayQuery
    struct Settings<Multihop, Obfuscation, Daita, QuantumResistant> {
        multihop: Multihop,
        obfuscation: Obfuscation,
        daita: Daita,
        quantum_resistant: QuantumResistant,
    }

    impl Default for Settings<Any, Any, Any, Any> {
        fn default() -> Self {
            Self {
                multihop: Any,
                obfuscation: Any,
                daita: Any,
                quantum_resistant: Any,
            }
        }
    }

    /// Quic obfuscation.
    ///
    /// Quic does not have any user-configurable parameters, so there is no type defined
    /// in the mullvad-types crate.
    pub struct Quic;

    /// LWO obfuscation.
    ///
    /// LWO does not have any user-configurable parameters, so there is no type defined
    /// in the mullvad-types crate.
    pub struct Lwo;

    // This impl-block is quantified over all configurations
    impl<Multihop, Obfuscation, Daita, QuantumResistant>
        RelayQueryBuilder<Multihop, Obfuscation, Daita, QuantumResistant>
    {
        /// Specify the port to ues when connecting to the selected
        /// Wireguard relay.
        pub const fn port(mut self, port: u16) -> Self {
            self.query.wireguard_constraints.port = Constraint::Only(port);
            self
        }

        /// Set the [`IpVersion`] to use when connecting to the selected
        /// Wireguard relay.
        pub const fn ip_version(mut self, ip_version: IpVersion) -> Self {
            self.query.wireguard_constraints.ip_version = Constraint::Only(ip_version);
            self
        }
    }

    impl<Multihop, Obfuscation, QuantumResistant>
        RelayQueryBuilder<Multihop, Obfuscation, Any, QuantumResistant>
    {
        /// Enable DAITA support.
        pub fn daita(mut self) -> RelayQueryBuilder<Multihop, Obfuscation, bool, QuantumResistant> {
            self.query.wireguard_constraints.daita = Constraint::Only(true);
            // Update the type state
            RelayQueryBuilder {
                query: self.query,
                settings: Settings {
                    multihop: self.settings.multihop,
                    obfuscation: self.settings.obfuscation,
                    daita: true,
                    quantum_resistant: self.settings.quantum_resistant,
                },
            }
        }
    }

    // impl-block for after DAITA is set
    impl<Multihop, Obfuscation, QuantumResistant>
        RelayQueryBuilder<Multihop, Obfuscation, bool, QuantumResistant>
    {
        /// Enable DAITA 'use_multihop_if_necessary'.
        pub fn daita_use_multihop_if_necessary(
            mut self,
            constraint: impl Into<Constraint<bool>>,
        ) -> Self {
            self.query
                .wireguard_constraints
                .daita_use_multihop_if_necessary = constraint.into();
            self
        }
    }

    impl<Multihop, Obfuscation, Daita> RelayQueryBuilder<Multihop, Obfuscation, Daita, Any> {
        /// Enable PQ support.
        pub fn quantum_resistant(
            mut self,
        ) -> RelayQueryBuilder<Multihop, Obfuscation, Daita, bool> {
            self.query.wireguard_constraints.quantum_resistant = QuantumResistantState::On.into();
            // Update the type state
            RelayQueryBuilder {
                query: self.query,
                settings: Settings {
                    multihop: self.settings.multihop,
                    obfuscation: self.settings.obfuscation,
                    daita: self.settings.daita,
                    quantum_resistant: true,
                },
            }
        }
    }

    impl<Obfuscation, Daita, QuantumResistant>
        RelayQueryBuilder<Any, Obfuscation, Daita, QuantumResistant>
    {
        /// Enable multihop.
        ///
        /// To configure the entry relay, see [`RelayQueryBuilder::entry`].
        pub fn multihop(mut self) -> RelayQueryBuilder<bool, Obfuscation, Daita, QuantumResistant> {
            self.query.wireguard_constraints.use_multihop = Constraint::Only(true);
            // Update the type state
            RelayQueryBuilder {
                query: self.query,
                settings: Settings {
                    multihop: true,
                    obfuscation: self.settings.obfuscation,
                    daita: self.settings.daita,
                    quantum_resistant: self.settings.quantum_resistant,
                },
            }
        }
    }

    impl<Obfuscation, Daita, QuantumResistant>
        RelayQueryBuilder<bool, Obfuscation, Daita, QuantumResistant>
    {
        /// Set the entry location in a multihop configuration. This requires
        /// multihop to be enabled.
        pub fn entry(mut self, location: impl Into<LocationConstraint>) -> Self {
            self.query.wireguard_constraints.entry_location = Constraint::Only(location.into());
            self
        }

        /// Set the entry location in a multihop configuration. This requires
        /// multihop to be enabled.
        pub fn entry_providers(mut self, providers: Providers) -> Self {
            self.query.wireguard_constraints.entry_providers = Constraint::Only(providers);
            self
        }

        /// Set the entry location in a multihop configuration. This requires
        /// multihop to be enabled.
        pub fn entry_ownership(mut self, ownership: Ownership) -> Self {
            self.query.wireguard_constraints.entry_ownership = Constraint::Only(ownership);
            self
        }
    }

    impl<Multihop, Daita, QuantumResistant> RelayQueryBuilder<Multihop, Any, Daita, QuantumResistant> {
        /// Enable `UDP2TCP` obufscation. This will in turn enable the option to configure the
        /// `UDP2TCP` port.
        pub fn udp2tcp(
            mut self,
        ) -> RelayQueryBuilder<Multihop, Udp2TcpObfuscationSettings, Daita, QuantumResistant>
        {
            let obfuscation = Udp2TcpObfuscationSettings {
                port: Constraint::Any,
            };
            let protocol = Settings {
                multihop: self.settings.multihop,
                obfuscation: obfuscation.clone(),
                daita: self.settings.daita,
                quantum_resistant: self.settings.quantum_resistant,
            };
            self.query.wireguard_constraints.obfuscation = ObfuscationQuery::Udp2tcp(obfuscation);
            RelayQueryBuilder {
                query: self.query,
                settings: protocol,
            }
        }

        /// Enable Shadowsocks obufscation. This will in turn enable the option to configure the
        /// port.
        pub fn shadowsocks(
            mut self,
        ) -> RelayQueryBuilder<Multihop, ShadowsocksSettings, Daita, QuantumResistant> {
            let obfuscation = ShadowsocksSettings {
                port: Constraint::Any,
            };
            let protocol = Settings {
                multihop: self.settings.multihop,
                obfuscation: obfuscation.clone(),
                daita: self.settings.daita,
                quantum_resistant: self.settings.quantum_resistant,
            };
            self.query.wireguard_constraints.obfuscation =
                ObfuscationQuery::Shadowsocks(obfuscation);
            RelayQueryBuilder {
                query: self.query,
                settings: protocol,
            }
        }

        /// Enable QUIC obfuscation.
        pub fn quic(mut self) -> RelayQueryBuilder<Multihop, Quic, Daita, QuantumResistant> {
            self.query.wireguard_constraints.obfuscation = ObfuscationQuery::Quic;
            RelayQueryBuilder {
                query: self.query,
                settings: Settings {
                    multihop: self.settings.multihop,
                    obfuscation: Quic,
                    daita: self.settings.daita,
                    quantum_resistant: self.settings.quantum_resistant,
                },
            }
        }

        /// Enable LWO obfuscation.
        pub fn lwo(mut self) -> RelayQueryBuilder<Multihop, Lwo, Daita, QuantumResistant> {
            self.query.wireguard_constraints.obfuscation = ObfuscationQuery::Lwo;
            RelayQueryBuilder {
                query: self.query,
                settings: Settings {
                    multihop: self.settings.multihop,
                    obfuscation: Lwo,
                    daita: self.settings.daita,
                    quantum_resistant: self.settings.quantum_resistant,
                },
            }
        }
    }

    impl<Multihop, Daita, QuantumResistant>
        RelayQueryBuilder<Multihop, Udp2TcpObfuscationSettings, Daita, QuantumResistant>
    {
        /// Set the `UDP2TCP` port. This is the TCP port which the `UDP2TCP` obfuscation
        /// protocol should use to connect to a relay.
        pub fn udp2tcp_port(mut self, port: u16) -> Self {
            self.settings.obfuscation.port = Constraint::Only(port);
            self.query.wireguard_constraints.obfuscation =
                ObfuscationQuery::Udp2tcp(self.settings.obfuscation.clone());
            self
        }
    }
}

/// This trait defines a bunch of helper methods on [`RelayQuery`].
pub trait RelayQueryExt {
    /// Are we using daita?
    fn using_daita(&self) -> bool;
    /// is `use_multihop_if_necessary` enabled? In other words, is `Direct only` disabled?
    fn use_multihop_if_necessary(&self) -> bool;
    /// Are we using singlehop? I.e. is multihop *not* explicitly enabled?
    fn singlehop(&self) -> bool;
}

impl RelayQueryExt for RelayQuery {
    fn using_daita(&self) -> bool {
        self.wireguard_constraints()
            .daita
            .is_only_and(|enabled| enabled)
    }
    fn use_multihop_if_necessary(&self) -> bool {
        self.wireguard_constraints()
            .daita_use_multihop_if_necessary
            // The default value is `Any`, which means that we need to check the intersection.
            .intersection(Constraint::Only(true))
            .is_some()
    }
    fn singlehop(&self) -> bool {
        !self.wireguard_constraints().multihop()
    }
}

#[cfg(test)]
mod test {
    use mullvad_types::{
        constraints::Constraint,
        relay_constraints::{
            ObfuscationSettings, SelectedObfuscation, ShadowsocksSettings,
            Udp2TcpObfuscationSettings,
        },
    };
    use proptest::prelude::*;

    use super::{Intersection, ObfuscationQuery};

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

        /// When obfuscation is set to automatic in [`ObfuscationSettings`], the query should not
        /// contain any specific obfuscation protocol settings.
        #[test]
        fn test_auto_obfuscation_settings(port1 in constraint(proptest::arbitrary::any::<u16>()), port2 in constraint(proptest::arbitrary::any::<u16>())) {
            let query = ObfuscationQuery::from(ObfuscationSettings {
                selected_obfuscation: SelectedObfuscation::Auto,
                udp2tcp: Udp2TcpObfuscationSettings {
                    port: port1,
                },
                shadowsocks: ShadowsocksSettings {
                    port: port2,
                },
            });
            assert_eq!(query, ObfuscationQuery::Auto);
        }
    }
}

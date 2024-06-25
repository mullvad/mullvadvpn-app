//! This module provides a flexible way to specify 'queries' for relays.
//!
//! A query is a set of constraints that the [`crate::RelaySelector`] will use when filtering out
//! potential relays that the daemon should connect to. It supports filtering relays by geographic
//! location, provider, ownership, and tunnel protocol, along with protocol-specific settings for
//! WireGuard and OpenVPN.
//!
//! The main components of this module include:
//!
//! - [`RelayQuery`]: The core struct for specifying a query to select relay servers. It aggregates
//!   constraints on location, providers, ownership, tunnel protocol, and protocol-specific
//!   constraints for WireGuard and OpenVPN.
//! - [`WireguardRelayQuery`] and [`OpenVpnRelayQuery`]: Structs that define protocol-specific
//!   constraints for selecting WireGuard and OpenVPN relays, respectively.
//! - [`Intersection`]: A trait implemented by the different query types that support intersection
//!   logic, which allows for combining two queries into a single query that represents the common
//!   constraints of both.
//! - [Builder patterns][builder]: The module also provides builder patterns for creating instances
//!   of `RelayQuery`, `WireguardRelayQuery`, and `OpenVpnRelayQuery` with a fluent API.
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

use crate::AdditionalWireguardConstraints;
use mullvad_types::{
    constraints::Constraint,
    relay_constraints::{
        BridgeConstraints, LocationConstraint, OpenVpnConstraints, Ownership, Providers,
        RelayConstraints, SelectedObfuscation, ShadowsocksSettings, TransportPort,
        Udp2TcpObfuscationSettings, WireguardConstraints,
    },
    Intersection,
};
use talpid_types::net::{proxy::CustomProxy, IpVersion, TunnelType};

/// Represents a query for a relay based on various constraints.
///
/// This struct contains constraints for the location, providers, ownership,
/// tunnel protocol, and additional protocol-specific constraints for WireGuard
/// and OpenVPN. These constraints are used by the [`crate::RelaySelector`] to
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
/// let query: RelayQuery = RelayQueryBuilder::new()
///     .wireguard()                                            // Specify the tunnel protocol
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
    pub location: Constraint<LocationConstraint>,
    pub providers: Constraint<Providers>,
    pub ownership: Constraint<Ownership>,
    pub tunnel_protocol: Constraint<TunnelType>,
    pub wireguard_constraints: WireguardRelayQuery,
    pub openvpn_constraints: OpenVpnRelayQuery,
}

impl RelayQuery {
    /// Create a new [`RelayQuery`] with no opinionated defaults. This query matches every relay
    /// with any configuration by setting each of its fields to [`Constraint::Any`]. Should be the
    /// const equivalent to [`Default::default`].
    ///
    /// Note that the following identity applies for any `other_query`:
    /// ```rust
    /// # use mullvad_relay_selector::query::RelayQuery;
    /// # use mullvad_types::Intersection;
    ///
    /// # let other_query = RelayQuery::new();
    /// assert_eq!(RelayQuery::new().intersection(other_query.clone()), Some(other_query));
    /// # let other_query = RelayQuery::new();
    /// assert_eq!(other_query.clone().intersection(RelayQuery::new()), Some(other_query));
    /// ```
    pub const fn new() -> RelayQuery {
        RelayQuery {
            location: Constraint::Any,
            providers: Constraint::Any,
            ownership: Constraint::Any,
            tunnel_protocol: Constraint::Any,
            wireguard_constraints: WireguardRelayQuery::new(),
            openvpn_constraints: OpenVpnRelayQuery::new(),
        }
    }
}

impl Default for RelayQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl From<RelayQuery> for RelayConstraints {
    /// The mapping from [`RelayQuery`] to [`RelayConstraints`].
    fn from(value: RelayQuery) -> Self {
        RelayConstraints {
            location: value.location,
            providers: value.providers,
            ownership: value.ownership,
            tunnel_protocol: value.tunnel_protocol,
            wireguard_constraints: WireguardConstraints::from(value.wireguard_constraints),
            openvpn_constraints: OpenVpnConstraints::from(value.openvpn_constraints),
        }
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
    pub use_multihop: Constraint<bool>,
    pub entry_location: Constraint<LocationConstraint>,
    pub obfuscation: SelectedObfuscation,
    pub udp2tcp_port: Constraint<Udp2TcpObfuscationSettings>,
    pub shadowsocks_port: Constraint<ShadowsocksSettings>,
    pub daita: Constraint<bool>,
}

impl WireguardRelayQuery {
    pub fn multihop(&self) -> bool {
        matches!(self.use_multihop, Constraint::Only(true))
    }
}

impl WireguardRelayQuery {
    pub const fn new() -> WireguardRelayQuery {
        WireguardRelayQuery {
            port: Constraint::Any,
            ip_version: Constraint::Any,
            use_multihop: Constraint::Any,
            entry_location: Constraint::Any,
            obfuscation: SelectedObfuscation::Auto,
            udp2tcp_port: Constraint::Any,
            shadowsocks_port: Constraint::Any,
            daita: Constraint::Any,
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
            entry_location: value.entry_location,
            use_multihop: value.use_multihop.unwrap_or(false),
        }
    }
}

impl From<WireguardRelayQuery> for AdditionalWireguardConstraints {
    /// The mapping from [`WireguardRelayQuery`] to [`AdditionalWireguardConstraints`].
    fn from(value: WireguardRelayQuery) -> Self {
        AdditionalWireguardConstraints {
            daita: value
                .daita
                .unwrap_or(AdditionalWireguardConstraints::default().daita),
        }
    }
}

/// A query for a relay with OpenVPN-specific properties, such as `bridge_settings`.
///
/// This struct may look a lot like [`OpenVpnConstraints`], and that is the point!
/// This struct is meant to be that type in the "universe of relay queries". The difference
/// between them may seem subtle, but in a [`OpenVpnRelayQuery`] every field is represented
/// as a [`Constraint`], which allow us to implement [`Intersection`] in a straight forward manner.
#[derive(Debug, Clone, Eq, PartialEq, Intersection)]
pub struct OpenVpnRelayQuery {
    pub port: Constraint<TransportPort>,
    pub bridge_settings: Constraint<BridgeQuery>,
}

impl OpenVpnRelayQuery {
    pub const fn new() -> OpenVpnRelayQuery {
        OpenVpnRelayQuery {
            port: Constraint::Any,
            bridge_settings: Constraint::Any,
        }
    }
}

impl Default for OpenVpnRelayQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// This is the reflection of [`BridgeState`] + [`BridgeSettings`] in the "universe of relay
/// queries".
///
/// [`BridgeState`]: mullvad_types::relay_constraints::BridgeState
/// [`BridgeSettings`]: mullvad_types::relay_constraints::BridgeSettings
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BridgeQuery {
    /// Bridges should not be used.
    Off,
    /// Don't care, let the relay selector choose!
    ///
    /// If this variant is intersected with another [`BridgeQuery`] `bq`,
    /// `bq` is always preferred.
    Auto,
    /// Bridges should be used.
    Normal(BridgeConstraints),
    /// Bridges should be used.
    Custom(Option<CustomProxy>),
}

impl BridgeQuery {
    /// If `bridge_constraints` is `Any`, bridges should not be used due to
    /// latency concerns.
    ///
    /// If `bridge_constraints` is `Only(settings)`, then `settings` will be
    /// used to decide if bridges should be used. See [`BridgeQuery`] for more
    /// details, but the algorithm beaks down to this:
    ///
    /// * `BridgeQuery::Off`: bridges will not be used
    /// * otherwise: bridges should be used
    pub const fn should_use_bridge(bridge_constraints: &Constraint<BridgeQuery>) -> bool {
        match bridge_constraints {
            Constraint::Only(settings) => match settings {
                BridgeQuery::Normal(_) | BridgeQuery::Custom(_) => true,
                BridgeQuery::Off | BridgeQuery::Auto => false,
            },
            Constraint::Any => false,
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

impl From<OpenVpnRelayQuery> for OpenVpnConstraints {
    /// The mapping from [`OpenVpnRelayQuery`] to [`OpenVpnConstraints`].
    fn from(value: OpenVpnRelayQuery) -> Self {
        OpenVpnConstraints { port: value.port }
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
    };
    use talpid_types::net::TunnelType;

    use super::{BridgeQuery, RelayQuery};

    // Re-exports
    pub use mullvad_types::relay_constraints::{
        GeographicLocationConstraint, Ownership, Providers,
    };
    pub use talpid_types::net::{IpVersion, TransportProtocol};

    /// Internal builder state for a [`RelayQuery`] parameterized over the
    /// type of VPN tunnel protocol. Some [`RelayQuery`] options are
    /// generic over the VPN protocol, while some options are protocol-specific.
    ///
    /// - The type parameter `VpnProtocol` keeps track of which VPN protocol that
    /// is being configured. Different instantiations of `VpnProtocol` will
    /// expose different functions for configuring a [`RelayQueryBuilder`]
    /// further.
    pub struct RelayQueryBuilder<VpnProtocol = Any> {
        query: RelayQuery,
        protocol: VpnProtocol,
    }

    ///  The `Any` type is equivalent to the `Constraint::Any` value. If a
    ///  type-parameter is of type `Any`, it means that the corresponding value
    ///  in the final `RelayQuery` is `Constraint::Any`.
    pub struct Any;

    // This impl-block is quantified over all configurations, e.g. [`Any`],
    // [`WireguardRelayQuery`] & [`OpenVpnRelayQuery`]
    impl<VpnProtocol> RelayQueryBuilder<VpnProtocol> {
        /// Configure the [`LocationConstraint`] to use.
        pub fn location(mut self, location: GeographicLocationConstraint) -> Self {
            self.query.location = Constraint::Only(LocationConstraint::from(location));
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
        pub fn build(self) -> RelayQuery {
            self.query
        }

        pub fn into_constraint(self) -> RelayConstraints {
            RelayConstraints::from(self.build())
        }
    }

    impl RelayQueryBuilder<Any> {
        /// Create a new [`RelayQueryBuilder`] with unopinionated defaults.
        ///
        /// Call [`Self::build`] to convert the builder into a [`RelayQuery`],
        /// which is used to guide the [`RelaySelector`]
        ///
        /// [`RelaySelector`]: crate::RelaySelector
        pub const fn new() -> RelayQueryBuilder<Any> {
            RelayQueryBuilder {
                query: RelayQuery::new(),
                protocol: Any,
            }
        }
        /// Set the VPN protocol for this [`RelayQueryBuilder`] to Wireguard.
        pub fn wireguard(mut self) -> RelayQueryBuilder<Wireguard<Any, Any, Any>> {
            let protocol = Wireguard {
                multihop: Any,
                obfuscation: Any,
                daita: Any,
            };
            self.query.tunnel_protocol = Constraint::Only(TunnelType::Wireguard);
            // Update the type state
            RelayQueryBuilder {
                query: self.query,
                protocol,
            }
        }

        /// Set the VPN protocol for this [`RelayQueryBuilder`] to OpenVPN.
        pub fn openvpn(mut self) -> RelayQueryBuilder<OpenVPN<Any, Any>> {
            let protocol = OpenVPN {
                transport_port: Any,
                bridge_settings: Any,
            };
            self.query.tunnel_protocol = Constraint::Only(TunnelType::OpenVpn);
            // Update the type state
            RelayQueryBuilder {
                query: self.query,
                protocol,
            }
        }
    }

    impl Default for RelayQueryBuilder<Any> {
        fn default() -> Self {
            Self::new()
        }
    }

    // Type-safe builder for Wireguard relay constraints.

    /// Internal builder state for a [`WireguardRelayQuery`] configuration.
    ///
    /// - The type parameter `Multihop` keeps track of the state of multihop.
    /// If multihop has been enabled, the builder should expose an option to
    /// select entry point.
    ///
    /// [`WireguardRelayQuery`]: super::WireguardRelayQuery
    pub struct Wireguard<Multihop, Obfuscation, Daita> {
        multihop: Multihop,
        obfuscation: Obfuscation,
        daita: Daita,
    }

    // This impl-block is quantified over all configurations
    impl<Multihop, Obfuscation, Daita> RelayQueryBuilder<Wireguard<Multihop, Obfuscation, Daita>> {
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

    impl<Multihop, Obfuscation> RelayQueryBuilder<Wireguard<Multihop, Obfuscation, Any>> {
        /// Enable DAITA support.
        pub fn daita(mut self) -> RelayQueryBuilder<Wireguard<Multihop, Obfuscation, bool>> {
            self.query.wireguard_constraints.daita = Constraint::Only(true);
            // Update the type state
            RelayQueryBuilder {
                query: self.query,
                protocol: Wireguard {
                    multihop: self.protocol.multihop,
                    obfuscation: self.protocol.obfuscation,
                    daita: true,
                },
            }
        }
    }

    impl<Obfuscation, Daita> RelayQueryBuilder<Wireguard<Any, Obfuscation, Daita>> {
        /// Enable multihop.
        ///
        /// To configure the entry relay, see [`RelayQueryBuilder::entry`].
        pub fn multihop(mut self) -> RelayQueryBuilder<Wireguard<bool, Obfuscation, Daita>> {
            self.query.wireguard_constraints.use_multihop = Constraint::Only(true);
            // Update the type state
            RelayQueryBuilder {
                query: self.query,
                protocol: Wireguard {
                    multihop: true,
                    obfuscation: self.protocol.obfuscation,
                    daita: self.protocol.daita,
                },
            }
        }
    }

    impl<Obfuscation, Daita> RelayQueryBuilder<Wireguard<bool, Obfuscation, Daita>> {
        /// Set the entry location in a multihop configuration. This requires
        /// multihop to be enabled.
        pub fn entry(mut self, location: GeographicLocationConstraint) -> Self {
            self.query.wireguard_constraints.entry_location =
                Constraint::Only(LocationConstraint::from(location));
            self
        }
    }

    impl<Multihop, Daita> RelayQueryBuilder<Wireguard<Multihop, Any, Daita>> {
        /// Enable `UDP2TCP` obufscation. This will in turn enable the option to configure the
        /// `UDP2TCP` port.
        pub fn udp2tcp(
            mut self,
        ) -> RelayQueryBuilder<Wireguard<Multihop, Udp2TcpObfuscationSettings, Daita>> {
            let obfuscation = Udp2TcpObfuscationSettings {
                port: Constraint::Any,
            };
            let protocol = Wireguard {
                multihop: self.protocol.multihop,
                obfuscation: obfuscation.clone(),
                daita: self.protocol.daita,
            };
            self.query.wireguard_constraints.udp2tcp_port = Constraint::Only(obfuscation);
            self.query.wireguard_constraints.obfuscation = SelectedObfuscation::Udp2Tcp;
            RelayQueryBuilder {
                query: self.query,
                protocol,
            }
        }

        /// Enable Shadowsocks obufscation. This will in turn enable the option to configure the
        /// port.
        pub fn shadowsocks(
            mut self,
        ) -> RelayQueryBuilder<Wireguard<Multihop, ShadowsocksSettings, Daita>> {
            let obfuscation = ShadowsocksSettings {
                port: Constraint::Any,
            };
            let protocol = Wireguard {
                multihop: self.protocol.multihop,
                obfuscation: obfuscation.clone(),
                daita: self.protocol.daita,
            };
            self.query.wireguard_constraints.shadowsocks_port = Constraint::Only(obfuscation);
            self.query.wireguard_constraints.obfuscation = SelectedObfuscation::Shadowsocks;
            RelayQueryBuilder {
                query: self.query,
                protocol,
            }
        }
    }

    impl<Multihop, Daita> RelayQueryBuilder<Wireguard<Multihop, Udp2TcpObfuscationSettings, Daita>> {
        /// Set the `UDP2TCP` port. This is the TCP port which the `UDP2TCP` obfuscation
        /// protocol should use to connect to a relay.
        pub fn udp2tcp_port(mut self, port: u16) -> Self {
            self.protocol.obfuscation.port = Constraint::Only(port);
            self.query.wireguard_constraints.udp2tcp_port =
                Constraint::Only(self.protocol.obfuscation.clone());
            self
        }
    }

    // Type-safe builder pattern for OpenVPN relay constraints.

    /// Internal builder state for a [`OpenVpnRelayQuery`] configuration.
    ///
    /// - The type parameter `TransportPort` keeps track of which
    /// [`TransportProtocol`] & port-combo to use. [`TransportProtocol`] has
    /// to be set first before the option to select a specific port is
    /// exposed.
    ///
    /// [`OpenVpnRelayQuery`]: super::OpenVpnRelayQuery
    pub struct OpenVPN<TransportPort, Bridge> {
        transport_port: TransportPort,
        bridge_settings: Bridge,
    }

    // This impl-block is quantified over all configurations
    impl<Transport, Bridge> RelayQueryBuilder<OpenVPN<Transport, Bridge>> {
        /// Configure what [`TransportProtocol`] to use. Calling this
        /// function on a builder will expose the option to select which
        /// port to use in combination with `protocol`.
        pub fn transport_protocol(
            mut self,
            protocol: TransportProtocol,
        ) -> RelayQueryBuilder<OpenVPN<TransportProtocol, Bridge>> {
            let transport_port = TransportPort {
                protocol,
                port: Constraint::Any,
            };
            self.query.openvpn_constraints.port = Constraint::Only(transport_port);
            // Update the type state
            RelayQueryBuilder {
                query: self.query,
                protocol: OpenVPN {
                    transport_port: protocol,
                    bridge_settings: self.protocol.bridge_settings,
                },
            }
        }
    }

    impl<Bridge> RelayQueryBuilder<OpenVPN<TransportProtocol, Bridge>> {
        /// Configure what port to use when connecting to a relay.
        pub fn port(mut self, port: u16) -> RelayQueryBuilder<OpenVPN<TransportPort, Bridge>> {
            let port = Constraint::Only(port);
            let transport_port = TransportPort {
                protocol: self.protocol.transport_port,
                port,
            };
            self.query.openvpn_constraints.port = Constraint::Only(transport_port);
            // Update the type state
            RelayQueryBuilder {
                query: self.query,
                protocol: OpenVPN {
                    transport_port,
                    bridge_settings: self.protocol.bridge_settings,
                },
            }
        }
    }

    impl<Transport> RelayQueryBuilder<OpenVPN<Transport, Any>> {
        /// Enable Bridges. This also sets the transport protocol to TCP and resets any
        /// previous port settings.
        pub fn bridge(
            mut self,
        ) -> RelayQueryBuilder<OpenVPN<TransportProtocol, BridgeConstraints>> {
            let bridge_settings = BridgeConstraints {
                location: Constraint::Any,
                providers: Constraint::Any,
                ownership: Constraint::Any,
            };

            let protocol = OpenVPN {
                transport_port: self.protocol.transport_port,
                bridge_settings: bridge_settings.clone(),
            };

            self.query.openvpn_constraints.bridge_settings =
                Constraint::Only(BridgeQuery::Normal(bridge_settings));

            let builder = RelayQueryBuilder {
                query: self.query,
                protocol,
            };

            builder.transport_protocol(TransportProtocol::Tcp)
        }
    }

    impl<Transport> RelayQueryBuilder<OpenVPN<Transport, BridgeConstraints>> {
        /// Constraint the geographical location of the selected bridge.
        pub fn bridge_location(mut self, location: GeographicLocationConstraint) -> Self {
            self.protocol.bridge_settings.location =
                Constraint::Only(LocationConstraint::from(location));
            self.query.openvpn_constraints.bridge_settings =
                Constraint::Only(BridgeQuery::Normal(self.protocol.bridge_settings.clone()));
            self
        }
        /// Constrain the [`Providers`] of the selected bridge.
        pub fn bridge_providers(mut self, providers: Providers) -> Self {
            self.protocol.bridge_settings.providers = Constraint::Only(providers);
            self.query.openvpn_constraints.bridge_settings =
                Constraint::Only(BridgeQuery::Normal(self.protocol.bridge_settings.clone()));
            self
        }
        /// Constrain the [`Ownership`] of the selected bridge.
        pub fn bridge_ownership(mut self, ownership: Ownership) -> Self {
            self.protocol.bridge_settings.ownership = Constraint::Only(ownership);
            self
        }
    }
}

#[cfg(test)]
mod test {
    use mullvad_types::constraints::Constraint;
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
    }
}

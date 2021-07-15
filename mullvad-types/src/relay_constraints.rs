//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.

use crate::{
    location::{CityCode, CountryCode, Hostname},
    relay_list::{OpenVpnEndpointData, Relay, WireguardEndpointData},
    CustomTunnelEndpoint,
};
#[cfg(target_os = "android")]
use jnix::{FromJava, IntoJava};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};
use talpid_types::net::{openvpn::ProxySettings, IpVersion, TransportProtocol, TunnelType};

pub const WIREGUARD_TCP_PORTS: [(u16, u16); 3] = [(80, 80), (443, 443), (5001, 5001)];


pub trait Match<T> {
    fn matches(&self, other: &T) -> bool;
}

pub trait Set<T> {
    fn is_subset(&self, other: &T) -> bool;
}

/// Limits the set of [`crate::relay_list::Relay`]s that a `RelaySelector` may select.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[cfg_attr(target_os = "android", jnix(bounds = "T: android.os.Parcelable"))]
pub enum Constraint<T: fmt::Debug + Clone + Eq + PartialEq> {
    Any,
    Only(T),
}

impl<T: fmt::Debug + Clone + Eq + PartialEq> Constraint<T> {
    pub fn unwrap(self) -> T {
        match self {
            Constraint::Any => panic!("called `Constraint::unwrap()` on an `Any` value"),
            Constraint::Only(value) => value,
        }
    }

    pub fn unwrap_or(self, other: T) -> T {
        match self {
            Constraint::Any => other,
            Constraint::Only(value) => value,
        }
    }

    pub fn or(self, other: Constraint<T>) -> Constraint<T> {
        match self {
            Constraint::Any => other,
            Constraint::Only(value) => Constraint::Only(value),
        }
    }

    pub fn map<U: fmt::Debug + Clone + Eq + PartialEq, F: FnOnce(T) -> U>(
        self,
        f: F,
    ) -> Constraint<U> {
        match self {
            Constraint::Any => Constraint::Any,
            Constraint::Only(value) => Constraint::Only(f(value)),
        }
    }

    pub fn is_any(&self) -> bool {
        match self {
            Constraint::Any => true,
            Constraint::Only(_value) => false,
        }
    }

    pub fn as_ref(&self) -> Constraint<&T> {
        match self {
            Constraint::Any => Constraint::Any,
            Constraint::Only(ref value) => Constraint::Only(value),
        }
    }

    pub fn option(self) -> Option<T> {
        match self {
            Constraint::Any => None,
            Constraint::Only(value) => Some(value),
        }
    }

    pub fn matches_eq(&self, other: &T) -> bool {
        match self {
            Constraint::Any => true,
            Constraint::Only(ref value) => value == other,
        }
    }
}

impl<T: fmt::Debug + Clone + Eq + PartialEq> Default for Constraint<T> {
    fn default() -> Self {
        Constraint::Any
    }
}

impl<T: Copy + fmt::Debug + Clone + Eq + PartialEq> Copy for Constraint<T> {}

impl<T: fmt::Debug + Clone + Eq + Match<U>, U> Match<U> for Constraint<T> {
    fn matches(&self, other: &U) -> bool {
        match *self {
            Constraint::Any => true,
            Constraint::Only(ref value) => value.matches(other),
        }
    }
}

impl<T: fmt::Debug + Clone + Eq + Set<U>, U: fmt::Debug + Clone + Eq> Set<Constraint<U>>
    for Constraint<T>
{
    fn is_subset(&self, other: &Constraint<U>) -> bool {
        match self {
            Constraint::Any => *other == Constraint::Any,
            Constraint::Only(ref constraint) => match other {
                Constraint::Only(ref other_constraint) => constraint.is_subset(other_constraint),
                _ => true,
            },
        }
    }
}

impl<T: fmt::Debug + Clone + Eq + PartialEq> From<Option<T>> for Constraint<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Constraint::Only(value),
            None => Constraint::Any,
        }
    }
}

/// Specifies a specific endpoint or [`RelayConstraints`] to use when `mullvad-daemon` selects a
/// relay.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub enum RelaySettings {
    CustomTunnelEndpoint(CustomTunnelEndpoint),
    Normal(RelayConstraints),
}

impl fmt::Display for RelaySettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            RelaySettings::CustomTunnelEndpoint(endpoint) => {
                write!(f, "custom endpoint {}", endpoint)
            }
            RelaySettings::Normal(constraints) => constraints.fmt(f),
        }
    }
}

impl RelaySettings {
    pub fn merge(&mut self, update: RelaySettingsUpdate) -> Self {
        match update {
            RelaySettingsUpdate::CustomTunnelEndpoint(relay) => {
                RelaySettings::CustomTunnelEndpoint(relay)
            }
            RelaySettingsUpdate::Normal(constraint_update) => RelaySettings::Normal(match *self {
                RelaySettings::CustomTunnelEndpoint(_) => {
                    RelayConstraints::default().merge(constraint_update)
                }
                RelaySettings::Normal(ref constraint) => constraint.merge(constraint_update),
            }),
        }
    }

    pub(crate) fn ensure_bridge_compatibility(&mut self) {
        match self {
            RelaySettings::Normal(ref mut constraints) => {
                if constraints.tunnel_protocol == Constraint::Only(TunnelType::Wireguard) {
                    constraints.tunnel_protocol = Constraint::Any;
                }
                if constraints.openvpn_constraints.protocol
                    == Constraint::Only(TransportProtocol::Udp)
                {
                    constraints.openvpn_constraints = OpenVpnConstraints {
                        protocol: Constraint::Any,
                        port: Constraint::Any,
                    }
                }
            }
            RelaySettings::CustomTunnelEndpoint(config) => {
                if config.endpoint().protocol == TransportProtocol::Udp {
                    log::warn!(
                        "Using custom tunnel endpoint with UDP, bridges will likely not work"
                    );
                }
            }
        }
    }
}

/// Limits the set of [`crate::relay_list::Relay`]s that a `RelaySelector` may select.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
#[cfg_attr(not(target_os = "android"), derive(Default))]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct RelayConstraints {
    pub location: Constraint<LocationConstraint>,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub providers: Constraint<Providers>,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub tunnel_protocol: Constraint<TunnelType>,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub wireguard_constraints: WireguardConstraints,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub openvpn_constraints: OpenVpnConstraints,
}

#[cfg(target_os = "android")]
impl Default for RelayConstraints {
    fn default() -> Self {
        RelayConstraints {
            tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
            location: Constraint::default(),
            providers: Constraint::default(),
            wireguard_constraints: WireguardConstraints::default(),
            openvpn_constraints: OpenVpnConstraints::default(),
        }
    }
}

impl RelayConstraints {
    pub fn merge(&self, update: RelayConstraintsUpdate) -> Self {
        RelayConstraints {
            location: update.location.unwrap_or_else(|| self.location.clone()),
            providers: update.providers.unwrap_or_else(|| self.providers.clone()),
            tunnel_protocol: update
                .tunnel_protocol
                .unwrap_or_else(|| self.tunnel_protocol.clone()),
            wireguard_constraints: update
                .wireguard_constraints
                .unwrap_or_else(|| self.wireguard_constraints.clone()),
            openvpn_constraints: update
                .openvpn_constraints
                .unwrap_or_else(|| self.openvpn_constraints.clone()),
        }
    }
}

impl fmt::Display for RelayConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.tunnel_protocol {
            Constraint::Any => write!(
                f,
                "Any tunnel protocol with OpenVPN through {} and WireGuard through {}",
                &self.openvpn_constraints, &self.wireguard_constraints,
            )?,
            Constraint::Only(ref tunnel_protocol) => {
                tunnel_protocol.fmt(f)?;
                match tunnel_protocol {
                    TunnelType::Wireguard => {
                        write!(f, " over {}", &self.wireguard_constraints)?;
                    }
                    TunnelType::OpenVpn => {
                        write!(f, " over {}", &self.openvpn_constraints)?;
                    }
                };
            }
        }
        write!(f, " in ")?;
        match self.location {
            Constraint::Any => write!(f, "any location")?,
            Constraint::Only(ref location_constraint) => location_constraint.fmt(f)?,
        }
        write!(f, " using ")?;
        match self.providers {
            Constraint::Any => write!(f, "any provider"),
            Constraint::Only(ref constraint) => constraint.fmt(f),
        }
    }
}


/// Limits the set of [`crate::relay_list::Relay`]s used by a `RelaySelector` based on
/// location.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub enum LocationConstraint {
    /// A country is represented by its two letter country code.
    Country(CountryCode),
    /// A city is composed of a country code and a city code.
    City(CountryCode, CityCode),
    /// An single hostname in a given city.
    Hostname(CountryCode, CityCode, Hostname),
}

impl Match<Relay> for LocationConstraint {
    fn matches(&self, relay: &Relay) -> bool {
        match self {
            LocationConstraint::Country(ref country) => {
                relay
                    .location
                    .as_ref()
                    .map_or(false, |loc| loc.country_code == *country)
                    && relay.include_in_country
            }
            LocationConstraint::City(ref country, ref city) => {
                relay.location.as_ref().map_or(false, |loc| {
                    loc.country_code == *country && loc.city_code == *city
                })
            }
            LocationConstraint::Hostname(ref country, ref city, ref hostname) => {
                relay.location.as_ref().map_or(false, |loc| {
                    loc.country_code == *country
                        && loc.city_code == *city
                        && relay.hostname == *hostname
                })
            }
        }
    }
}

impl Set<LocationConstraint> for LocationConstraint {
    /// Returns whether `self` is equal to or a subset of `other`.
    fn is_subset(&self, other: &Self) -> bool {
        match self {
            LocationConstraint::Country(_) => self == other,
            LocationConstraint::City(ref country, ref _city) => match other {
                LocationConstraint::Country(ref other_country) => country == other_country,
                LocationConstraint::City(..) => self == other,
                _ => false,
            },
            LocationConstraint::Hostname(ref country, ref city, ref _hostname) => match other {
                LocationConstraint::Country(ref other_country) => country == other_country,
                LocationConstraint::City(ref other_country, ref other_city) => {
                    country == other_country && city == other_city
                }
                LocationConstraint::Hostname(..) => self == other,
            },
        }
    }
}

/// Limits the set of [`crate::relay_list::Relay`]s used by a `RelaySelector` based on
/// provider.
pub type Provider = String;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Providers {
    providers: HashSet<Provider>,
}

/// Returned if the iterator contained no providers.
pub struct NoProviders(());

impl Providers {
    pub fn new(providers: impl Iterator<Item = Provider>) -> Result<Providers, NoProviders> {
        let providers = Providers {
            providers: providers.collect(),
        };
        if providers.providers.is_empty() {
            return Err(NoProviders(()));
        }
        Ok(providers)
    }
}

impl Match<Relay> for Providers {
    fn matches(&self, relay: &Relay) -> bool {
        self.providers.contains(&relay.provider)
    }
}

impl From<Providers> for Vec<Provider> {
    fn from(providers: Providers) -> Vec<Provider> {
        providers.providers.into_iter().collect()
    }
}

impl fmt::Display for Providers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "provider(s) ")?;
        for (i, provider) in self.providers.iter().enumerate() {
            if i == 0 {
                write!(f, "{}", provider)?;
            } else {
                write!(f, ", {}", provider)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for LocationConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            LocationConstraint::Country(country) => write!(f, "country {}", country),
            LocationConstraint::City(country, city) => write!(f, "city {}, {}", city, country),
            LocationConstraint::Hostname(country, city, hostname) => {
                write!(f, "city {}, {}, hostname {}", city, country, hostname)
            }
        }
    }
}

/// Deprecated. Contains protocol-specific constraints for relay selection.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum TunnelConstraints {
    #[serde(rename = "openvpn")]
    OpenVpn(OpenVpnConstraints),
    #[serde(rename = "wireguard")]
    Wireguard(WireguardConstraints),
}

impl fmt::Display for TunnelConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            TunnelConstraints::OpenVpn(openvpn_constraints) => {
                write!(f, "OpenVPN over ")?;
                openvpn_constraints.fmt(f)
            }
            TunnelConstraints::Wireguard(wireguard_constraints) => {
                write!(f, "Wireguard over ")?;
                wireguard_constraints.fmt(f)
            }
        }
    }
}

impl Match<OpenVpnEndpointData> for TunnelConstraints {
    fn matches(&self, endpoint: &OpenVpnEndpointData) -> bool {
        match *self {
            TunnelConstraints::OpenVpn(ref constraints) => constraints.matches(endpoint),
            _ => false,
        }
    }
}

impl Match<WireguardEndpointData> for TunnelConstraints {
    fn matches(&self, endpoint: &WireguardEndpointData) -> bool {
        match *self {
            TunnelConstraints::Wireguard(ref constraints) => constraints.matches(endpoint),
            _ => false,
        }
    }
}

/// [`Constraint`]s applicable to OpenVPN relay servers.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct OpenVpnConstraints {
    pub port: Constraint<u16>,
    pub protocol: Constraint<TransportProtocol>,
}

impl fmt::Display for OpenVpnConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.port {
            Constraint::Any => write!(f, "any port")?,
            Constraint::Only(port) => write!(f, "port {}", port)?,
        }
        write!(f, " over ")?;
        match self.protocol {
            Constraint::Any => write!(f, "any protocol"),
            Constraint::Only(protocol) => write!(f, "{}", protocol),
        }
    }
}

impl Match<OpenVpnEndpointData> for OpenVpnConstraints {
    fn matches(&self, endpoint: &OpenVpnEndpointData) -> bool {
        self.port.matches_eq(&endpoint.port) && self.protocol.matches_eq(&endpoint.protocol)
    }
}

/// [`Constraint`]s applicable to WireGuard relay servers.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct WireguardConstraints {
    pub port: Constraint<u16>,
    pub protocol: Constraint<TransportProtocol>,
    pub ip_version: Constraint<IpVersion>,
    pub entry_location: Option<Constraint<LocationConstraint>>,
}

impl fmt::Display for WireguardConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.port {
            Constraint::Any => write!(f, "any port")?,
            Constraint::Only(port) => write!(f, "port {}", port)?,
        }
        write!(f, " over ")?;
        match self.ip_version {
            Constraint::Any => write!(f, "IPv4 or IPv6")?,
            Constraint::Only(protocol) => write!(f, "{}", protocol)?,
        }
        if let Some(Constraint::Only(ref entry)) = self.entry_location {
            write!(f, " (via {})", entry)
        } else {
            Ok(())
        }
    }
}

impl Match<WireguardEndpointData> for WireguardConstraints {
    fn matches(&self, endpoint: &WireguardEndpointData) -> bool {
        match self.port {
            Constraint::Any => true,
            Constraint::Only(port) => match self.protocol {
                Constraint::Only(TransportProtocol::Tcp) => WIREGUARD_TCP_PORTS
                    .iter()
                    .any(|range| (port >= range.0 && port <= range.1)),
                _ => endpoint
                    .port_ranges
                    .iter()
                    .any(|range| (port >= range.0 && port <= range.1)),
            },
        }
    }
}


/// Specifies a specific endpoint or [`BridgeConstraints`] to use when `mullvad-daemon` selects a
/// bridge server.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeSettings {
    /// Let the relay selection algorithm decide on bridges, based on the relay list.
    Normal(BridgeConstraints),
    Custom(ProxySettings),
}


/// Limits the set of bridge servers to use in `mullvad-daemon`.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
#[serde(rename_all = "snake_case")]
pub struct BridgeConstraints {
    pub location: Constraint<LocationConstraint>,
    pub providers: Constraint<Providers>,
}

impl fmt::Display for BridgeConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.location {
            Constraint::Any => write!(f, "any location"),
            Constraint::Only(ref location_constraint) => location_constraint.fmt(f),
        }
    }
}

/// Setting indicating whether to connect to a bridge server, or to handle it automatically.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeState {
    Auto,
    On,
    Off,
}

impl fmt::Display for BridgeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BridgeState::Auto => "auto",
                BridgeState::On => "on",
                BridgeState::Off => "off",
            }
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct InternalBridgeConstraints {
    pub location: Constraint<LocationConstraint>,
    pub providers: Constraint<Providers>,
    pub transport_protocol: Constraint<TransportProtocol>,
}

/// Used to update the [`RelaySettings`] used in `mullvad-daemon`.
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(target_os = "android", derive(FromJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[serde(rename_all = "snake_case")]
pub enum RelaySettingsUpdate {
    #[cfg_attr(target_os = "android", jnix(deny))]
    CustomTunnelEndpoint(CustomTunnelEndpoint),
    Normal(RelayConstraintsUpdate),
}

impl RelaySettingsUpdate {
    /// Returns false if the specified relay settings update explicitly do not allow for bridging
    /// (i.e. use UDP instead of TCP)
    pub fn supports_bridge(&self) -> bool {
        match &self {
            RelaySettingsUpdate::CustomTunnelEndpoint(endpoint) => {
                endpoint.endpoint().protocol == TransportProtocol::Tcp
            }
            RelaySettingsUpdate::Normal(update) => {
                if let Some(Constraint::Only(TunnelType::Wireguard)) = &update.tunnel_protocol {
                    false
                } else if let Some(constraints) = &update.openvpn_constraints {
                    if let Constraint::Only(TransportProtocol::Udp) = &constraints.protocol {
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
        }
    }
}

/// Used in [`RelaySettings`] to change relay constraints in the daemon.
#[derive(Debug, Default, Deserialize, Serialize)]
#[cfg_attr(target_os = "android", derive(FromJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[serde(default)]
pub struct RelayConstraintsUpdate {
    pub location: Option<Constraint<LocationConstraint>>,
    #[cfg_attr(target_os = "android", jnix(default))]
    pub providers: Option<Constraint<Providers>>,
    #[cfg_attr(target_os = "android", jnix(default))]
    pub tunnel_protocol: Option<Constraint<TunnelType>>,
    #[cfg_attr(target_os = "android", jnix(default))]
    pub wireguard_constraints: Option<WireguardConstraints>,
    #[cfg_attr(target_os = "android", jnix(default))]
    pub openvpn_constraints: Option<OpenVpnConstraints>,
}

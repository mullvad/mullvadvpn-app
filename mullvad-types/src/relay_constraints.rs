//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.

use crate::{
    custom_list::{CustomListsSettings, Id},
    location::{CityCode, CountryCode, Hostname},
    relay_list::Relay,
    CustomTunnelEndpoint,
};
#[cfg(target_os = "android")]
use jnix::{jni::objects::JObject, FromJava, IntoJava, JnixEnv};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt, str::FromStr};
use talpid_types::net::{openvpn::ProxySettings, IpVersion, TransportProtocol, TunnelType};

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
pub enum Constraint<T> {
    Any,
    Only(T),
}

impl<T: fmt::Display> fmt::Display for Constraint<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Constraint::Any => "any".fmt(f),
            Constraint::Only(value) => fmt::Display::fmt(value, f),
        }
    }
}

impl<T> Constraint<T> {
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

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Constraint<U> {
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

    pub fn is_only(&self) -> bool {
        !self.is_any()
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
}

impl<T: PartialEq> Constraint<T> {
    pub fn matches_eq(&self, other: &T) -> bool {
        match self {
            Constraint::Any => true,
            Constraint::Only(ref value) => value == other,
        }
    }
}

// Using the default attribute fails on Android
#[allow(clippy::derivable_impls)]
impl<T> Default for Constraint<T> {
    fn default() -> Self {
        Constraint::Any
    }
}

impl<T: Copy> Copy for Constraint<T> {}

impl<T: Match<U>, U> Match<U> for Constraint<T> {
    fn matches(&self, other: &U) -> bool {
        match *self {
            Constraint::Any => true,
            Constraint::Only(ref value) => value.matches(other),
        }
    }
}

impl<T: Set<U>, U> Set<Constraint<U>> for Constraint<T> {
    fn is_subset(&self, other: &Constraint<U>) -> bool {
        match self {
            Constraint::Any => other.is_any(),
            Constraint::Only(ref constraint) => match other {
                Constraint::Only(ref other_constraint) => constraint.is_subset(other_constraint),
                _ => true,
            },
        }
    }
}

impl<T> From<Option<T>> for Constraint<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Constraint::Only(value),
            None => Constraint::Any,
        }
    }
}

impl<T: fmt::Debug + Clone + FromStr> FromStr for Constraint<T> {
    type Err = T::Err;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("any") {
            return Ok(Self::Any);
        }
        Ok(Self::Only(T::from_str(value)?))
    }
}

#[cfg(feature = "clap")]
impl<T: fmt::Debug + Clone + clap::builder::ValueParserFactory> clap::builder::ValueParserFactory
    for Constraint<T>
where
    <T as clap::builder::ValueParserFactory>::Parser: Sync + Send + Clone,
{
    type Parser = ConstraintParser<T::Parser>;

    fn value_parser() -> Self::Parser {
        ConstraintParser(T::value_parser())
    }
}

#[cfg(feature = "clap")]
#[derive(fmt::Debug, Clone)]
pub struct ConstraintParser<T>(T);

#[cfg(feature = "clap")]
impl<T: clap::builder::TypedValueParser> clap::builder::TypedValueParser for ConstraintParser<T>
where
    T::Value: fmt::Debug,
{
    type Value = Constraint<T::Value>;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        if value.eq_ignore_ascii_case("any") {
            return Ok(Constraint::Any);
        }
        self.0.parse_ref(cmd, arg, value).map(Constraint::Only)
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

impl RelaySettings {
    /// Returns false if the specified relay settings update explicitly do not allow for bridging
    /// (i.e. use UDP instead of TCP)
    pub fn supports_bridge(&self) -> bool {
        match &self {
            RelaySettings::CustomTunnelEndpoint(endpoint) => {
                endpoint.endpoint().protocol == TransportProtocol::Tcp
            }
            RelaySettings::Normal(update) => !matches!(
                &update.openvpn_constraints,
                OpenVpnConstraints {
                    port: Constraint::Only(TransportPort {
                        protocol: TransportProtocol::Udp,
                        ..
                    })
                }
            ),
        }
    }
}

pub struct RelaySettingsFormatter<'a> {
    pub settings: &'a RelaySettings,
    pub custom_lists: &'a CustomListsSettings,
}

impl<'a> fmt::Display for RelaySettingsFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.settings {
            RelaySettings::CustomTunnelEndpoint(endpoint) => {
                write!(f, "custom endpoint {endpoint}")
            }
            RelaySettings::Normal(constraints) => {
                write!(
                    f,
                    "{}",
                    RelayConstraintsFormatter {
                        constraints,
                        custom_lists: self.custom_lists
                    }
                )
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub enum LocationConstraint {
    Location(GeographicLocationConstraint),
    CustomList { list_id: Id },
}

#[derive(Debug, Clone)]
pub enum ResolvedLocationConstraint {
    Location(GeographicLocationConstraint),
    Locations(Vec<GeographicLocationConstraint>),
}

impl ResolvedLocationConstraint {
    pub fn from_constraint(
        location: Constraint<LocationConstraint>,
        custom_lists: &CustomListsSettings,
    ) -> Constraint<ResolvedLocationConstraint> {
        match location {
            Constraint::Any => Constraint::Any,
            Constraint::Only(LocationConstraint::Location(location)) => {
                Constraint::Only(Self::Location(location))
            }
            Constraint::Only(LocationConstraint::CustomList { list_id }) => custom_lists
                .iter()
                .find(|list| list.id == list_id)
                .map(|custom_list| {
                    Constraint::Only(Self::Locations(
                        custom_list.locations.iter().cloned().collect(),
                    ))
                })
                .unwrap_or_else(|| {
                    log::warn!("Resolved non-existent custom list");
                    Constraint::Only(ResolvedLocationConstraint::Locations(vec![]))
                }),
        }
    }
}

impl From<GeographicLocationConstraint> for LocationConstraint {
    fn from(location: GeographicLocationConstraint) -> Self {
        Self::Location(location)
    }
}

impl Set<Constraint<ResolvedLocationConstraint>> for Constraint<ResolvedLocationConstraint> {
    fn is_subset(&self, other: &Self) -> bool {
        match self {
            Constraint::Any => other.is_any(),
            Constraint::Only(ResolvedLocationConstraint::Location(location)) => match other {
                Constraint::Any => true,
                Constraint::Only(ResolvedLocationConstraint::Location(other_location)) => {
                    location.is_subset(other_location)
                }
                Constraint::Only(ResolvedLocationConstraint::Locations(other_locations)) => {
                    other_locations
                        .iter()
                        .any(|other_location| location.is_subset(other_location))
                }
            },
            Constraint::Only(ResolvedLocationConstraint::Locations(locations)) => match other {
                Constraint::Any => true,
                Constraint::Only(ResolvedLocationConstraint::Location(other_location)) => locations
                    .iter()
                    .all(|location| location.is_subset(other_location)),
                Constraint::Only(ResolvedLocationConstraint::Locations(other_locations)) => {
                    for location in locations {
                        if !other_locations
                            .iter()
                            .any(|other_location| location.is_subset(other_location))
                        {
                            return false;
                        }
                    }
                    true
                }
            },
        }
    }
}

impl Constraint<ResolvedLocationConstraint> {
    pub fn matches_with_opts(&self, relay: &Relay, ignore_include_in_country: bool) -> bool {
        match self {
            Constraint::Any => true,
            Constraint::Only(ResolvedLocationConstraint::Location(location)) => {
                location.matches_with_opts(relay, ignore_include_in_country)
            }
            Constraint::Only(ResolvedLocationConstraint::Locations(locations)) => locations
                .iter()
                .any(|loc| loc.matches_with_opts(relay, ignore_include_in_country)),
        }
    }
}

pub struct LocationConstraintFormatter<'a> {
    pub constraint: &'a LocationConstraint,
    pub custom_lists: &'a CustomListsSettings,
}

impl<'a> fmt::Display for LocationConstraintFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.constraint {
            LocationConstraint::Location(location) => write!(f, "{}", location),
            LocationConstraint::CustomList { list_id } => self
                .custom_lists
                .iter()
                .find(|list| &list.id == list_id)
                .map(|custom_list| write!(f, "{}", custom_list.name))
                .unwrap_or_else(|| write!(f, "invalid custom list")),
        }
    }
}

/// Limits the set of [`crate::relay_list::Relay`]s that a `RelaySelector` may select.
#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct RelayConstraints {
    pub location: Constraint<LocationConstraint>,
    pub providers: Constraint<Providers>,
    pub ownership: Constraint<Ownership>,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub tunnel_protocol: Constraint<TunnelType>,
    pub wireguard_constraints: WireguardConstraints,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub openvpn_constraints: OpenVpnConstraints,
}

pub struct RelayConstraintsFormatter<'a> {
    pub constraints: &'a RelayConstraints,
    pub custom_lists: &'a CustomListsSettings,
}

impl<'a> fmt::Display for RelayConstraintsFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Tunnel protocol: {}\nOpenVPN constraints: {}\nWireguard constraints: {}",
            self.constraints.tunnel_protocol,
            self.constraints.openvpn_constraints,
            WireguardConstraintsFormatter {
                constraints: &self.constraints.wireguard_constraints,
                custom_lists: self.custom_lists,
            },
        )?;
        writeln!(
            f,
            "Location: {}",
            self.constraints
                .location
                .as_ref()
                .map(|location| LocationConstraintFormatter {
                    constraint: location,
                    custom_lists: self.custom_lists,
                })
        )?;
        writeln!(f, "Provider(s): {}", self.constraints.providers)?;
        write!(f, "Ownership: {}", self.constraints.ownership)
    }
}

/// Limits the set of [`crate::relay_list::Relay`]s used by a `RelaySelector` based on
/// location.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub enum GeographicLocationConstraint {
    /// A country is represented by its two letter country code.
    Country(CountryCode),
    /// A city is composed of a country code and a city code.
    City(CountryCode, CityCode),
    /// An single hostname in a given city.
    Hostname(CountryCode, CityCode, Hostname),
}

impl GeographicLocationConstraint {
    pub fn matches_with_opts(&self, relay: &Relay, ignore_include_in_country: bool) -> bool {
        match self {
            GeographicLocationConstraint::Country(ref country) => {
                relay
                    .location
                    .as_ref()
                    .map_or(false, |loc| loc.country_code == *country)
                    && (ignore_include_in_country || relay.include_in_country)
            }
            GeographicLocationConstraint::City(ref country, ref city) => {
                relay.location.as_ref().map_or(false, |loc| {
                    loc.country_code == *country && loc.city_code == *city
                })
            }
            GeographicLocationConstraint::Hostname(ref country, ref city, ref hostname) => {
                relay.location.as_ref().map_or(false, |loc| {
                    loc.country_code == *country
                        && loc.city_code == *city
                        && relay.hostname == *hostname
                })
            }
        }
    }
}

impl Constraint<Vec<GeographicLocationConstraint>> {
    pub fn matches_with_opts(&self, relay: &Relay, ignore_include_in_country: bool) -> bool {
        match self {
            Constraint::Only(constraint) => constraint
                .iter()
                .any(|loc| loc.matches_with_opts(relay, ignore_include_in_country)),
            Constraint::Any => true,
        }
    }
}

impl Constraint<GeographicLocationConstraint> {
    pub fn matches_with_opts(&self, relay: &Relay, ignore_include_in_country: bool) -> bool {
        match self {
            Constraint::Only(constraint) => {
                constraint.matches_with_opts(relay, ignore_include_in_country)
            }
            Constraint::Any => true,
        }
    }
}

impl Match<Relay> for GeographicLocationConstraint {
    fn matches(&self, relay: &Relay) -> bool {
        self.matches_with_opts(relay, false)
    }
}

impl Set<GeographicLocationConstraint> for GeographicLocationConstraint {
    /// Returns whether `self` is equal to or a subset of `other`.
    fn is_subset(&self, other: &Self) -> bool {
        match self {
            GeographicLocationConstraint::Country(_) => self == other,
            GeographicLocationConstraint::City(ref country, ref _city) => match other {
                GeographicLocationConstraint::Country(ref other_country) => {
                    country == other_country
                }
                GeographicLocationConstraint::City(..) => self == other,
                _ => false,
            },
            GeographicLocationConstraint::Hostname(ref country, ref city, ref _hostname) => {
                match other {
                    GeographicLocationConstraint::Country(ref other_country) => {
                        country == other_country
                    }
                    GeographicLocationConstraint::City(ref other_country, ref other_city) => {
                        country == other_country && city == other_city
                    }
                    GeographicLocationConstraint::Hostname(..) => self == other,
                }
            }
        }
    }
}

impl Set<Constraint<Vec<GeographicLocationConstraint>>>
    for Constraint<Vec<GeographicLocationConstraint>>
{
    fn is_subset(&self, other: &Self) -> bool {
        match self {
            Constraint::Any => other.is_any(),
            Constraint::Only(locations) => match other {
                Constraint::Any => true,
                Constraint::Only(other_locations) => locations.iter().all(|location| {
                    other_locations
                        .iter()
                        .any(|other_location| location.is_subset(other_location))
                }),
            },
        }
    }
}

/// Limits the set of servers to choose based on ownership.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(target_os = "android", derive(IntoJava, FromJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Ownership {
    MullvadOwned,
    Rented,
}

impl Match<Relay> for Ownership {
    fn matches(&self, relay: &Relay) -> bool {
        match self {
            Ownership::MullvadOwned => relay.owned,
            Ownership::Rented => !relay.owned,
        }
    }
}

impl fmt::Display for Ownership {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Ownership::MullvadOwned => write!(f, "Mullvad-owned servers"),
            Ownership::Rented => write!(f, "rented servers"),
        }
    }
}

impl FromStr for Ownership {
    type Err = OwnershipParseError;

    fn from_str(s: &str) -> Result<Ownership, Self::Err> {
        match s {
            "owned" | "mullvad-owned" => Ok(Ownership::MullvadOwned),
            "rented" => Ok(Ownership::Rented),
            _ => Err(OwnershipParseError),
        }
    }
}

/// Returned when `Ownership::from_str` fails to convert a string into a
/// [`Ownership`] object.
#[derive(err_derive::Error, Debug, Clone, PartialEq, Eq)]
#[error(display = "Not a valid ownership setting")]
pub struct OwnershipParseError;

/// Limits the set of [`crate::relay_list::Relay`]s used by a `RelaySelector` based on
/// provider.
pub type Provider = String;

#[cfg_attr(target_os = "android", derive(IntoJava, FromJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Providers {
    providers: HashSet<Provider>,
}

/// Returned if the iterator contained no providers.
#[derive(Debug)]
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

    pub fn into_vec(self) -> Vec<Provider> {
        self.providers.into_iter().collect()
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
                write!(f, "{provider}")?;
            } else {
                write!(f, ", {provider}")?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for GeographicLocationConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            GeographicLocationConstraint::Country(country) => write!(f, "country {country}"),
            GeographicLocationConstraint::City(country, city) => {
                write!(f, "city {city}, {country}")
            }
            GeographicLocationConstraint::Hostname(country, city, hostname) => {
                write!(f, "city {city}, {country}, hostname {hostname}")
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct TransportPort {
    pub protocol: TransportProtocol,
    pub port: Constraint<u16>,
}

/// [`Constraint`]s applicable to OpenVPN relays.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct OpenVpnConstraints {
    pub port: Constraint<TransportPort>,
}

impl fmt::Display for OpenVpnConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.port {
            Constraint::Any => write!(f, "any port"),
            Constraint::Only(port) => {
                match port.port {
                    Constraint::Any => write!(f, "any port")?,
                    Constraint::Only(port) => write!(f, "port {port}")?,
                }
                write!(f, "/{}", port.protocol)
            }
        }
    }
}

/// [`Constraint`]s applicable to WireGuard relays.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[serde(rename_all = "snake_case", default)]
pub struct WireguardConstraints {
    #[cfg_attr(
        target_os = "android",
        jnix(map = "|constraint| constraint.map(|v| Port { value: v as i32 })")
    )]
    pub port: Constraint<u16>,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub ip_version: Constraint<IpVersion>,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub use_multihop: bool,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub entry_location: Constraint<LocationConstraint>,
}

pub struct WireguardConstraintsFormatter<'a> {
    pub constraints: &'a WireguardConstraints,
    pub custom_lists: &'a CustomListsSettings,
}

impl<'a> fmt::Display for WireguardConstraintsFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.constraints.port {
            Constraint::Any => write!(f, "any port")?,
            Constraint::Only(port) => write!(f, "port {}", port)?,
        }
        if let Constraint::Only(ip_version) = self.constraints.ip_version {
            write!(f, ", {},", ip_version)?;
        }
        if self.constraints.use_multihop {
            let location = self.constraints.entry_location.as_ref().map(|location| {
                LocationConstraintFormatter {
                    constraint: location,
                    custom_lists: self.custom_lists,
                }
            });
            write!(f, ", multihop entry {}", location)?;
        }
        Ok(())
    }
}

#[cfg(target_os = "android")]
impl<'env, 'sub_env> FromJava<'env, JObject<'sub_env>> for WireguardConstraints
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "Lnet/mullvad/mullvadvpn/model/WireguardConstraints;";

    fn from_java(env: &JnixEnv<'env>, object: JObject<'sub_env>) -> Self {
        let object = env
            .call_method(
                object,
                "component1",
                "()Lnet/mullvad/mullvadvpn/model/Constraint;",
                &[],
            )
            .expect("missing WireguardConstraints.port")
            .l()
            .expect("WireguardConstraints.port did not return an object");

        let port: Constraint<Port> = Constraint::from_java(env, object);

        WireguardConstraints {
            port: port.map(|port| port.value as u16),
            ..Default::default()
        }
    }
}

/// Used for jni conversion.
#[cfg(target_os = "android")]
#[derive(Debug, Default, Clone, Eq, PartialEq, FromJava, IntoJava)]
#[jnix(package = "net.mullvad.mullvadvpn.model")]
struct Port {
    value: i32,
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

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum SelectedObfuscation {
    Auto,
    #[default]
    Off,
    #[cfg_attr(feature = "clap", clap(name = "udp2tcp"))]
    Udp2Tcp,
}

impl fmt::Display for SelectedObfuscation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectedObfuscation::Auto => "auto".fmt(f),
            SelectedObfuscation::Off => "off".fmt(f),
            SelectedObfuscation::Udp2Tcp => "udp2tcp".fmt(f),
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[serde(rename_all = "snake_case")]
pub struct Udp2TcpObfuscationSettings {
    #[cfg_attr(
        target_os = "android",
        jnix(map = "|constraint| constraint.map(|v| v as i32)")
    )]
    pub port: Constraint<u16>,
}

#[cfg(target_os = "android")]
impl<'env, 'sub_env> FromJava<'env, JObject<'sub_env>> for Udp2TcpObfuscationSettings
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "Lnet/mullvad/mullvadvpn/model/Udp2TcpObfuscationSettings;";

    fn from_java(env: &JnixEnv<'env>, object: JObject<'sub_env>) -> Self {
        let object = env
            .call_method(
                object,
                "component1",
                "()Lnet/mullvad/mullvadvpn/model/Constraint;",
                &[],
            )
            .expect("missing Udp2TcpObfuscationSettings.port")
            .l()
            .expect("Udp2TcpObfuscationSettings.port did not return an object");

        let port: Constraint<i32> = Constraint::from_java(env, object);

        Udp2TcpObfuscationSettings {
            port: port.map(|port| port as u16),
        }
    }
}

impl fmt::Display for Udp2TcpObfuscationSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.port {
            Constraint::Any => write!(f, "any port"),
            Constraint::Only(port) => write!(f, "port {port}"),
        }
    }
}

/// Contains obfuscation settings
#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct ObfuscationSettings {
    pub selected_obfuscation: SelectedObfuscation,
    pub udp2tcp: Udp2TcpObfuscationSettings,
}

/// Limits the set of bridge servers to use in `mullvad-daemon`.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
#[serde(rename_all = "snake_case")]
pub struct BridgeConstraints {
    pub location: Constraint<LocationConstraint>,
    pub providers: Constraint<Providers>,
    pub ownership: Constraint<Ownership>,
}

pub struct BridgeConstraintsFormatter<'a> {
    pub constraints: &'a BridgeConstraints,
    pub custom_lists: &'a CustomListsSettings,
}

impl<'a> fmt::Display for BridgeConstraintsFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.constraints.location {
            Constraint::Any => write!(f, "any location")?,
            Constraint::Only(ref constraint) => write!(
                f,
                "{}",
                LocationConstraintFormatter {
                    constraint,
                    custom_lists: self.custom_lists,
                }
            )?,
        }
        write!(f, " using ")?;
        match self.constraints.providers {
            Constraint::Any => write!(f, "any provider")?,
            Constraint::Only(ref constraint) => write!(f, "{}", constraint)?,
        }
        match self.constraints.ownership {
            Constraint::Any => Ok(()),
            Constraint::Only(ref constraint) => {
                write!(f, " and {constraint}")
            }
        }
    }
}

/// Setting indicating whether to connect to a bridge server, or to handle it automatically.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
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
    pub ownership: Constraint<Ownership>,
    pub transport_protocol: Constraint<TransportProtocol>,
}

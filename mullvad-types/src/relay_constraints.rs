//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.

use crate::{
    constraints::{Constraint, Intersection, Match, Set},
    custom_list::{CustomListsSettings, Id},
    location::{CityCode, CountryCode, Hostname},
    relay_list::Relay,
    CustomTunnelEndpoint,
};
#[cfg(target_os = "android")]
use jnix::{jni::objects::JObject, FromJava, IntoJava, JnixEnv};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt,
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};
use talpid_types::net::{proxy::CustomProxy, IpVersion, TransportProtocol, TunnelType};

/// Specifies a specific endpoint or [`RelayConstraints`] to use when `mullvad-daemon` selects a
/// relay.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(IntoJava, FromJava))]
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

impl RelayConstraints {
    /// Create a new [`RelayConstraints`] with no opinionated defaults. This
    /// should be the const equivalent to [`Default::default`].
    pub const fn new() -> RelayConstraints {
        RelayConstraints {
            location: Constraint::Any,
            providers: Constraint::Any,
            ownership: Constraint::Any,
            tunnel_protocol: Constraint::Any,
            wireguard_constraints: WireguardConstraints::new(),
            openvpn_constraints: OpenVpnConstraints::new(),
        }
    }
}

// `Intersection` on [`RelayConstraints`] represents a cautious merge strategy
// for [`RelayConstraints`].
//
// If two [`RelayConstraints`] differ in any configuration such that running
// [`Constraint::intersection`] between the configurations returns
// [`Option::None`], no concenssus can be reached and the two
// [`RelayConstraints`] are said to be incompatible.
//
// This way, if the mullvad app wants to check if the user's configured
// [`RelayConstraints`] are compatible with some pre-configured
// [`RelayConstraints`], taking the `intersection` between those two will never
// result in a situation where the app can override the user's preferences.
impl Intersection for RelayConstraints {
    fn intersection(self, other: Self) -> Option<Self>
    where
        Self: PartialEq,
        Self: Sized,
    {
        Some(RelayConstraints {
            location: self.location.intersection(other.location)?,
            providers: self.providers.intersection(other.providers)?,
            ownership: self.ownership.intersection(other.ownership)?,
            tunnel_protocol: self.tunnel_protocol.intersection(other.tunnel_protocol)?,
            wireguard_constraints: self
                .wireguard_constraints
                .intersection(other.wireguard_constraints)?,
            openvpn_constraints: self
                .openvpn_constraints
                .intersection(other.openvpn_constraints)?,
        })
    }
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

#[cfg(target_os = "android")]
impl<'env, 'sub_env> FromJava<'env, JObject<'sub_env>> for RelayConstraints
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "Lnet/mullvad/mullvadvpn/model/RelayConstraints;";

    fn from_java(env: &JnixEnv<'env>, object: JObject<'sub_env>) -> Self {
        let object_location = env
            .call_method(
                object,
                "component1",
                "()Lnet/mullvad/mullvadvpn/model/Constraint;",
                &[],
            )
            .expect("missing RelayConstraints.location")
            .l()
            .expect("RelayConstraints.location did not return an object");

        let location: Constraint<LocationConstraint> = Constraint::from_java(env, object_location);

        let object_providers = env
            .call_method(
                object,
                "component2",
                "()Lnet/mullvad/mullvadvpn/model/Constraint;",
                &[],
            )
            .expect("missing RelayConstraints.providers")
            .l()
            .expect("RelayConstraints.providers did not return an object");

        let providers: Constraint<Providers> = Constraint::from_java(env, object_providers);

        let object_ownership = env
            .call_method(
                object,
                "component3",
                "()Lnet/mullvad/mullvadvpn/model/Constraint;",
                &[],
            )
            .expect("missing RelayConstraints.providers")
            .l()
            .expect("RelayConstraints.providers did not return an object");

        let ownership: Constraint<Ownership> = Constraint::from_java(env, object_ownership);

        let object_wireguard_constraints = env
            .call_method(
                object,
                "component4",
                "()Lnet/mullvad/mullvadvpn/model/WireguardConstraints;",
                &[],
            )
            .expect("missing RelayConstraints.wireguard_constraints")
            .l()
            .expect("RelayConstraints.wireguard_constraints did not return an object");

        let wireguard_constraints: WireguardConstraints =
            WireguardConstraints::from_java(env, object_wireguard_constraints);

        RelayConstraints {
            location,
            providers,
            ownership,
            wireguard_constraints,
            ..Default::default()
        }
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

impl OpenVpnConstraints {
    /// Create a new [`OpenVpnConstraints`] with no opinionated defaults. This
    /// should be the const equivalent to [`Default::default`].
    pub const fn new() -> OpenVpnConstraints {
        OpenVpnConstraints {
            port: Constraint::Any,
        }
    }
}

impl Intersection for OpenVpnConstraints {
    fn intersection(self, other: Self) -> Option<Self>
    where
        Self: PartialEq,
        Self: Sized,
    {
        Some(OpenVpnConstraints {
            port: self.port.intersection(other.port)?,
        })
    }
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

impl WireguardConstraints {
    /// Create a new [`WireguardConstraints`] with no opinionated defaults. This
    /// should be the const equivalent to [`Default::default`].
    pub const fn new() -> WireguardConstraints {
        WireguardConstraints {
            port: Constraint::Any,
            ip_version: Constraint::Any,
            use_multihop: false,
            entry_location: Constraint::Any,
        }
    }
}

impl Intersection for WireguardConstraints {
    fn intersection(self, other: Self) -> Option<Self>
    where
        Self: PartialEq,
        Self: Sized,
    {
        // Since `Intersection` is not defined on `bool`, this becomes a bit ad-hoc.
        // Let's define intersection on `bool` as
        // true  ∩ true  = Some(true)
        // false ∩ false = Some(false)
        // otherwise     = None
        let use_multihop = if self.use_multihop == other.use_multihop {
            Some(self.use_multihop)
        } else {
            None
        };
        Some(WireguardConstraints {
            port: self.port.intersection(other.port)?,
            ip_version: self.ip_version.intersection(other.ip_version)?,
            use_multihop: use_multihop?,
            entry_location: self.entry_location.intersection(other.entry_location)?,
        })
    }
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

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeType {
    /// Let the relay selection algorithm decide on bridges, based on the relay list
    /// and normal bridge constraints.
    #[default]
    Normal,
    /// Use custom bridge configuration.
    Custom,
}

impl fmt::Display for BridgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            BridgeType::Normal => f.write_str("normal"),
            BridgeType::Custom => f.write_str("custom"),
        }
    }
}

#[derive(err_derive::Error, Debug)]
#[error(display = "Missing custom bridge settings")]
pub struct MissingCustomBridgeSettings(());

/// Specifies a specific endpoint or [`BridgeConstraints`] to use when `mullvad-daemon` selects a
/// bridge server.
#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BridgeSettings {
    pub bridge_type: BridgeType,
    pub normal: BridgeConstraints,
    pub custom: Option<CustomProxy>,
}

pub enum ResolvedBridgeSettings<'a> {
    Normal(&'a BridgeConstraints),
    Custom(&'a CustomProxy),
}

impl BridgeSettings {
    pub fn resolve(&self) -> Result<ResolvedBridgeSettings<'_>, MissingCustomBridgeSettings> {
        match (self.bridge_type, &self.custom) {
            (BridgeType::Normal, _) => Ok(ResolvedBridgeSettings::Normal(&self.normal)),
            (BridgeType::Custom, Some(custom)) => Ok(ResolvedBridgeSettings::Custom(custom)),
            (BridgeType::Custom, None) => Err(MissingCustomBridgeSettings(())),
        }
    }
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

/// Options to override for a particular relay to use instead of the ones specified in the relay
/// list
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct RelayOverride {
    /// Hostname for which to override the given options
    pub hostname: Hostname,
    /// IPv4 address to use instead of the default
    pub ipv4_addr_in: Option<Ipv4Addr>,
    /// IPv6 address to use instead of the default
    pub ipv6_addr_in: Option<Ipv6Addr>,
}

impl RelayOverride {
    pub fn empty(hostname: Hostname) -> RelayOverride {
        RelayOverride {
            hostname,
            ipv4_addr_in: None,
            ipv6_addr_in: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::empty(self.hostname.clone())
    }

    pub fn apply_to_relay(&self, relay: &mut Relay) {
        if let Some(ipv4_addr_in) = self.ipv4_addr_in {
            log::debug!(
                "Overriding ipv4_addr_in for {}: {ipv4_addr_in}",
                relay.hostname
            );
            relay.ipv4_addr_in = ipv4_addr_in;
        }
        if let Some(ipv6_addr_in) = self.ipv6_addr_in {
            log::debug!(
                "Overriding ipv6_addr_in for {}: {ipv6_addr_in}",
                relay.hostname
            );
            relay.ipv6_addr_in = Some(ipv6_addr_in);
        }
    }
}

#[allow(dead_code)]
pub mod builder {
    //! Strongly typed Builder pattern for of relay constraints though the use of the Typestate pattern.
    use super::RelayConstraints;
    pub use super::{LocationConstraint, Ownership, Providers};
    use crate::constraints::Constraint;

    /// Internal builder state for a [`RelayConstraint`] parameterized over the
    /// type of VPN tunnel protocol. Some [`RelayConstraint`] options are
    /// generic over the VPN protocol, while some options are protocol-specific.
    ///
    /// - The type parameter `VpnProtocol` keeps track of which VPN protocol that
    /// is being configured. Different instantiations of `VpnProtocol` will
    /// expose different functions for configuring a [`RelayConstraintBuilder`]
    /// further.
    pub struct RelayConstraintBuilder<VpnProtocol> {
        constraints: RelayConstraints,
        protocol: VpnProtocol,
    }

    ///  The `Any` type is equivalent to the `Constraint::Any` value. If a
    ///  type-parameter is of type `Any`, it means that the corresponding value
    ///  in the final `RelayConstraint` is `Constraint::Any`.
    pub struct Any;

    /// Create a new [`RelayConstraintBuilder`] with unopinionated defaults.
    pub const fn any() -> RelayConstraintBuilder<Any> {
        RelayConstraintBuilder::new(Any)
    }

    // This impl-block is quantified over all configurations, e.g. [`Any`],
    // [`WireguardConstraints`] & [`OpenVpnConstraints`]
    impl<VpnProtocol> RelayConstraintBuilder<VpnProtocol> {
        const fn new(protocol: VpnProtocol) -> RelayConstraintBuilder<VpnProtocol> {
            RelayConstraintBuilder {
                constraints: RelayConstraints::new(),
                protocol,
            }
        }

        /// Configure the [`LocationConstraint`] to use.
        pub fn location(mut self, location: LocationConstraint) -> Self {
            self.constraints.location = Constraint::Only(location);
            self
        }

        /// Configure which [`Ownership`] to use.
        pub const fn ownership(mut self, ownership: Ownership) -> Self {
            self.constraints.ownership = Constraint::Only(ownership);
            self
        }

        /// Configure which [`Providers`] to use.
        pub fn providers(mut self, providers: Providers) -> Self {
            self.constraints.providers = Constraint::Only(providers);
            self
        }

        /// Assemble the final [`RelayConstraints`] that has been configured
        /// through `self`.
        pub fn build(self) -> RelayConstraints {
            self.constraints
        }
    }

    pub mod wireguard {
        //! Type-safe builder for Wireguard relay constraints.
        use super::{Any, RelayConstraintBuilder};
        use crate::{constraints::Constraint, relay_constraints::WireguardConstraints};
        // Re-exports
        pub use super::LocationConstraint;
        pub use talpid_types::net::IpVersion;

        /// Internal builder state for a [`WireguardConstraints`] configuration.
        ///
        /// - The type parameter `Multihop` keeps track of the state of multihop.
        /// If multihop has been enabled, the builder should expose an option to
        /// select entry point.
        pub struct Wireguard<Multihop> {
            multihop: Multihop,
        }

        /// Create a new Wireguard-oriented [`RelayConstraintBuilder`] with
        /// otherwise unopinionated defaults.
        pub const fn new() -> RelayConstraintBuilder<Wireguard<Any>> {
            RelayConstraintBuilder::new(Wireguard { multihop: Any })
        }

        // This impl-block is quantified over all configurations
        impl<Multihop> RelayConstraintBuilder<Wireguard<Multihop>> {
            pub const fn port(mut self, port: u16) -> Self {
                self.constraints.wireguard_constraints.port = Constraint::Only(port);
                self
            }

            pub const fn ip_version(mut self, ip_version: IpVersion) -> Self {
                self.constraints.wireguard_constraints.ip_version = Constraint::Only(ip_version);
                self
            }

            /// Extract the underlying [`WireguardConstraints`] from `self`.
            pub fn into_constraints(self) -> WireguardConstraints {
                self.build().wireguard_constraints
            }
        }

        impl RelayConstraintBuilder<Wireguard<Any>> {
            /// Enable multihop
            pub fn multihop(mut self) -> RelayConstraintBuilder<Wireguard<bool>> {
                self.constraints.wireguard_constraints.use_multihop = true;
                // Update the type state
                RelayConstraintBuilder {
                    constraints: self.constraints,
                    protocol: Wireguard { multihop: true },
                }
            }
        }

        impl RelayConstraintBuilder<Wireguard<bool>> {
            /// Set the entry location in a multihop configuration. This requires
            /// multihop to be enabled.
            pub fn entry(mut self, location: LocationConstraint) -> Self {
                self.constraints.wireguard_constraints.entry_location = Constraint::Only(location);
                self
            }
        }
    }

    pub mod openvpn {
        //! Type-safe builder pattern for OpenVPN relay constraints.
        use super::{Any, RelayConstraintBuilder};
        use crate::constraints::Constraint;
        use crate::relay_constraints::{OpenVpnConstraints, TransportPort};
        // Re-exports
        pub use talpid_types::net::TransportProtocol;

        /// Internal builder state for a [`OpenVPNConstraints`] configuration.
        ///
        /// - The type parameter `TransportPort` keeps track of which
        /// [`TransportProtocol`] & port-combo to use. [`TransportProtocol`] has
        /// to be set first before the option to select a specific port is
        /// exposed.
        pub struct OpenVPN<TransportPort> {
            transport_port: TransportPort,
        }

        /// Create a new OpenVPN-oriented [`RelayConstraintBuilder`] with
        /// otherwise unopinionated defaults.
        pub const fn new() -> RelayConstraintBuilder<OpenVPN<Any>> {
            RelayConstraintBuilder::new(OpenVPN {
                transport_port: Any,
            })
        }

        // This impl-block is quantified over all configurations
        impl<TransportPort> RelayConstraintBuilder<OpenVPN<TransportPort>> {
            /// Extract the underlying [`OpenVpnConstraints`] from `self`.
            pub fn into_constraints(self) -> OpenVpnConstraints {
                self.build().openvpn_constraints
            }
        }

        impl RelayConstraintBuilder<OpenVPN<Any>> {
            /// Configure what [`TransportProtocol`] to use. Calling this
            /// function on a builder will expose the option to select which
            /// port to use in combination with `protocol`.
            pub fn transport_protocol(
                mut self,
                protocol: TransportProtocol,
            ) -> RelayConstraintBuilder<OpenVPN<TransportPort>> {
                let transport_port = TransportPort {
                    protocol,
                    // The port has not been configured yet
                    port: Constraint::Any,
                };
                self.constraints.openvpn_constraints.port = Constraint::Only(transport_port);
                // Update the type state
                RelayConstraintBuilder {
                    constraints: self.constraints,
                    protocol: OpenVPN { transport_port },
                }
            }
        }

        impl RelayConstraintBuilder<OpenVPN<TransportPort>> {
            /// Configure what port to use when connecting to a relay.
            pub const fn port(mut self, port: u16) -> Self {
                let port = Constraint::Only(port);
                let mut transport_port = self.protocol.transport_port;
                transport_port.port = port;
                self.constraints.openvpn_constraints.port = Constraint::Only(transport_port);
                self
            }
        }
    }
}

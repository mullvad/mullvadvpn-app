//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.

use crate::{
    constraints::{Constraint, Match},
    custom_list::{CustomListsSettings, Id},
    location::{CityCode, CountryCode, Hostname},
    relay_list::{Relay, RelayEndpointData},
    CustomTunnelEndpoint, Intersection,
};
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

impl From<CustomTunnelEndpoint> for RelaySettings {
    fn from(value: CustomTunnelEndpoint) -> Self {
        Self::CustomTunnelEndpoint(value)
    }
}

impl From<RelayConstraints> for RelaySettings {
    fn from(value: RelayConstraints) -> Self {
        Self::Normal(value)
    }
}

pub struct RelaySettingsFormatter<'a> {
    pub settings: &'a RelaySettings,
    pub custom_lists: &'a CustomListsSettings,
}

impl fmt::Display for RelaySettingsFormatter<'_> {
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
pub enum LocationConstraint {
    Location(GeographicLocationConstraint),
    CustomList { list_id: Id },
}

pub struct LocationConstraintFormatter<'a> {
    pub constraint: &'a LocationConstraint,
    pub custom_lists: &'a CustomListsSettings,
}

impl From<GeographicLocationConstraint> for LocationConstraint {
    fn from(location: GeographicLocationConstraint) -> Self {
        Self::Location(location)
    }
}

impl fmt::Display for LocationConstraintFormatter<'_> {
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
pub struct RelayConstraints {
    pub location: Constraint<LocationConstraint>,
    pub providers: Constraint<Providers>,
    pub ownership: Constraint<Ownership>,
    pub tunnel_protocol: TunnelType,
    pub wireguard_constraints: WireguardConstraints,
    pub openvpn_constraints: OpenVpnConstraints,
}

pub struct RelayConstraintsFormatter<'a> {
    pub constraints: &'a RelayConstraints,
    pub custom_lists: &'a CustomListsSettings,
}

impl fmt::Display for RelayConstraintsFormatter<'_> {
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
pub enum GeographicLocationConstraint {
    /// A country is represented by its two letter country code.
    Country(CountryCode),
    /// A city is composed of a country code and a city code.
    City(CountryCode, CityCode),
    /// An single hostname in a given city.
    Hostname(CountryCode, CityCode, Hostname),
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to parse {input} into a geographic location constraint")]
pub struct ParseGeoLocationError {
    input: String,
}

impl GeographicLocationConstraint {
    /// Create a new [`GeographicLocationConstraint`] given a country.
    pub fn country(country: impl Into<String>) -> Self {
        GeographicLocationConstraint::Country(country.into())
    }

    /// Create a new [`GeographicLocationConstraint`] given a country and city.
    pub fn city(country: impl Into<String>, city: impl Into<String>) -> Self {
        GeographicLocationConstraint::City(country.into(), city.into())
    }

    /// Create a new [`GeographicLocationConstraint`] given a country, city and hostname.
    pub fn hostname(
        country: impl Into<String>,
        city: impl Into<String>,
        hostname: impl Into<String>,
    ) -> Self {
        GeographicLocationConstraint::Hostname(country.into(), city.into(), hostname.into())
    }

    /// Check if `self` is _just_ a country. See [`GeographicLocationConstraint`] for more details.
    pub fn is_country(&self) -> bool {
        matches!(self, GeographicLocationConstraint::Country(_))
    }

    pub fn get_hostname(&self) -> Option<&Hostname> {
        match self {
            GeographicLocationConstraint::Hostname(_, _, hostname) => Some(hostname),
            _ => None,
        }
    }
}

impl Match<Relay> for GeographicLocationConstraint {
    fn matches(&self, relay: &Relay) -> bool {
        match self {
            GeographicLocationConstraint::Country(country) => {
                relay.location.country_code == *country
            }
            GeographicLocationConstraint::City(country, city) => {
                let loc = &relay.location;
                loc.country_code == *country && loc.city_code == *city
            }
            GeographicLocationConstraint::Hostname(country, city, hostname) => {
                let loc = &relay.location;
                loc.country_code == *country
                    && loc.city_code == *city
                    && relay.hostname == *hostname
            }
        }
    }
}

impl FromStr for GeographicLocationConstraint {
    type Err = ParseGeoLocationError;

    // TODO: Implement for country and city as well?
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // A host name, such as "se-got-wg-101" maps to
        // Country: se
        // City: got
        // hostname: se-got-wg-101
        let x = input.split("-").collect::<Vec<_>>();
        match x[..] {
            [country] => Ok(GeographicLocationConstraint::country(country)),
            [country, city] => Ok(GeographicLocationConstraint::city(country, city)),
            [country, city, ..] => Ok(GeographicLocationConstraint::hostname(country, city, input)),
            _ => Err(ParseGeoLocationError {
                input: input.to_string(),
            }),
        }
    }
}

/// Limits the set of servers to choose based on ownership.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
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
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("Not a valid ownership setting")]
pub struct OwnershipParseError;

/// Limits the set of [`crate::relay_list::Relay`]s used by a `RelaySelector` based on
/// provider.
pub type Provider = String;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Providers {
    providers: HashSet<Provider>,
}

/// Returned if the iterator contained no providers.
#[derive(Debug)]
pub struct NoProviders(());

impl Providers {
    pub fn new(
        providers: impl IntoIterator<Item = impl Into<Provider>>,
    ) -> Result<Providers, NoProviders> {
        let providers = Providers {
            providers: providers.into_iter().map(Into::into).collect(),
        };
        if providers.providers.is_empty() {
            return Err(NoProviders(()));
        }
        Ok(providers)
    }

    pub fn into_vec(self) -> Vec<Provider> {
        self.providers.into_iter().collect()
    }

    /// Access the underlying set of [providers][`Provider`]
    pub fn providers(&self) -> &HashSet<Provider> {
        &self.providers
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize, Intersection)]
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
#[serde(rename_all = "snake_case", default)]
pub struct WireguardConstraints {
    pub port: Constraint<u16>,
    pub ip_version: Constraint<IpVersion>,
    pub use_multihop: bool,
    pub entry_location: Constraint<LocationConstraint>,
}

impl WireguardConstraints {
    /// Enable or disable multihop.
    pub fn use_multihop(&mut self, multihop: bool) {
        self.use_multihop = multihop
    }

    /// Check if multihop is enabled.
    pub fn multihop(&self) -> bool {
        self.use_multihop
    }
}

pub struct WireguardConstraintsFormatter<'a> {
    pub constraints: &'a WireguardConstraints,
    pub custom_lists: &'a CustomListsSettings,
}

impl fmt::Display for WireguardConstraintsFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.constraints.port {
            Constraint::Any => write!(f, "any port")?,
            Constraint::Only(port) => write!(f, "port {}", port)?,
        }
        if let Constraint::Only(ip_version) = self.constraints.ip_version {
            write!(f, ", {},", ip_version)?;
        }
        if self.constraints.multihop() {
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

#[derive(thiserror::Error, Debug)]
#[error("Missing custom bridge settings")]
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
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum SelectedObfuscation {
    #[default]
    Auto,
    Off,
    #[cfg_attr(feature = "clap", clap(name = "udp2tcp"))]
    Udp2Tcp,
    Shadowsocks,
}

impl Intersection for SelectedObfuscation {
    fn intersection(self, other: Self) -> Option<Self>
    where
        Self: PartialEq,
        Self: Sized,
    {
        match (self, other) {
            (left, SelectedObfuscation::Auto) => Some(left),
            (SelectedObfuscation::Auto, right) => Some(right),
            (left, right) if left == right => Some(left),
            _ => None,
        }
    }
}

impl fmt::Display for SelectedObfuscation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectedObfuscation::Auto => "auto".fmt(f),
            SelectedObfuscation::Off => "off".fmt(f),
            SelectedObfuscation::Udp2Tcp => "udp2tcp".fmt(f),
            SelectedObfuscation::Shadowsocks => "shadowsocks".fmt(f),
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize, Intersection)]
#[serde(rename_all = "snake_case")]
pub struct Udp2TcpObfuscationSettings {
    pub port: Constraint<u16>,
}

impl fmt::Display for Udp2TcpObfuscationSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.port {
            Constraint::Any => write!(f, "any port"),
            Constraint::Only(port) => write!(f, "port {port}"),
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize, Intersection)]
#[serde(rename_all = "snake_case")]
pub struct ShadowsocksSettings {
    pub port: Constraint<u16>,
}

impl fmt::Display for ShadowsocksSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.port {
            Constraint::Any => write!(f, "any port"),
            Constraint::Only(port) => write!(f, "port {port}"),
        }
    }
}

/// Contains obfuscation settings
#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct ObfuscationSettings {
    pub selected_obfuscation: SelectedObfuscation,
    pub udp2tcp: Udp2TcpObfuscationSettings,
    pub shadowsocks: ShadowsocksSettings,
}

/// Limits the set of bridge servers to use in `mullvad-daemon`.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize, Intersection)]
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

impl fmt::Display for BridgeConstraintsFormatter<'_> {
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
            relay.override_ipv4(ipv4_addr_in);
        }
        if let Some(ipv6_addr_in) = self.ipv6_addr_in {
            log::debug!(
                "Overriding ipv6_addr_in for {}: {ipv6_addr_in}",
                relay.hostname
            );
            relay.override_ipv6(ipv6_addr_in);
        }

        // Additional IPs should be ignored when overrides are present
        if let RelayEndpointData::Wireguard(data) = &mut relay.endpoint_data {
            data.shadowsocks_extra_addr_in.retain(|addr| {
                let not_overridden_v4 = self.ipv4_addr_in.is_none() && addr.is_ipv4();
                let not_overridden_v6 = self.ipv6_addr_in.is_none() && addr.is_ipv6();

                // Keep address if it's not overridden
                not_overridden_v4 || not_overridden_v6
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hostname() {
        // Parse a country
        assert_eq!(
            "se".parse::<GeographicLocationConstraint>().unwrap(),
            GeographicLocationConstraint::country("se")
        );
        // Parse a city
        assert_eq!(
            "se-got".parse::<GeographicLocationConstraint>().unwrap(),
            GeographicLocationConstraint::city("se", "got")
        );
        // Parse a hostname
        assert_eq!(
            "se-got-wg-101"
                .parse::<GeographicLocationConstraint>()
                .unwrap(),
            GeographicLocationConstraint::hostname("se", "got", "se-got-wg-101")
        );
    }
}

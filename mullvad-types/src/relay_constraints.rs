//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.

use crate::{
    CustomTunnelEndpoint, Intersection,
    constraints::{Constraint, Match},
    custom_list::{CustomListsSettings, Id},
    location::{CityCode, CountryCode, Hostname},
    relay_list::{Relay, RelayEndpointData},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt,
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};
use talpid_types::net::{IpVersion, TransportProtocol, proxy::CustomProxy};

/// Specifies a specific endpoint or [`RelayConstraints`] to use when `mullvad-daemon` selects a
/// relay.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelaySettings {
    CustomTunnelEndpoint(CustomTunnelEndpoint),
    Normal(RelayConstraints),
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
            LocationConstraint::Location(location) => write!(f, "{location}"),
            LocationConstraint::CustomList { list_id } => self
                .custom_lists
                .iter()
                .find(|list| list.id() == *list_id)
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
    pub wireguard_constraints: WireguardConstraints,
}

pub struct RelayConstraintsFormatter<'a> {
    pub constraints: &'a RelayConstraints,
    pub custom_lists: &'a CustomListsSettings,
}

impl fmt::Display for RelayConstraintsFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Tunnel protocol: wireguard\nWireguard constraints: {}",
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

/// [`Constraint`]s applicable to WireGuard relays.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", default)]
pub struct WireguardConstraints {
    pub ip_version: Constraint<IpVersion>,
    pub allowed_ips: Constraint<AllowedIps>,
    pub use_multihop: bool,
    pub entry_location: Constraint<LocationConstraint>,
    pub entry_providers: Constraint<Providers>,
    pub entry_ownership: Constraint<Ownership>,
}

pub use allowed_ip::AllowedIps;
pub mod allowed_ip {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use crate::constraints::Constraint;
    use ipnetwork::IpNetwork;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
    pub struct AllowedIps(pub Vec<IpNetwork>);

    impl Default for AllowedIps {
        fn default() -> Self {
            AllowedIps::allow_all()
        }
    }

    impl std::fmt::Display for AllowedIps {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(
                &self
                    .0
                    .iter()
                    .map(|net| net.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum AllowedIpParseError {
        #[error("Failed to parse IP network: {0}")]
        Parse(#[from] ipnetwork::IpNetworkError),
        #[error("IP network {0} has non-zero host bits (should be {1})")]
        NonZeroHostBits(IpNetwork, std::net::IpAddr),
    }

    /// Represents a collection of allowed IP networks.
    ///
    /// Provides utility methods to construct `AllowedIps` from various sources,
    /// including allowing all IPs, parsing from string representations, and
    /// converting into a constraint.
    impl AllowedIps {
        /// Creates an `AllowedIps` instance that allows all IP addresses.
        ///
        /// # Returns
        ///
        /// An `AllowedIps` containing all possible IP networks.
        pub fn allow_all() -> Self {
            AllowedIps(vec![
                "0.0.0.0/0".parse().expect("Failed to parse ipv4 network"),
                "::0/0".parse().expect("Failed to parse ipv6 network"),
            ])
        }

        /// Constructs an `AllowedIps` from an iterator of string representations of IP networks.
        ///
        /// Each string should be a valid CIDR notation (e.g., "192.168.1.0/24").
        /// Ignores empty strings. Returns an error if any string is not a valid network or if it contains non-zero host bits.
        ///
        /// # Errors
        ///
        /// Returns `AllowedIpParseError::Parse` if parsing fails, or
        /// `AllowedIpParseError::NonZeroHostBits` if the network contains non-zero host bits.
        pub fn parse<I, S>(allowed_ips: I) -> Result<AllowedIps, AllowedIpParseError>
        where
            I: IntoIterator<Item = S>,
            S: AsRef<str>,
        {
            let mut networks = vec![];
            for s in allowed_ips {
                let s = s.as_ref().trim();
                if !s.is_empty() {
                    let net: IpNetwork = s.parse().map_err(AllowedIpParseError::Parse)?;
                    if net.network() != net.ip() {
                        return Err(AllowedIpParseError::NonZeroHostBits(net, net.network()));
                    }
                    networks.push(net);
                }
            }
            Ok(AllowedIps(networks))
        }

        /// Converts the `AllowedIps` into a `Constraint<AllowedIps>`.
        /// If the list of ip ranges is empty, it returns `Constraint::Any`, otherwise it returns `Constraint::Only(self)`.
        pub fn to_constraint(self) -> Constraint<AllowedIps> {
            if self.0.is_empty() {
                Constraint::Any
            } else {
                Constraint::Only(self)
            }
        }

        /// Resolves the allowed IPs to a `Vec<IpNetwork>`, adding the host IPv4 and IPv6 addresses if provided.
        pub fn resolve(
            self,
            host_ipv4: Option<Ipv4Addr>,
            host_ipv6: Option<Ipv6Addr>,
        ) -> Vec<IpNetwork> {
            let mut networks = self.0;
            if let Some(host_ipv6) = host_ipv6 {
                networks.push(IpNetwork::V6(host_ipv6.into()));
            }
            if let Some(host_ipv4) = host_ipv4 {
                networks.push(IpNetwork::V4(host_ipv4.into()));
            }
            log::trace!("Resolved allowed IPs: {networks:?}");
            networks
        }
    }

    /// Resolves the allowed IPs from a `Constraint<AllowedIps>`, adding the host IPv4 and IPv6 addresses if provided.
    /// If the constraint is `Constraint::Any` or `Constraint::Only` with an empty list, it allows all IPs.
    /// Returns a vector of `IpNetwork` containing the resolved allowed IPs.
    pub fn resolve_from_constraint(
        allowed_ips: &Constraint<AllowedIps>,
        host_ipv4: Option<Ipv4Addr>,
        host_ipv6: Option<Ipv6Addr>,
    ) -> Vec<IpNetwork> {
        match allowed_ips {
            Constraint::Any => AllowedIps::allow_all(),
            Constraint::Only(ips) if ips.0.is_empty() => AllowedIps::allow_all(),
            Constraint::Only(ips) => ips.clone(),
        }
        .resolve(host_ipv4, host_ipv6)
    }
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
        if let Constraint::Only(ip_version) = self.constraints.ip_version {
            write!(f, ", {ip_version},")?;
        }
        if self.constraints.multihop() {
            let location = self.constraints.entry_location.as_ref().map(|location| {
                LocationConstraintFormatter {
                    constraint: location,
                    custom_lists: self.custom_lists,
                }
            });
            write!(f, ", multihop entry {location}")?;
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
pub struct MissingCustomProxy(());

/// Specifies a specific endpoint or [`BridgeConstraints`] to use when `mullvad-daemon` selects a
/// bridge server.
#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BridgeSettings {
    pub bridge_type: BridgeType,
    pub custom: Option<CustomProxy>,
}

pub enum ResolvedBridgeSettings<'a> {
    Normal,
    Custom(&'a CustomProxy),
}

impl BridgeSettings {
    pub fn resolve(&self) -> Result<ResolvedBridgeSettings<'_>, MissingCustomProxy> {
        match (self.bridge_type, &self.custom) {
            (BridgeType::Normal, _) => Ok(ResolvedBridgeSettings::Normal),
            (BridgeType::Custom, Some(custom)) => Ok(ResolvedBridgeSettings::Custom(custom)),
            (BridgeType::Custom, None) => Err(MissingCustomProxy(())),
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
    WireguardPort,
    #[cfg_attr(feature = "clap", clap(name = "udp2tcp"))]
    Udp2Tcp,
    Shadowsocks,
    Quic,
    Lwo,
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
            SelectedObfuscation::Quic => "quic".fmt(f),
            SelectedObfuscation::Lwo => "lwo".fmt(f),
            SelectedObfuscation::WireguardPort => "wireguard port".fmt(f),
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

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, Intersection)]
#[serde(rename_all = "snake_case")]
pub struct WireguardPortSettings {
    port: Constraint<u16>,
}

impl WireguardPortSettings {
    pub const fn get(&self) -> Constraint<u16> {
        self.port
    }
}

impl From<Constraint<u16>> for WireguardPortSettings {
    fn from(port: Constraint<u16>) -> Self {
        Self { port }
    }
}

impl From<Option<u16>> for WireguardPortSettings {
    fn from(port: Option<u16>) -> Self {
        Self::from(Constraint::from(port))
    }
}

impl From<u16> for WireguardPortSettings {
    fn from(port: u16) -> Self {
        Self::from(Constraint::Only(port))
    }
}

impl fmt::Display for WireguardPortSettings {
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
    pub wireguard_port: WireguardPortSettings,
}

// pub struct BridgeConstraintsFormatter<'a> {
//     pub custom_lists: &'a CustomListsSettings,
// }

// impl fmt::Display for BridgeConstraintsFormatter<'_> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self.constraints.location {
//             Constraint::Any => write!(f, "any location")?,
//             Constraint::Only(ref constraint) => write!(
//                 f,
//                 "{}",
//                 LocationConstraintFormatter {
//                     constraint,
//                     custom_lists: self.custom_lists,
//                 }
//             )?,
//         }
//         write!(f, " using ")?;
//         match self.constraints.providers {
//             Constraint::Any => write!(f, "any provider")?,
//             Constraint::Only(ref constraint) => write!(f, "{constraint}")?,
//         }
//         match self.constraints.ownership {
//             Constraint::Any => Ok(()),
//             Constraint::Only(ref constraint) => {
//                 write!(f, " and {constraint}")
//             }
//         }
//     }
// }

/// Setting indicating whether to connect to a bridge server, or to handle it automatically.
// TODO: remove
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

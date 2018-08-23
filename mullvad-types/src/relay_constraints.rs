use location::{CityCode, CountryCode, Hostname};
use CustomTunnelEndpoint;

use std::fmt;

use talpid_types::net::{OpenVpnEndpointData, TransportProtocol, WireguardEndpointData};


pub trait Match<T> {
    fn matches(&self, other: &T) -> bool;
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Constraint<T: fmt::Debug + Clone + Eq + PartialEq> {
    Any,
    Only(T),
}

impl<T: fmt::Debug + Clone + Eq + PartialEq> Constraint<T> {
    pub fn unwrap_or(self, other: T) -> T {
        match self {
            Constraint::Any => other,
            Constraint::Only(value) => value,
        }
    }
}

impl<T: fmt::Debug + Clone + Eq + PartialEq> Default for Constraint<T> {
    fn default() -> Self {
        Constraint::Any
    }
}

impl<T: Copy + fmt::Debug + Clone + Eq + PartialEq> Copy for Constraint<T> {}

impl<T: fmt::Debug + Clone + Eq + PartialEq> Match<T> for Constraint<T> {
    fn matches(&self, other: &T) -> bool {
        match *self {
            Constraint::Any => true,
            Constraint::Only(ref value) => value == other,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelaySettings {
    CustomTunnelEndpoint(CustomTunnelEndpoint),
    Normal(RelayConstraints),
}

impl Default for RelaySettings {
    fn default() -> Self {
        RelaySettings::Normal(RelayConstraints::default())
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
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RelayConstraints {
    pub location: Constraint<LocationConstraint>,
    pub tunnel: Constraint<TunnelConstraints>,
}

impl RelayConstraints {
    pub fn merge(&self, update: RelayConstraintsUpdate) -> Self {
        RelayConstraints {
            location: update.location.unwrap_or_else(|| self.location.clone()),
            tunnel: update.tunnel.unwrap_or_else(|| self.tunnel.clone()),
        }
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationConstraint {
    /// A country is represented by its two letter country code.
    Country(CountryCode),
    /// A city is composed of a country code and a city code.
    City(CountryCode, CityCode),
    /// An single hostname in a given city.
    Hostname(CountryCode, CityCode, Hostname),
}


#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum TunnelConstraints {
    #[serde(rename = "openvpn")]
    OpenVpn(OpenVpnConstraints),
    #[serde(rename = "wireguard")]
    Wireguard(WireguardConstraints),
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

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct OpenVpnConstraints {
    pub port: Constraint<u16>,
    pub protocol: Constraint<TransportProtocol>,
}

impl Match<OpenVpnEndpointData> for OpenVpnConstraints {
    fn matches(&self, endpoint: &OpenVpnEndpointData) -> bool {
        self.port.matches(&endpoint.port) && self.protocol.matches(&endpoint.protocol)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct WireguardConstraints {
    pub port: Constraint<u16>,
}

impl Match<WireguardEndpointData> for WireguardConstraints {
    fn matches(&self, endpoint: &WireguardEndpointData) -> bool {
        self.port.matches(&endpoint.port)
    }
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelaySettingsUpdate {
    CustomTunnelEndpoint(CustomTunnelEndpoint),
    Normal(RelayConstraintsUpdate),
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RelayConstraintsUpdate {
    pub location: Option<Constraint<LocationConstraint>>,
    pub tunnel: Option<Constraint<TunnelConstraints>>,
}

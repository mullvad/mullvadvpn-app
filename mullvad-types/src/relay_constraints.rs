use crate::{
    location::{CityCode, CountryCode, Hostname},
    relay_list::{OpenVpnEndpointData, WireguardEndpointData},
    CustomTunnelEndpoint,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use talpid_types::net::TransportProtocol;


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

    pub fn or(self, other: Constraint<T>) -> Constraint<T> {
        match self {
            Constraint::Any => other,
            Constraint::Only(value) => Constraint::Only(value),
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

impl fmt::Display for RelayConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.tunnel {
            Constraint::Any => write!(f, "any relay")?,
            Constraint::Only(ref tunnel_constraint) => tunnel_constraint.fmt(f)?,
        }
        write!(f, " in ")?;
        match self.location {
            Constraint::Any => write!(f, "any location"),
            Constraint::Only(ref location_constraint) => location_constraint.fmt(f),
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

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
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
        self.port.matches(&endpoint.port) && self.protocol.matches(&endpoint.protocol)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct WireguardConstraints {
    pub port: Constraint<u16>,
}

impl fmt::Display for WireguardConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.port {
            Constraint::Any => write!(f, "any port"),
            Constraint::Only(port) => write!(f, "port {}", port),
        }
    }
}

impl Match<WireguardEndpointData> for WireguardConstraints {
    fn matches(&self, endpoint: &WireguardEndpointData) -> bool {
        match self.port {
            Constraint::Any => true,
            Constraint::Only(port) => endpoint
                .port_ranges
                .iter()
                .any(|range| (port >= range.0 && port <= range.1)),
        }
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

//! Common types for pre- and post migration.

use std::collections::{BTreeSet, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod v17;
pub mod v18;

#[derive(Debug, Clone, Default, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Constraint<T> {
    #[default]
    Any,
    Only(T),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationConstraint {
    Location(GeographicLocationConstraint),
    CustomList { list_id: String },
}

impl Default for LocationConstraint {
    fn default() -> Self {
        Self::Location(Default::default())
    }
}

impl From<LocationConstraint> for mullvad_types::relay_constraints::LocationConstraint {
    fn from(constraint: LocationConstraint) -> Self {
        use mullvad_types::relay_constraints::LocationConstraint::*;
        match constraint {
            LocationConstraint::Location(geographic_location_constraint) => {
                Location(geographic_location_constraint.into())
            }
            LocationConstraint::CustomList { list_id } => CustomList {
                list_id: list_id.parse().expect("TODO: Do not unwrap"),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeographicLocationConstraint {
    /// A country is represented by its two letter country code.
    Country(String),
    /// A city is composed of a country code and a city code.
    City(String, String),
    /// An single hostname in a given city.
    Hostname(String, String, String),
}

impl Default for GeographicLocationConstraint {
    fn default() -> Self {
        Self::Country("se".to_string())
    }
}

impl From<GeographicLocationConstraint>
    for mullvad_types::relay_constraints::GeographicLocationConstraint
{
    fn from(constraint: GeographicLocationConstraint) -> Self {
        use mullvad_types::relay_constraints::GeographicLocationConstraint::*;
        match constraint {
            GeographicLocationConstraint::Country(country) => Country(country),
            GeographicLocationConstraint::City(country, city) => City(country, city),
            GeographicLocationConstraint::Hostname(country, city, hostname) => {
                Hostname(country, city, hostname)
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomListsSettings {
    custom_lists: Vec<CustomList>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomList {
    id: String,
    pub name: String,
    pub locations: BTreeSet<GeographicLocationConstraint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ownership {
    MullvadOwned,
    Rented,
}

impl From<Ownership> for mullvad_types::relay_constraints::Ownership {
    fn from(value: Ownership) -> Self {
        match value {
            Ownership::MullvadOwned => Self::MullvadOwned,
            Ownership::Rented => Self::Rented,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Providers {
    providers: HashSet<String>,
}

impl TryFrom<Providers> for mullvad_types::relay_constraints::Providers {
    type Error = ();

    fn try_from(value: Providers) -> Result<Self, Self::Error> {
        match Self::new(value.providers) {
            Ok(providers) => Ok(providers),
            Err(no_providers) => Err(()),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IpVersion {
    #[default]
    V4,
    V6,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AllowedIps(Vec<String>);

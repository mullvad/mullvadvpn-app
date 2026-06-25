//! Common types for pre- and post migration.
//!
//! Note that these types mirror pre-existing types from talpid-* and mullvad-* crates. To hide
//! these types from rust-analyzer's symbol resolution they are prefixed with `__`.

use std::collections::{BTreeSet, HashSet};

use serde::{Deserialize, Serialize};

pub mod v17;
pub mod v18;

#[derive(Debug, Clone, Default, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum __Constraint<T> {
    #[default]
    Any,
    Only(T),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum __LocationConstraint {
    Location(__GeographicLocationConstraint),
    CustomList { list_id: String },
}

impl Default for __LocationConstraint {
    fn default() -> Self {
        Self::Location(Default::default())
    }
}

impl From<__LocationConstraint> for mullvad_types::relay_constraints::LocationConstraint {
    fn from(constraint: __LocationConstraint) -> Self {
        use mullvad_types::relay_constraints::LocationConstraint::*;
        match constraint {
            __LocationConstraint::Location(geographic_location_constraint) => {
                Location(geographic_location_constraint.into())
            }
            __LocationConstraint::CustomList { list_id } => CustomList {
                list_id: list_id.parse().expect("TODO: Do not unwrap"),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum __GeographicLocationConstraint {
    /// A country is represented by its two letter country code.
    Country(String),
    /// A city is composed of a country code and a city code.
    City(String, String),
    /// An single hostname in a given city.
    Hostname(String, String, String),
}

impl Default for __GeographicLocationConstraint {
    fn default() -> Self {
        Self::Country("se".to_string())
    }
}

impl From<__GeographicLocationConstraint>
    for mullvad_types::relay_constraints::GeographicLocationConstraint
{
    fn from(constraint: __GeographicLocationConstraint) -> Self {
        use mullvad_types::relay_constraints::GeographicLocationConstraint::*;
        match constraint {
            __GeographicLocationConstraint::Country(country) => Country(country),
            __GeographicLocationConstraint::City(country, city) => City(country, city),
            __GeographicLocationConstraint::Hostname(country, city, hostname) => {
                Hostname(country, city, hostname)
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct __CustomListsSettings {
    custom_lists: Vec<__CustomList>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct __CustomList {
    id: String,
    pub name: String,
    pub locations: BTreeSet<__GeographicLocationConstraint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum __Ownership {
    MullvadOwned,
    Rented,
}

impl From<__Ownership> for mullvad_types::relay_constraints::Ownership {
    fn from(value: __Ownership) -> Self {
        match value {
            __Ownership::MullvadOwned => Self::MullvadOwned,
            __Ownership::Rented => Self::Rented,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct __Providers {
    providers: HashSet<String>,
}

impl TryFrom<__Providers> for mullvad_types::relay_constraints::Providers {
    type Error = ();

    fn try_from(value: __Providers) -> Result<Self, Self::Error> {
        match Self::new(value.providers) {
            Ok(providers) => Ok(providers),
            Err(_no_providers) => Err(()),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum __IpVersion {
    #[default]
    V4,
    V6,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct __AllowedIps(Vec<String>);

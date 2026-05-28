//! Common types for pre- and post migration.

use std::collections::HashSet;

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
    Location(Value),
    /// If the entry location is a custom list, just re-use it.
    CustomList(Value),
}

impl Default for LocationConstraint {
    fn default() -> Self {
        Self::Location(Default::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ownership {
    MullvadOwned,
    Rented,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Providers {
    providers: HashSet<String>,
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

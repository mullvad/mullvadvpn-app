//! Common types for pre- and post migration.
//!
//! Note that these types mirror pre-existing types from talpid-* and mullvad-* crates. To hide
//! these types from rust-analyzer's symbol resolution they are prefixed with `__`.

use std::collections::HashSet;

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
pub enum LocationConstraint {
    Location(Value),
    /// If the entry location is a custom list, just re-use it.
    CustomList(Value),
}

impl Default for __LocationConstraint {
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

impl Default for __GeographicLocationConstraint {
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
pub struct __Providers {
    providers: HashSet<String>,
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

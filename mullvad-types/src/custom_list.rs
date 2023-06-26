use crate::relay_constraints::{Constraint, GeographicLocationConstraint};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct Id(pub uuid::Uuid);

impl TryFrom<&str> for Id {
    type Error = ();
    fn try_from(string: &str) -> Result<Self, Self::Error> {
        let uuid = uuid::Uuid::from_str(string).map_err(|_| ())?;
        Ok(Id(uuid))
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str(&self.0.to_string())
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct CustomListsSettings {
    pub custom_lists: HashMap<Id, CustomList>,
}

impl CustomListsSettings {
    pub fn get_custom_list_with_name(&self, name: &String) -> Option<&CustomList> {
        self.custom_lists
            .values()
            .find(|custom_list| &custom_list.name == name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CustomListLocationUpdate {
    Add {
        name: String,
        location: Constraint<GeographicLocationConstraint>,
    },
    Remove {
        name: String,
        location: Constraint<GeographicLocationConstraint>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct CustomList {
    pub id: Id,
    pub name: String,
    pub locations: Vec<GeographicLocationConstraint>,
}

impl CustomList {
    pub fn new(name: String) -> Self {
        CustomList {
            id: Id(uuid::Uuid::new_v4()),
            name,
            locations: Vec::new(),
        }
    }
}

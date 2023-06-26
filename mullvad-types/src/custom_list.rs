use crate::relay_constraints::{Constraint, GeographicLocationConstraint};
#[cfg(target_os = "android")]
use jnix::{FromJava, IntoJava};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct Id(pub String);

impl TryFrom<&str> for Id {
    type Error = ();
    fn try_from(string: &str) -> Result<Self, Self::Error> {
        Ok(Id(string.to_string()))
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str(&self.0.to_string())
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct CustomListsSettings {
    pub custom_lists: Vec<CustomList>,
}

impl CustomListsSettings {
    pub fn get_custom_list_with_name(&self, name: &String) -> Option<&CustomList> {
        self.custom_lists
            .iter()
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
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct CustomList {
    pub id: Id,
    pub name: String,
    pub locations: Vec<GeographicLocationConstraint>,
}

impl CustomList {
    pub fn new(name: String) -> Self {
        CustomList {
            id: Id(uuid::Uuid::new_v4().to_string()),
            name,
            locations: Vec::new(),
        }
    }
}

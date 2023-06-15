use serde::{Serialize, Deserialize};
use crate::relay_constraints::{Constraint, GeographicLocationConstraint};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

impl std::fmt::Display for Id {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str(&self.0)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomListsSettings {
    pub custom_lists: HashMap<Id, CustomList>,
}

impl CustomListsSettings {
    pub fn get_custom_list_with_name(&self, name: &String) -> Option<&CustomList> {
        self.custom_lists.values().find(|custom_list| &custom_list.name == name)
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
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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



use serde::{Serialize, Deserialize};
use crate::relay_constraints::{Constraint, LocationConstraint};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomListsSettings {
    pub custom_lists: HashMap<String, CustomList>,
    pub selected_list: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CustomListLocationUpdate {
    Add {
        name: String,
        location: Constraint<LocationConstraint>,
    },
    Remove {
        name: String,
        location: Constraint<LocationConstraint>,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomList {
    pub id: String,
    pub name: String,
    pub locations: Vec<Constraint<LocationConstraint>>,
}

impl CustomList {
    pub fn new(name: String) -> Self {
        CustomList {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            locations: Vec::new(),
        }
    }
}

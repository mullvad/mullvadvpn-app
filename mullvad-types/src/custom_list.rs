use serde::{Serialize, Deserialize};
use crate::relay_constraints::{Constraint, LocationConstraint};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomListsSettings {
    pub custom_lists: HashMap<String, CustomList>,
    pub selected_list_entry: Option<String>,
    pub selected_list_exit: Option<String>,
}

impl CustomListsSettings {
    pub fn get_selected_list_entry(&self) -> Option<&CustomList> {
        match &self.selected_list_entry {
            None => None,
            Some(selected_list) => {
                for list in self.custom_lists.values() {
                    if &list.id == selected_list {
                        return Some(list);
                    }
                }
                None
            }
        }
    }

    pub fn get_selected_list_exit(&self) -> Option<&CustomList> {
        match &self.selected_list_exit {
            None => None,
            Some(selected_list) => {
                for list in self.custom_lists.values() {
                    if &list.id == selected_list {
                        return Some(list);
                    }
                }
                None
            }
        }
    }
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
    pub locations: Vec<LocationConstraint>,
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



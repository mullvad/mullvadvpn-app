use crate::relay_constraints::GeographicLocationConstraint;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    ops::{Deref, DerefMut},
};

pub type Id = uuid::Uuid;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomListsSettings {
    custom_lists: Vec<CustomList>,
}

impl From<Vec<CustomList>> for CustomListsSettings {
    fn from(custom_lists: Vec<CustomList>) -> Self {
        Self { custom_lists }
    }
}

impl CustomListsSettings {
    pub fn add(&mut self, list: CustomList) {
        self.custom_lists.push(list);
    }

    pub fn remove(&mut self, index: usize) {
        self.custom_lists.remove(index);
    }
}

impl IntoIterator for CustomListsSettings {
    type Item = CustomList;
    type IntoIter = <Vec<CustomList> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.custom_lists.into_iter()
    }
}

impl Deref for CustomListsSettings {
    type Target = [CustomList];

    fn deref(&self) -> &Self::Target {
        &self.custom_lists
    }
}

impl DerefMut for CustomListsSettings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom_lists
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomList {
    pub id: Id,
    pub name: String,
    pub locations: BTreeSet<GeographicLocationConstraint>,
}

impl CustomList {
    pub fn new(name: String) -> Self {
        CustomList {
            id: uuid::Uuid::new_v4(),
            name,
            locations: BTreeSet::new(),
        }
    }
}

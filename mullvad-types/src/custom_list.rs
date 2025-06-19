use crate::relay_constraints::GeographicLocationConstraint;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    ops::{Deref, DerefMut},
    str::FromStr,
};

const CUSTOM_LIST_NAME_MAX_SIZE: usize = 30;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Custom list name too long")]
    NameTooLong,
    #[error("Custom list with name already exists")]
    DuplicateName,
    #[error("Custom list not found")]
    ListNotFound,
    #[error("List with given ID already exists")]
    ListExists,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(uuid::Uuid);

impl Deref for Id {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Id {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for Id {
    type Err = <uuid::Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::from_str(s).map(Id)
    }
}

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
    pub fn add(&mut self, new_list: CustomList) -> Result<(), Error> {
        self.check_if_id_is_unique(&new_list)?;
        self.check_list_name_is_unique(&new_list)?;
        self.custom_lists.push(new_list);
        Ok(())
    }

    pub fn remove(&mut self, list_id: &Id) -> Result<(), Error> {
        let Some(list_index) = self.find_list_index(list_id) else {
            return Err(Error::ListNotFound);
        };
        self.custom_lists.remove(list_index);
        Ok(())
    }

    /// Remove all custom lists
    pub fn clear(&mut self) {
        self.custom_lists.clear();
    }

    pub fn update(&mut self, new_list: CustomList) -> Result<(), Error> {
        let list_index = self
            .find_list_index(&new_list.id)
            .ok_or(Error::ListNotFound)?;
        self.check_list_name_is_unique(&new_list)?;
        self.custom_lists[list_index] = new_list;
        Ok(())
    }

    fn check_list_name_is_unique(&self, new_list: &CustomList) -> Result<(), Error> {
        if self
            .custom_lists
            .iter()
            .any(|list| list.name == new_list.name && list.id != new_list.id)
        {
            return Err(Error::DuplicateName);
        }
        Ok(())
    }

    fn check_if_id_is_unique(&self, new_list: &CustomList) -> Result<(), Error> {
        if self.custom_lists.iter().any(|list| list.id == new_list.id) {
            return Err(Error::ListExists);
        }
        Ok(())
    }

    fn find_list_index(&self, list_id: &Id) -> Option<usize> {
        self.custom_lists
            .iter()
            .enumerate()
            .find(|(_idx, list)| list.id == *list_id)
            .map(|(idx, _list)| idx)
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
    id: Id,
    pub name: String,
    pub locations: BTreeSet<GeographicLocationConstraint>,
}

impl CustomList {
    /// Create a new [CustomList] with a given name. This function will check that the name
    /// is appropriate (see implementation for details) and generate a new unique [Id].
    pub fn new(name: String) -> Result<Self, Error> {
        if name.chars().count() > CUSTOM_LIST_NAME_MAX_SIZE {
            return Err(Error::NameTooLong);
        }

        let id = Id(uuid::Uuid::new_v4());
        let mut custom_list = Self::with_id(id);
        custom_list.name = name;

        Ok(custom_list)
    }

    /// Instantiate an empty custom list with a pre-existing [Id]. This is useful when
    /// serializing/deserializing, and most likely you want to use [CustomList::new] instead.
    pub fn with_id(id: Id) -> Self {
        Self {
            id,
            name: Default::default(),
            locations: Default::default(),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn append(&mut self, mut locations: BTreeSet<GeographicLocationConstraint>) {
        self.locations.append(&mut locations);
    }
}

use crate::relay_constraints::GeographicLocationConstraint;
#[cfg(target_os = "android")]
use jnix::{
    jni::objects::{AutoLocal, JObject, JString},
    FromJava, IntoJava, JnixEnv,
};
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

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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

#[cfg(target_os = "android")]
impl<'env, 'sub_env> FromJava<'env, JString<'sub_env>> for Id
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "Ljava/lang/String;";

    fn from_java(env: &JnixEnv<'env>, source: JString<'sub_env>) -> Self {
        let s = env
            .get_string(source)
            .expect("Failed to convert from Java String");
        Id::from_str(s.to_str().unwrap()).expect("invalid ID")
    }
}

#[cfg(target_os = "android")]
impl<'env, 'sub_env> FromJava<'env, JObject<'sub_env>> for Id
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "Ljava/lang/String;";

    fn from_java(env: &JnixEnv<'env>, source: JObject<'sub_env>) -> Self {
        Id::from_java(env, JString::from(source))
    }
}

#[cfg(target_os = "android")]
impl<'borrow, 'env: 'borrow> IntoJava<'borrow, 'env> for Id {
    const JNI_SIGNATURE: &'static str = "Ljava/lang/String;";

    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let s = self.to_string();
        let jstring = env.new_string(&s).expect("Failed to create Java String");

        env.auto_local(jstring)
    }
}

#[cfg_attr(target_os = "android", derive(IntoJava, FromJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
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
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct CustomList {
    pub id: Id,
    pub name: String,
    #[cfg_attr(
        target_os = "android",
        jnix(
            map = "|locations| locations.into_iter().collect::<Vec<GeographicLocationConstraint>>()"
        )
    )]
    pub locations: BTreeSet<GeographicLocationConstraint>,
}

impl CustomList {
    pub fn new(name: String) -> Result<Self, Error> {
        if name.chars().count() > CUSTOM_LIST_NAME_MAX_SIZE {
            return Err(Error::NameTooLong);
        }

        Ok(CustomList {
            id: Id(uuid::Uuid::new_v4()),
            name,
            locations: BTreeSet::new(),
        })
    }
}

#[cfg(target_os = "android")]
impl<'env, 'sub_env> FromJava<'env, JObject<'sub_env>> for CustomList
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "Lnet/mullvad/mullvadvpn/model/CustomList;";

    fn from_java(env: &JnixEnv<'env>, source: JObject<'sub_env>) -> Self {
        let object_id = env
            .call_method(source, "component1", "()Ljava/lang/String;", &[])
            .expect("missing CustomList.id")
            .l()
            .expect("CustomList.id did not return an object");
        let id = Id::from_str(&String::from_java(env, object_id)).expect("invalid ID");

        let object_name = env
            .call_method(source, "component2", "()Ljava/lang/String;", &[])
            .expect("missing CustomList.name")
            .l()
            .expect("CustomList.name did not return an object");
        let name = String::from_java(env, object_name);

        let object_locations = env
            .call_method(source, "component3", "()Ljava/util/ArrayList;", &[])
            .expect("missing CustomList.locations")
            .l()
            .expect("CustomList.locations did not return an object");
        let locations = BTreeSet::from_iter(Vec::from_java(env, object_locations));

        CustomList {
            id,
            name,
            locations,
        }
    }
}

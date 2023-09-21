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
            id: Id(uuid::Uuid::new_v4()),
            name,
            locations: BTreeSet::new(),
        }
    }
}

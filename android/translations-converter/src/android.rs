use serde::{Deserialize, Serialize};

/// Contents of an Android string resources file.
///
/// This type can be created directly deserializing the `strings.xml` file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StringResources {
    #[serde(rename = "string")]
    entries: Vec<StringResource>,
}

/// An entry in an Android string resources file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StringResource {
    /// The string resource ID.
    pub name: String,

    /// The string value.
    #[serde(rename = "$value")]
    pub value: String,
}

impl IntoIterator for StringResources {
    type Item = StringResource;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

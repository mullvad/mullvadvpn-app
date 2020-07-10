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
    name: String,
    #[serde(rename = "$value")]
    value: String,
}

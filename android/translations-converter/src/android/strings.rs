use super::string_value::StringValue;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
};

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

    /// If the string should be translated or not.
    #[serde(default = "default_translatable")]
    pub translatable: bool,

    /// The string value.
    #[serde(rename = "$value")]
    pub value: StringValue,
}

impl StringResources {
    /// Create an empty list of Android string resources.
    pub fn new() -> Self {
        StringResources {
            entries: Vec::new(),
        }
    }

    /// Normalize the strings into a common format.
    ///
    /// Allows the string values to be compared to the gettext messages.
    pub fn normalize(&mut self) {
        for entry in &mut self.entries {
            entry.normalize();
        }
    }

    /// Sorts the entries alphabetically based on their IDs.
    pub fn sort(&mut self) {
        self.entries
            .sort_by(|left, right| left.name.cmp(&right.name));
    }
}

impl Deref for StringResources {
    type Target = Vec<StringResource>;

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl DerefMut for StringResources {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

impl IntoIterator for StringResources {
    type Item = StringResource;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl StringResource {
    /// Create a new Android string resource entry.
    ///
    /// The name is the resource ID, and the value will be properly escaped.
    pub fn new(name: String, value: &str) -> Self {
        StringResource {
            name,
            translatable: true,
            value: StringValue::from(value),
        }
    }

    /// Normalize the string value into a common format.
    ///
    /// Makes it possible to compare the Android strings with the gettext messages.
    pub fn normalize(&mut self) {
        self.value.normalize();
    }
}

fn default_translatable() -> bool {
    true
}

// Unfortunately, direct serialization to XML isn't working correctly.
impl Display for StringResources {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, r#"<?xml version="1.0" encoding="utf-8"?>"#)?;
        writeln!(formatter, "<resources>")?;

        for string in &self.entries {
            writeln!(formatter, "    {}", string)?;
        }

        writeln!(formatter, "</resources>")
    }
}

impl Display for StringResource {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        if self.translatable {
            write!(
                formatter,
                r#"<string name="{}">{}</string>"#,
                self.name, self.value
            )
        } else {
            write!(
                formatter,
                r#"<string name="{}" translatable="false">{}</string>"#,
                self.name, self.value
            )
        }
    }
}

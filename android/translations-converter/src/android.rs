use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
};

lazy_static! {
    static ref LINE_BREAKS: Regex = Regex::new(r"\s*\n\s*").unwrap();
    static ref APOSTROPHES: Regex = Regex::new(r"\\'").unwrap();
    static ref PARAMETERS: Regex = Regex::new(r"%[0-9]*\$").unwrap();
}

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
    pub value: String,
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
        let value_with_parameters = value
            .replace(r"\", r"\\")
            .replace("\"", "\\\"")
            .replace(r"'", r"\'");

        let mut parts = value_with_parameters.split("%");
        let mut value = parts.next().unwrap().to_owned();

        for (index, part) in parts.enumerate() {
            value.push_str(&format!("%{}$", index + 1));
            value.push_str(part);
        }

        StringResource {
            name,
            translatable: true,
            value,
        }
    }

    /// Normalize the string value into a common format.
    ///
    /// Makes it possible to compare the Android strings with the gettext messages.
    pub fn normalize(&mut self) {
        // Collapse line breaks present in the XML file
        let value = LINE_BREAKS.replace_all(&self.value, " ");
        // Unescape apostrophes
        let value = APOSTROPHES.replace_all(&value, "'");
        // Mark where parameters are positioned, removing the parameter index
        let value = PARAMETERS.replace_all(&value, "%");

        self.value = value.into_owned();
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

/// Contents of an Android plurals resources file.
///
/// This type can be created directly deserializing the `plurals.xml` file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluralResources {
    #[serde(rename = "plurals")]
    entries: Vec<PluralResource>,
}

/// An entry in an Android plurals resources file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluralResource {
    /// The plural resource ID.
    pub name: String,

    /// The items of the plural resource, one for each quantity variant.
    #[serde(rename = "item")]
    pub items: Vec<PluralVariant>,
}

/// A string resource for a specific quantity.
///
/// This is part of a plural resource.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluralVariant {
    /// The quantity for this variant to be used.
    pub quantity: PluralQuantity,

    /// The string value
    #[serde(rename = "$value")]
    pub string: String,
}

/// A valid quantity for a plural variant.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PluralQuantity {
    Zero,
    One,
    Few,
    Many,
    Other,
}

impl Deref for PluralResources {
    type Target = Vec<PluralResource>;

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl DerefMut for PluralResources {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

impl IntoIterator for PluralResources {
    type Item = PluralResource;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

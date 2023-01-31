use super::string_value::StringValue;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
};

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
    #[serde(rename = "@name")]
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
    #[serde(rename = "@quantity")]
    pub quantity: PluralQuantity,

    /// The string value
    #[serde(rename = "$value")]
    pub string: StringValue,
}

/// A valid quantity for a plural variant.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

impl PluralResources {
    /// Create an empty list of plural resources.
    pub fn new() -> Self {
        PluralResources {
            entries: Vec::new(),
        }
    }
}

impl PluralResource {
    /// Create a plural resource representation.
    ///
    /// The resource has a name, used as the identifier, and a list of items. Each item contains
    /// the message and the quantity it should be used for.
    pub fn new(name: String, values: impl Iterator<Item = (PluralQuantity, String)>) -> Self {
        let items = values
            .map(|(quantity, string)| PluralVariant {
                quantity,
                string: StringValue::from_unescaped(&string),
            })
            .collect();

        PluralResource { name, items }
    }
}

impl Display for PluralResources {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, r#"<?xml version="1.0" encoding="utf-8"?>"#)?;
        writeln!(formatter, "<resources>")?;

        for entry in &self.entries {
            write!(formatter, "{entry}")?;
        }

        writeln!(formatter, "</resources>")
    }
}

impl Display for PluralResource {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, r#"    <plurals name="{}">"#, self.name)?;

        for item in &self.items {
            writeln!(formatter, "        {item}")?;
        }

        writeln!(formatter, "    </plurals>")
    }
}

impl Display for PluralVariant {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            r#"<item quantity="{}">{}</item>"#,
            self.quantity, self.string
        )
    }
}

impl Display for PluralQuantity {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let quantity = match self {
            PluralQuantity::Zero => "zero",
            PluralQuantity::One => "one",
            PluralQuantity::Few => "few",
            PluralQuantity::Many => "many",
            PluralQuantity::Other => "other",
        };

        write!(formatter, "{quantity}")
    }
}

use super::{string_value::StringValue, tag_name_to_string};
use derive_more::{Display, Error, From};
use std::{
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
    str::FromStr,
};

/// Contents of an Android plurals resources file.
///
/// This type can be created directly deserializing the `plurals.xml` file.
#[derive(Clone, Debug)]
pub struct PluralResources {
    entries: Vec<PluralResource>,
}

/// An entry in an Android plurals resources file.
#[derive(Clone, Debug)]
pub struct PluralResource {
    /// The plural resource ID.
    pub name: String,

    /// The items of the plural resource, one for each quantity variant.
    pub items: Vec<PluralVariant>,
}

/// A string resource for a specific quantity.
///
/// This is part of a plural resource.
#[derive(Clone, Debug)]
pub struct PluralVariant {
    /// The quantity for this variant to be used.
    pub quantity: PluralQuantity,

    /// The string value
    pub string: StringValue,
}

/// A valid quantity for a plural variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

impl FromStr for PluralResources {
    type Err = ParseError;

    /// Parse a plural resources XML document string.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let document = roxmltree::Document::parse(input)?;
        let root_node = document.root_element();
        let root_node_name = root_node.tag_name();

        // Ensure the root element has the correct name (`resources`, without a namespace)
        if root_node_name.name() != "resources" || root_node_name.namespace().is_some() {
            return Err(ParseError::UnexpectedRootNode(tag_name_to_string(
                root_node_name,
            )));
        }

        // Parse each entry from each plurals node
        let entries = root_node
            .children()
            .filter(|node| node.is_element())
            .map(|element| PluralResource::from_xml_node(&element))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PluralResources { entries })
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

    /// Parse an Android plural resource from an XML node.
    pub fn from_xml_node(node: &roxmltree::Node<'_, '_>) -> Result<Self, ParseError> {
        let tag_name = node.tag_name();

        // Ensure the element name is `plurals` without a namespace
        if tag_name.name() != "plurals" || tag_name.namespace().is_some() {
            return Err(ParseError::UnexpectedNode {
                expected: "plurals",
                found: tag_name_to_string(tag_name),
            });
        }

        // Extract the name attribute
        let name = node
            .attribute("name")
            .ok_or_else(|| {
                ParseError::MissingName(node.document().text_pos_at(node.range().start))
            })?
            .to_owned();

        // Parse the plural variants
        let items = node
            .children()
            .filter(|node| node.is_element())
            .map(|element| PluralVariant::from_xml_node(&element))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PluralResource { name, items })
    }
}

impl PluralVariant {
    /// Parse an Android plural variant from an XML node.
    pub fn from_xml_node(node: &roxmltree::Node<'_, '_>) -> Result<Self, ParseError> {
        let tag_name = node.tag_name();

        // Ensure the element name is `item` without a namespace
        if tag_name.name() != "item" || tag_name.namespace().is_some() {
            return Err(ParseError::UnexpectedNode {
                expected: "item",
                found: tag_name_to_string(tag_name),
            });
        }

        // Extract the name attribute
        let quantity = node
            .attribute("quantity")
            .ok_or_else(|| {
                ParseError::MissingQuantity(node.document().text_pos_at(node.range().start))
            })?
            .parse()?;

        // Build the string value from the node's children
        let string = StringValue::from_string_xml_node(node);

        Ok(PluralVariant { quantity, string })
    }
}

impl FromStr for PluralQuantity {
    type Err = ParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "zero" => Ok(PluralQuantity::Zero),
            "one" => Ok(PluralQuantity::One),
            "few" => Ok(PluralQuantity::Few),
            "many" => Ok(PluralQuantity::Many),
            "other" => Ok(PluralQuantity::Other),
            unknown => Err(ParseError::InvalidPluralQuantity(unknown.to_owned())),
        }
    }
}

impl Display for PluralResources {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, r#"<?xml version="1.0" encoding="utf-8"?>"#)?;
        writeln!(formatter, "<resources>")?;

        for entry in &self.entries {
            write!(formatter, "{}", entry)?;
        }

        writeln!(formatter, "</resources>")
    }
}

impl Display for PluralResource {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, r#"    <plurals name="{}">"#, self.name)?;

        for item in &self.items {
            writeln!(formatter, "        {}", item)?;
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

        write!(formatter, "{}", quantity)
    }
}

/// Failure to parse a plural resources XML input
#[derive(Debug, Display, Error, From)]
pub enum ParseError {
    #[display(fmt = "Failed to parse XML in string resources input")]
    ParseXml(roxmltree::Error),

    #[display(fmt = "Expected a `resources` root node, found: {}", _0)]
    #[from(ignore)]
    UnexpectedRootNode(#[error(not(source))] String),

    #[display(fmt = "Expected a `{}` node, found: {}", expected, found)]
    #[from(ignore)]
    UnexpectedNode {
        expected: &'static str,
        found: String,
    },

    #[display(fmt = "Plurals node has no `name` attribute at {}", _0)]
    #[from(ignore)]
    MissingName(#[error(not(source))] roxmltree::TextPos),

    #[display(fmt = "Plural variant item node has no `quantity` attribute at {}", _0)]
    #[from(ignore)]
    MissingQuantity(#[error(not(source))] roxmltree::TextPos),

    #[display(fmt = "Invalid plural quantity value: {}", _0)]
    #[from(ignore)]
    InvalidPluralQuantity(#[error(not(source))] String),
}

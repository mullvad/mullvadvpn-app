use super::string_value::StringValue;
use derive_more::{Display, Error, From};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
    str::{FromStr, ParseBoolError},
};

/// Contents of an Android string resources file.
///
/// This type can be created directly parsing the `strings.xml` file.
#[derive(Clone, Debug, Eq, Deserialize, PartialEq, Serialize)]
pub struct StringResources {
    #[serde(rename = "string")]
    entries: Vec<StringResource>,
}

/// An entry in an Android string resources file.
#[derive(Clone, Debug, Eq, Deserialize, PartialEq, Serialize)]
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

    /// Sorts the entries alphabetically based on their IDs.
    pub fn sort(&mut self) {
        self.entries
            .sort_by(|left, right| left.name.cmp(&right.name));
    }
}

impl FromStr for StringResources {
    type Err = ParseError;

    /// Parse a string resources XML document string.
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

        // Parse each entry from each string node
        let entries = root_node
            .children()
            .filter(|node| node.is_element())
            .map(|element| StringResource::from_xml_node(&element))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(StringResources { entries })
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
            value: StringValue::from_unescaped(value),
        }
    }

    /// Parse an Android string resource entry from an XML node.
    pub fn from_xml_node(node: &roxmltree::Node<'_, '_>) -> Result<Self, ParseError> {
        let tag_name = node.tag_name();

        // Ensure the element name is `string` without a namespace
        if tag_name.name() != "string" || tag_name.namespace().is_some() {
            return Err(ParseError::UnexpectedNode(tag_name_to_string(tag_name)));
        }

        // Extract the name attribute
        let name = node
            .attribute("name")
            .ok_or_else(|| {
                ParseError::MissingName(node.document().text_pos_at(node.range().start))
            })?
            .to_owned();

        // Extract the optional translatable attribute
        let translatable = node
            .attribute("translatable")
            .map(bool::from_str)
            .unwrap_or(Ok(true))?;

        // Build the string value from the node's children
        let value = StringValue::from_string_xml_node(node);

        Ok(StringResource {
            name,
            translatable,
            value,
        })
    }
}

fn tag_name_to_string(tag_name: roxmltree::ExpandedName<'_, '_>) -> String {
    match tag_name.namespace() {
        Some(namespace) => format!("{}:{}", namespace, tag_name.name()),
        None => tag_name.name().to_owned(),
    }
}

fn default_translatable() -> bool {
    true
}

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

/// Failure to parse a string resources XML input
#[derive(Debug, Display, Error, From)]
pub enum ParseError {
    #[display(fmt = "Failed to parse XML in string resources input")]
    ParseXml(roxmltree::Error),

    #[display(fmt = "Expected a `resources` root node, found: {}", _0)]
    #[from(ignore)]
    UnexpectedRootNode(#[error(not(source))] String),

    #[display(fmt = "Expected a `string` node, found: {}", _0)]
    #[from(ignore)]
    UnexpectedNode(#[error(not(source))] String),

    #[display(fmt = "String node has no `name` attribute at {}", _0)]
    MissingName(#[error(not(source))] roxmltree::TextPos),

    #[display(fmt = "Invalid translatable attribute value")]
    InvalidTranslatableAttribute(ParseBoolError),
}

#[cfg(test)]
mod tests {
    use super::{StringResource, StringResources, StringValue};

    #[test]
    fn deserialization() {
        let xml_input = r#"<resources>
            <string name="first">First string</string>
            <string name="second" translatable="false">Second string</string>
        </resources>"#;

        let mut expected = StringResources::new();

        expected.extend(vec![
            StringResource {
                name: "first".to_owned(),
                translatable: true,
                value: StringValue::from_unescaped("First string"),
            },
            StringResource {
                name: "second".to_owned(),
                translatable: false,
                value: StringValue::from_unescaped("Second string"),
            },
        ]);

        let deserialized: StringResources =
            serde_xml_rs::from_str(xml_input).expect("malformed XML in test input");

        assert_eq!(deserialized, expected);
    }

    #[test]
    fn deserialization_of_multi_line_strings() {
        let xml_input = r#"<resources>
            <string name="first">First string is
                split in two lines</string>
            <string
                name="second"
                translatable="false"
                >
                Second string is also split
                but it also has some weird whitespace
                inside the tags and some indentation
            </string>
        </resources>"#;

        let mut expected = StringResources::new();

        expected.extend(vec![
            StringResource {
                name: "first".to_owned(),
                translatable: true,
                value: StringValue::from_unescaped("First string is split in two lines"),
            },
            StringResource {
                name: "second".to_owned(),
                translatable: false,
                value: StringValue::from_unescaped(concat!(
                    "Second string is also split but it also has some weird whitespace inside the ",
                    "tags and some indentation",
                )),
            },
        ]);

        let deserialized: StringResources =
            serde_xml_rs::from_str(xml_input).expect("malformed XML in test input");

        assert_eq!(deserialized, expected);
    }
}

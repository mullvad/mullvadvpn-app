use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    fmt::{self, Display, Formatter, Write},
    ops::Deref,
};

/// An Android string value
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct StringValue(String);

impl StringValue {
    /// Create a `StringValue` from an unescaped string.
    ///
    /// The string will be properly escaped, and all parameters will have indices added to them if
    /// they don't have any. Indices are assigned sequentially starting from the previously
    /// specified index plus one, or starting from one if there aren't any previously specified
    /// indices.
    pub fn from_unescaped(string: &str) -> Self {
        let value_with_parameters = htmlize::escape_text(string)
            .replace('\\', r"\\")
            .replace('\"', "\\\"")
            .replace('\'', r"\'");

        let value_without_line_breaks = Self::collapse_line_breaks(value_with_parameters);
        let value = Self::ensure_parameters_are_indexed(value_without_line_breaks);

        StringValue(value)
    }

    /// The input XML file might have line breaks inside the string, and they should be collapsed
    /// into a single whitespace character.
    fn collapse_line_breaks(original: String) -> String {
        lazy_static! {
            static ref LINE_BREAKS: Regex = Regex::new(r"\s*\n\s*").unwrap();
        }

        LINE_BREAKS.replace_all(&original, " ").into_owned()
    }

    /// This helper method ensures parameters are in the form of `%4$d`, i.e., it will ensure that
    /// there is the `<number>$` part.
    ///
    /// A typical input would be something like `Things are %d, %3$s and %s`, and this method
    /// would update the string so that all parameters have indices: `Things are %1$d, %3$s and
    /// %4$s`.
    fn ensure_parameters_are_indexed(original: String) -> String {
        lazy_static! {
            static ref PARAMETER_INDEX: Regex = Regex::new(r"^(\d+)\$").unwrap();
        }

        let mut parts = original.split('%');
        let mut output = parts.next().unwrap().to_owned();
        let mut offset = 1;

        for (index, part) in parts.enumerate() {
            let index = index as isize;

            if let Some(captures) = PARAMETER_INDEX.captures(part) {
                // String already has a parameter index
                let specified_index: isize = captures
                    .get(1)
                    .expect("Regex has at least one capture group")
                    .as_str()
                    .parse()
                    .expect("First capture group should match an integer");

                // Update offset so that next parameters without index receive sequential values
                // starting after the specified index
                offset = specified_index - index;

                // Restore '%' removed during the split
                output.push('%');
            } else {
                // String doesn't have a parameter index, so it is added
                write!(&mut output, "%{}$", index + offset).expect("formatting failed");
            }

            output.push_str(part);
        }

        output
    }
}

impl Deref for StringValue {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl Display for StringValue {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for StringValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw_string = String::deserialize(deserializer)?;
        let string_with_collapsed_newlines = Self::collapse_line_breaks(raw_string);

        Ok(StringValue(string_with_collapsed_newlines))
    }
}

#[cfg(test)]
mod tests {
    use super::StringValue;

    #[test]
    fn android_escaping() {
        let input = StringValue::from_unescaped(concat!(
            r"A backslash \",
            r#""Inside double quotes""#,
            "'Inside single quotes'",
        ));

        let expected = concat!(
            r"A backslash \\",
            r#"\"Inside double quotes\""#,
            r"\'Inside single quotes\'",
        );

        assert_eq!(input.to_string(), expected);
    }

    #[test]
    fn newline_collapsing() {
        let input = StringValue::from_unescaped(
            "This is
            a multi-line string		
            that should be  
            	collapsed into a single line",
        );

        let expected = "This is a multi-line string that should be collapsed into a single line";

        assert_eq!(input.to_string(), expected);
    }

    #[test]
    fn xml_escaping() {
        let input = StringValue::from_unescaped(concat!(
            "An ampersand: &",
            "<tag>A dummy fake XML tag</tag>",
        ));

        let expected = concat!(
            "An ampersand: &amp;",
            r"&lt;tag&gt;A dummy fake XML tag&lt;/tag&gt;",
        );

        assert_eq!(input.to_string(), expected);
    }

    #[test]
    fn doesnt_change_parameter_indices() {
        let original = "%1$d %3$s %9$s %6$d %7$d";

        let input = StringValue::from_unescaped(original);

        assert_eq!(input.to_string(), original);
    }

    #[test]
    fn adds_parameter_indices() {
        let input = StringValue::from_unescaped("%d %s %s %d");

        let expected = "%1$d %2$s %3$s %4$d";

        assert_eq!(input.to_string(), expected);
    }

    #[test]
    fn correctly_updates_generated_index_offset_based_on_existing_indices() {
        let input = StringValue::from_unescaped("%d %4$s %d %2$s %d");

        let expected = "%1$d %4$s %5$d %2$s %3$d";

        assert_eq!(input.to_string(), expected);
    }

    #[test]
    fn deserialization() {
        #[derive(serde::Deserialize)]
        pub struct Wrapper {
            #[serde(rename = "$value")]
            value: StringValue,
        }

        let serialized_input = r#"<root>A multi-line string value
            with \"quotes\" and  
            parameters %2$s %d %1$d</root>"#;

        let deserialized: Wrapper =
            quick_xml::de::from_str(serialized_input).expect("Mal-formed serialized input");

        let expected = StringValue(
            r#"A multi-line string value with \"quotes\" and parameters %2$s %d %1$d"#.to_owned(),
        );

        assert_eq!(deserialized.value, expected);
    }
}

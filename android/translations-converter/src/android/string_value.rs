use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

/// An Android string value
#[derive(Clone, Debug, Eq, Deserialize, Hash, PartialEq, Serialize)]
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
            .replace(r"\", r"\\")
            .replace("\"", "\\\"")
            .replace(r"'", r"\'");

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

        let mut parts = original.split("%");
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
                output.push_str(&format!("%{}$", index + offset));
            }

            output.push_str(part);
        }

        output
    }
}

impl StringValue {
    /// Clones the internal string value.
    pub fn to_string(&self) -> String {
        self.0.clone()
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

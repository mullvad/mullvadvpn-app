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
    /// The string will be properly escaped, and all parameters will have indices added to them.
    pub fn from_unescaped(string: &str) -> Self {
        let value_with_parameters = htmlize::escape_text(string)
            .replace(r"\", r"\\")
            .replace("\"", "\\\"")
            .replace(r"'", r"\'");

        let value = Self::ensure_parameters_are_indexed(value_with_parameters);

        StringValue(value)
    }

    fn ensure_parameters_are_indexed(original: String) -> String {
        let mut parts = original.split("%");
        let mut output = parts.next().unwrap().to_owned();

        for (index, part) in parts.enumerate() {
            output.push_str(&format!("%{}$", index + 1));
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

use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

/// An Android string value
#[derive(Clone, Debug, Eq, Deserialize, Hash, PartialEq, Serialize)]
pub struct StringValue(String);

impl From<&str> for StringValue {
    fn from(string: &str) -> Self {
        let value_with_parameters = htmlize::escape_text(string)
            .replace(r"\", r"\\")
            .replace("\"", "\\\"")
            .replace(r"'", r"\'");

        let mut parts = value_with_parameters.split("%");
        let mut value = parts.next().unwrap().to_owned();

        for (index, part) in parts.enumerate() {
            value.push_str(&format!("%{}$", index + 1));
            value.push_str(part);
        }

        StringValue(value)
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

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

lazy_static! {
    static ref LINE_BREAKS: Regex = Regex::new(r"\s*\n\s*").unwrap();
    static ref APOSTROPHES: Regex = Regex::new(r"\\'").unwrap();
    static ref DOUBLE_QUOTES: Regex = Regex::new(r#"\\""#).unwrap();
    static ref PARAMETERS: Regex = Regex::new(r"%[0-9]*\$").unwrap();
}

/// An Android string value
///
/// Handles escaping the string when it is created but also allows normalizing it for comparing it
/// with gettext messages through a `normalize` method.
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
    /// Normalize the string value into a common format.
    ///
    /// Makes it possible to compare the Android strings with the gettext messages.
    pub fn normalize(&mut self) {
        // Collapse line breaks present in the XML file
        let value = LINE_BREAKS.replace_all(&self.0, " ");
        // Unescape apostrophes
        let value = APOSTROPHES.replace_all(&value, "'");
        // Unescape double quotes
        let value = DOUBLE_QUOTES.replace_all(&value, r#"""#);
        // Mark where parameters are positioned, removing the parameter index
        let value = PARAMETERS.replace_all(&value, "%");

        // Unescape XML characters
        self.0 = htmlize::unescape(value.as_bytes());
    }

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

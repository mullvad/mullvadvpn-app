use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

/// A message string in a gettext translation file.
#[derive(Clone, Debug)]
pub struct MsgString(String);

impl MsgString {
    /// Create a new empty `MsgString`.
    ///
    /// Equivalent to `MsgString::from_escaped("")`.
    pub fn empty() -> Self {
        MsgString(String::new())
    }

    /// Create a new `MsgString` from string without any escaped characters.
    ///
    /// This will ensure that the string has the double quotes characters properly escaped.
    pub fn from_unescaped(string: &str) -> Self {
        MsgString(string.replace(r#"""#, r#"\""#))
    }

    /// Create a new `MsgString` from string that already has proper escaping.
    pub fn from_escaped(string: impl Into<String>) -> Self {
        MsgString(string.into())
    }
}

impl Display for MsgString {
    /// Write the ID message string with proper escaping.
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Deref for MsgString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

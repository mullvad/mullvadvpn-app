use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

/// A message string in a gettext translation file.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MsgString(String);

impl From<String> for MsgString {
    fn from(string: String) -> Self {
        MsgString(string)
    }
}

impl From<&str> for MsgString {
    fn from(string: &str) -> Self {
        string.to_owned().into()
    }
}

impl Display for MsgString {
    /// Write the ID message string with proper escaping.
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        self.0.replace(r#"""#, r#"\""#).fmt(formatter)
    }
}

impl Deref for MsgString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

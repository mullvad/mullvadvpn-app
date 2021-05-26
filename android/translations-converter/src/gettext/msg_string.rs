use std::{
    fmt::{self, Display, Formatter},
    ops::{AddAssign, Deref},
};

/// A message string in a gettext translation file.
#[derive(Clone, Debug, Eq, PartialEq)]
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
    /// This will ensure that the string has common C escape sequences properly created for special
    /// characters. It will not attempt to escape non-ASCII characters and will just keep them as
    /// UTF-8 characters.
    pub fn from_unescaped(string: &str) -> Self {
        let string = string.replace(r"\", r"\\");
        let string = string.replace("\n", r"\n");
        let string = string.replace("\r", r"\r");
        let string = string.replace("\t", r"\t");
        let string = string.replace(r#"""#, r#"\""#);

        MsgString(string)
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

impl AsRef<MsgString> for MsgString {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<M> AddAssign<M> for MsgString
where
    M: AsRef<MsgString>,
{
    fn add_assign(&mut self, other: M) {
        self.0 += &other.as_ref().0;
    }
}

#[cfg(test)]
mod tests {
    use super::MsgString;

    #[test]
    fn empty_constructor() {
        let input = MsgString::empty();

        assert_eq!(input.to_string(), "");
    }

    #[test]
    fn escaping() {
        let input = MsgString::from_unescaped(concat!(
            r#""Inside double quotes""#,
            r"'Inside single quotes'",
            r"Back-slash character: \",
            "Whitespace characters: \n\r\t",
        ));

        let expected = concat!(
            r#"\"Inside double quotes\""#,
            "'Inside single quotes'",
            r"Back-slash character: \\",
            r"Whitespace characters: \n\r\t",
        );

        assert_eq!(input.to_string(), expected);
    }

    #[test]
    fn not_escaping() {
        let original = r#"\"Inside double quotes\""#;

        let input = MsgString::from_escaped(original);

        assert_eq!(input.to_string(), original);
    }
}

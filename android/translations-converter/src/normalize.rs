use lazy_static::lazy_static;
use regex::Regex;

pub trait Normalize {
    /// Normalize the string value into a common format.
    ///
    /// Makes it possible to compare different representations of translation messages.
    fn normalize(&self) -> String;
}

mod android {
    use super::*;
    use crate::android::StringValue;

    lazy_static! {
        static ref APOSTROPHES: Regex = Regex::new(r"\\'").unwrap();
        static ref DOUBLE_QUOTES: Regex = Regex::new(r#"\\""#).unwrap();
        static ref PARAMETERS: Regex = Regex::new(r"%[0-9]*\$").unwrap();
    }

    impl Normalize for StringValue {
        fn normalize(&self) -> String {
            // Unescape apostrophes
            let value = APOSTROPHES.replace_all(&*self, "'");
            // Unescape double quotes
            let value = DOUBLE_QUOTES.replace_all(&value, r#"""#);
            // Mark where parameters are positioned, removing the parameter index
            let value = PARAMETERS.replace_all(&value, "%");

            // Unescape XML characters
            htmlize::unescape(value.as_bytes())
        }
    }
}

mod gettext {
    use super::*;
    use crate::gettext::MsgString;

    lazy_static! {
        static ref APOSTROPHE_VARIATION: Regex = Regex::new("’").unwrap();
        static ref ESCAPED_DOUBLE_QUOTES: Regex = Regex::new(r#"\\""#).unwrap();
        static ref PARAMETERS: Regex = Regex::new(r"%\([^)]*\)").unwrap();
    }

    impl Normalize for MsgString {
        fn normalize(&self) -> String {
            // Use a single common apostrophe character
            let string = APOSTROPHE_VARIATION.replace_all(&*self, "'");
            // Mark where parameters are positioned, removing the parameter name
            let string = PARAMETERS.replace_all(&string, "%");
            // Remove escaped double-quotes
            let string = ESCAPED_DOUBLE_QUOTES.replace_all(&string, r#"""#);

            string.into_owned()
        }
    }
}

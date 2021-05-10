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
        static ref LINE_BREAKS: Regex = Regex::new(r"\s*\n\s*").unwrap();
        static ref APOSTROPHES: Regex = Regex::new(r"\\'").unwrap();
        static ref DOUBLE_QUOTES: Regex = Regex::new(r#"\\""#).unwrap();
        static ref PARAMETERS: Regex = Regex::new(r"%[0-9]*\$").unwrap();
    }

    impl Normalize for StringValue {
        fn normalize(&self) -> String {
            // Collapse line breaks present in the XML file
            let value = LINE_BREAKS.replace_all(&*self, " ");
            // Unescape apostrophes
            let value = APOSTROPHES.replace_all(&value, "'");
            // Unescape double quotes
            let value = DOUBLE_QUOTES.replace_all(&value, r#"""#);
            // Mark where parameters are positioned, removing the parameter index
            let value = PARAMETERS.replace_all(&value, "%");

            // Unescape XML characters
            htmlize::unescape(value.as_bytes())
        }
    }
}

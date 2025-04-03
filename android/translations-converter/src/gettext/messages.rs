use super::{msg_string::MsgString, parser::Parser, plural_form::PluralForm};
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    sync::LazyLock,
};

/// A parsed gettext messages file.
#[derive(Clone, Debug, Default)]
pub struct Messages {
    pub plural_form: Option<PluralForm>,
    entries: Vec<MsgEntry>,
}

/// A message entry in a gettext translation file.
#[derive(Clone, Debug)]
pub struct MsgEntry {
    pub id: MsgString,
    pub value: MsgValue,
}

/// A message string or plural set in a gettext translation file.
#[derive(Clone, Debug)]
pub enum MsgValue {
    Invariant(MsgString, Option<Vec<i8>>),
    Plural {
        plural_id: MsgString,
        values: Vec<MsgString>,
    },
}

impl Messages {
    /// Load message entries from a gettext translation file.
    ///
    /// See [`Parser`] for more information.
    pub fn from_file(file_path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = BufReader::new(File::open(file_path).expect("Failed to open gettext file"));
        let mut parser = Parser::new();

        for line in file.lines() {
            parser.parse_line(&line?)?;
        }

        Ok(parser.finish()?)
    }

    /// Construct an empty messages list configured with the specified plural form.
    pub fn with_plural_form(plural_form: PluralForm) -> Self {
        Messages {
            plural_form: Some(plural_form),
            entries: Vec::new(),
        }
    }

    /// Create a messages list with a single non-plural entry.
    ///
    /// The plural form for the messages is left unconfigured.
    pub fn starting_with(id: MsgString, msg_str: MsgString) -> Self {
        let first_entry = MsgEntry {
            id: id.clone(),
            value: MsgValue::Invariant(msg_str.clone(), argument_ordering(id, msg_str)),
        };

        Messages {
            plural_form: None,
            entries: vec![first_entry],
        }
    }

    /// Add a non-plural entry.
    pub fn add(&mut self, id: MsgString, msg_str: MsgString) {
        let entry = MsgEntry {
            id: id.clone(),
            value: MsgValue::Invariant(msg_str.clone(), argument_ordering(id, msg_str)),
        };

        self.entries.push(entry);
    }

    /// Add a plural entry.
    pub fn add_plural(&mut self, id: MsgString, plural_id: MsgString, values: Vec<MsgString>) {
        let entry = MsgEntry {
            id,
            value: MsgValue::Plural { plural_id, values },
        };

        self.entries.push(entry);
    }
}

impl IntoIterator for Messages {
    type Item = MsgEntry;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl From<MsgString> for MsgValue {
    fn from(string: MsgString) -> Self {
        MsgValue::Invariant(string, None)
    }
}

static NAMED_ARGUMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%\([a-zA-Z]+\)").unwrap());

fn argument_ordering(id: MsgString, msg_str: MsgString) -> Option<Vec<i8>> {
    if NAMED_ARGUMENT.is_match(&id) && NAMED_ARGUMENT.is_match(&msg_str) {
        // Extract arguments in id
        let id_args = extract_arguments(id);
        // Extract arguments in translation
        let value_args = extract_arguments(msg_str);
        // Set index as id order and value as translation order
        Some(
            id_args
                .iter()
                .map(|id_arg| value_args.iter().position(|value_arg| value_arg == id_arg))
                .map(|f| f.unwrap() as i8 + 1)
                .collect(),
        )
    } else {
        None
    }
}

fn extract_arguments(msg: MsgString) -> Vec<String> {
    NAMED_ARGUMENT
        .find_iter(&msg)
        .map(|s| String::from(s.as_str()))
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Parser error while parsing file
    #[error("Failed to parse input file")]
    Parse(#[from] super::parser::Error),

    /// IO error while reading input file.
    #[error("Failed to read from the input file")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use crate::gettext::messages::argument_ordering;
    use crate::gettext::MsgString;

    #[test]
    fn if_message_has_no_argument_should_have_no_argument_ordering() {
        let msg_id = MsgString::from_escaped("This is a text");
        let msg_str = MsgString::from_escaped("Det här är en text");
        let argument_ordering = argument_ordering(msg_id, msg_str);

        let expected = None;

        assert_eq!(argument_ordering, expected);
    }

    #[test]
    fn if_message_has_no_translation_should_have_no_argument_ordering() {
        let msg_id = MsgString::from_escaped("This is a %(text)");
        let msg_str = MsgString::from_escaped("");
        let argument_ordering = argument_ordering(msg_id, msg_str);

        let expected = None;

        assert_eq!(argument_ordering, expected);
    }

    #[test]
    fn if_argument_ordering_is_same_should_have_sequential_ordering() {
        let msg_id = MsgString::from_escaped("This is a %(text) and %(star)");
        let msg_str = MsgString::from_escaped("Det här är en %(text) och %(star)");
        let argument_ordering = argument_ordering(msg_id, msg_str);

        let expected = Some([1, 2].to_vec());

        assert_eq!(argument_ordering, expected);
    }

    #[test]
    fn if_argument_ordering_is_reversed_should_have_reversed_ordering() {
        let msg_id = MsgString::from_escaped("This is a %(text) and %(star)");
        let msg_str = MsgString::from_escaped("Det här är en %(star) och %(text)");
        let argument_ordering = argument_ordering(msg_id, msg_str);

        let expected = Some([2, 1].to_vec());

        assert_eq!(argument_ordering, expected);
    }

    #[test]
    fn if_argument_is_repeated_should_have_repeated_ordering() {
        let msg_id = MsgString::from_escaped("This is a %(text) and %(text)");
        let msg_str = MsgString::from_escaped("Det här är en %(text) och %(text)");
        let argument_ordering = argument_ordering(msg_id, msg_str);

        let expected = Some([1, 1].to_vec());

        assert_eq!(argument_ordering, expected);
    }
}

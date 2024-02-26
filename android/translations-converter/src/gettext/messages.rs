use super::{msg_string::MsgString, parser::Parser, plural_form::PluralForm};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
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
    Invariant(MsgString),
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
            id,
            value: MsgValue::Invariant(msg_str),
        };

        Messages {
            plural_form: None,
            entries: vec![first_entry],
        }
    }

    /// Add a non-plural entry.
    pub fn add(&mut self, id: MsgString, msg_str: MsgString) {
        let entry = MsgEntry {
            id,
            value: MsgValue::Invariant(msg_str),
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
        MsgValue::Invariant(string)
    }
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

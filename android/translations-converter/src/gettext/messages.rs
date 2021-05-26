use super::{msg_string::MsgString, parse_line, plural_form::PluralForm};
use derive_more::{Display, Error, From};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    mem,
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
    /// The only metadata that is parsed from the file is the "Plural-Form" header. It is assumed
    /// that the header value is one of some hard-coded values, so if new languages that have new
    /// plurals are added, the code will have to be updated.
    ///
    /// An gettext translation file has the format in the example below:
    ///
    /// ```
    /// # The start of the file can contain empty entries to include some header with meta
    /// # information. Below is the header indicating the plural format.
    /// msgid ""
    /// msgstr ""
    /// "Plural-Forms: nplurals=2; plural=(n != 1);"
    ///
    /// # Simple translated messages
    /// msgid "Message in original language"
    /// msgstr "Mesaĝo en tradukita lingvo"
    ///
    /// # Plural translated messages (with two forms)
    /// msgid "One translated message"
    /// msgid_plural "%d translated messages"
    /// msgstr[0] "Unu tradukita mesaĝo"
    /// msgstr[1] "%d tradukitaj mesaĝoj"
    /// ```
    pub fn from_file(file_path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut parsing_header = false;
        let mut entries = Vec::new();
        let mut current_id = None;
        let mut current_plural_id = None;
        let mut plural_form = None;
        let mut variants = BTreeMap::new();

        let file = BufReader::new(File::open(file_path).expect("Failed to open gettext file"));
        // Ensure there's an empty line at the end so that the "else" part of the string matching
        // code will run for the last message in the file.
        let lines = file.lines().chain(Some(Ok(String::new())));

        for line_result in lines {
            let line = line_result?;

            match_str! { (line.trim())
                ["msgid \"", msg_id, "\""] => {
                    current_id = Some(MsgString::from_escaped(msg_id));
                },
                ["msgstr \"", translation, "\""] => {
                    if let Some(id) = current_id.take() {
                        let value = MsgValue::Invariant(MsgString::from_escaped(translation));

                        parsing_header = id.is_empty() && translation.is_empty();

                        entries.push(MsgEntry { id, value });
                    }

                    current_id = None;
                    current_plural_id = None;
                },
                ["msgid_plural \"", plural_id, "\""] => {
                    current_plural_id = Some(MsgString::from_escaped(plural_id));
                    parsing_header = false;
                },
                ["msgstr[", plural_translation, "\""] => {
                    let variant_id_end = plural_translation
                        .chars()
                        .position(|character| character == ']')
                        .expect("Invalid plural msgstr");
                    let variant_id: usize = plural_translation[..variant_id_end]
                        .parse()
                        .expect("Invalid variant index");
                    let variant_msg = parse_line(&plural_translation[variant_id_end..], "] \"", "")
                        .expect("Invalid plural msgstr");

                    variants.insert(variant_id, MsgString::from_escaped(variant_msg));
                    parsing_header = false;
                },
                ["\"", header, "\\n\""] => {
                    if parsing_header {
                        if let Some(plural_formula) = parse_line(header, "Plural-Forms: ", ";") {
                            plural_form = PluralForm::from_formula(plural_formula);
                        }
                    }
                },
                _ => {
                    if let Some(plural_id) = current_plural_id.take() {
                        let id = current_id.take().expect("Missing msgid for plural message");
                        let values = mem::replace(&mut variants, BTreeMap::new())
                            .into_iter()
                            .enumerate()
                            .inspect(|(index, (variant_id, _))| {
                                assert_eq!(
                                    index, variant_id,
                                    "Unexpected variant ID for plural msgstr"
                                )
                            })
                            .map(|(_, (_, value))| value)
                            .collect();
                        let value = MsgValue::Plural { plural_id, values };

                        entries.push(MsgEntry { id, value });
                    }

                    current_id = None;
                    current_plural_id = None;
                    variants.clear();
                    parsing_header = false;
                },
            }
        }

        Ok(Messages {
            entries,
            plural_form,
        })
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

#[derive(Debug, Display, Error, From)]
pub enum Error {
    /// IO error while reading input file.
    #[display(fmt = "Failed to read from the input file")]
    Io(std::io::Error),
}

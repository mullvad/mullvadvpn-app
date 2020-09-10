use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
};

lazy_static! {
    static ref APOSTROPHE_VARIATION: Regex = Regex::new("’").unwrap();
    static ref PARAMETERS: Regex = Regex::new(r"%\([^)]*\)").unwrap();
}

/// A parsed gettext translation file.
pub struct Translation {
    entries: Vec<MsgEntry>,
}

/// A message entry in a gettext translation file.
#[derive(Clone, Debug)]
pub struct MsgEntry {
    pub id: String,
    pub value: MsgValue,
}

/// A message string or plural set in a gettext translation file.
#[derive(Clone, Debug)]
pub enum MsgValue {
    Invariant(String),
    Plural {
        plural_id: String,
        values: Vec<String>,
    },
}

impl IntoIterator for Translation {
    type Item = MsgEntry;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl From<String> for MsgValue {
    fn from(string: String) -> Self {
        MsgValue::Invariant(string)
    }
}

/// Load message entries from a gettext translation file.
///
/// The messages are normalized into a common format so that they can be compared to Android string
/// resource entries.
pub fn load_file(file_path: impl AsRef<Path>) -> Translation {
    let mut entries = Vec::new();
    let mut current_id = None;
    let file = BufReader::new(File::open(file_path).expect("Failed to open gettext file"));

    for line in file.lines() {
        let line = line.expect("Failed to read from gettext file");
        let line = line.trim();

        if let Some(msg_id) = parse_line(line, "msgid \"", "\"") {
            current_id = Some(normalize(msg_id));
        } else {
            if let Some(translation) = parse_line(line, "msgstr \"", "\"") {
                if let Some(id) = current_id.take() {
                    let value = MsgValue::from(normalize(translation));

                    entries.push(MsgEntry { id, value });
                }
            }

            current_id = None;
        }
    }

    Translation { entries }
}

/// Append message entries to a translation file.
///
/// This is used to append missing translation entries back to the base translation template file.
pub fn append_to_template(
    file_path: impl AsRef<Path>,
    entries: impl Iterator<Item = MsgEntry>,
) -> Result<(), io::Error> {
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(file_path)?;
    let mut writer = BufWriter::new(file);

    for entry in entries {
        writeln!(writer)?;
        writeln!(writer, "msgid {:?}", entry.id)?;

        match entry.value {
            MsgValue::Invariant(value) => writeln!(writer, "msgstr {:?}", value)?,
            MsgValue::Plural { plural_id, values } => {
                writeln!(writer, "msgid_plural {:?}", plural_id)?;

                for (index, value) in values.into_iter().enumerate() {
                    writeln!(writer, "msgstr[{}] {:?}", index, value)?;
                }
            }
        }
    }

    Ok(())
}

fn parse_line<'l>(line: &'l str, prefix: &str, suffix: &str) -> Option<&'l str> {
    if line.starts_with(prefix) && line.ends_with(suffix) {
        let start = prefix.len();
        let end = line.len() - suffix.len();

        Some(&line[start..end])
    } else {
        None
    }
}

fn normalize(string: &str) -> String {
    // Use a single common apostrophe character
    let string = APOSTROPHE_VARIATION.replace_all(&string, "'");
    // Mark where parameters are positioned, removing the parameter name
    let string = PARAMETERS.replace_all(&string, "%");

    string.into_owned()
}

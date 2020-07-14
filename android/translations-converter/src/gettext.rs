use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

lazy_static! {
    static ref APOSTROPHE_VARIATION: Regex = Regex::new("’").unwrap();
    static ref PARAMETERS: Regex = Regex::new(r"%\([^)]*\)").unwrap();
}

/// A message entry in a gettext translation file.
#[derive(Clone, Debug)]
pub struct MsgEntry {
    pub id: String,
    pub value: String,
}

/// Load message entries from a gettext translation file.
///
/// The messages are normalized into a common format so that they can be compared to Android string
/// resource entries.
pub fn load_file(file_path: impl AsRef<Path>) -> Vec<MsgEntry> {
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
                    let value = normalize(translation);

                    entries.push(MsgEntry { id, value });
                }
            }

            current_id = None;
        }
    }

    entries
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

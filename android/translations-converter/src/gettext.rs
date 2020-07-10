use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

/// A message entry in a gettext translation file.
#[derive(Clone, Debug)]
pub struct MsgEntry {
    pub id: String,
    pub value: String,
}

/// Load message entries from a gettext translation file.
pub fn load_file(file_path: impl AsRef<Path>) -> Vec<MsgEntry> {
    let mut entries = Vec::new();
    let mut current_id = None;
    let file = BufReader::new(File::open(file_path).expect("Failed to open gettext file"));

    for line in file.lines() {
        let line = line.expect("Failed to read from gettext file");
        let line = line.trim();

        if let Some(msg_id) = parse_line(line, "msgid \"", "\"") {
            current_id = Some(msg_id);
        } else {
            if let Some(value) = parse_line(line, "msgstr \"", "\"") {
                if let Some(id) = current_id.take() {
                    entries.push(MsgEntry { id, value });
                }
            }

            current_id = None;
        }
    }

    entries
}

fn parse_line(line: &str, prefix: &str, suffix: &str) -> Option<String> {
    if line.starts_with(prefix) && line.ends_with(suffix) {
        let start = prefix.len();
        let end = line.len() - suffix.len();

        Some(line[start..end].to_owned())
    } else {
        None
    }
}

#[macro_use]
mod match_str;
mod messages;
mod msg_string;
mod parser;
mod plural_form;

use std::{
    fs::OpenOptions,
    io::{self, BufWriter, Write},
    path::Path,
};

pub use self::{
    messages::{Messages, MsgEntry, MsgValue},
    msg_string::MsgString,
    plural_form::PluralForm,
};

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
    let mut sorted_entries: Vec<_> = entries.collect();
    let mut writer = BufWriter::new(file);

    sorted_entries.sort_by(|first, second| first.id.cmp(&second.id));

    for entry in sorted_entries {
        writeln!(writer)?;
        writeln!(writer, r#"msgid "{}""#, entry.id)?;

        match entry.value {
            MsgValue::Invariant(value) => writeln!(writer, r#"msgstr "{value}""#)?,
            MsgValue::Plural { plural_id, values } => {
                writeln!(writer, r#"msgid_plural "{plural_id}""#)?;

                for (index, value) in values.into_iter().enumerate() {
                    writeln!(writer, r#"msgstr[{index}] "{value}""#)?;
                }
            }
        }
    }

    Ok(())
}

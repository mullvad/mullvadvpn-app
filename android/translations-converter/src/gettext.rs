use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    mem,
    path::Path,
};

lazy_static! {
    static ref APOSTROPHE_VARIATION: Regex = Regex::new("’").unwrap();
    static ref PARAMETERS: Regex = Regex::new(r"%\([^)]*\)").unwrap();
}

/// A parsed gettext translation file.
#[derive(Clone, Debug)]
pub struct Translation {
    pub plural_form: Option<PluralForm>,
    entries: Vec<MsgEntry>,
}

/// Known plural forms.
#[derive(Clone, Copy, Debug)]
pub enum PluralForm {
    Single,
    SingularForOne,
    SingularForZeroAndOne,
    Polish,
    Russian,
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

/// A helper macro to match a string to various prefix and suffix combinations.
macro_rules! match_str {
    (
        ( $string:expr )
        $( [$start:expr, $middle:ident, $end:expr] => $body:tt )*
        _ => $else:expr $(,)*
    ) => {
        $(
            if let Some($middle) = parse_line($string, $start, $end) {
                $body
            } else
        )* {
            $else
        }
    };
}

impl Translation {
    /// Load message entries from a gettext translation file.
    ///
    /// The messages are normalized into a common format so that they can be compared to Android
    /// string resource entries.
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
    pub fn from_file(file_path: impl AsRef<Path>) -> Self {
        let mut parsing_header = false;
        let mut entries = Vec::new();
        let mut current_id = None;
        let mut current_plural_id = None;
        let mut plural_form = None;
        let mut variants = BTreeMap::new();
        let file = BufReader::new(File::open(file_path).expect("Failed to open gettext file"));

        for line in file.lines() {
            let line = line.expect("Failed to read from gettext file");
            let line = line.trim();

            match_str! { (line)
                ["msgid \"", msg_id, "\""] => {
                    current_id = Some(normalize(msg_id));
                }
                ["msgstr \"", translation, "\""] => {
                    if let Some(id) = current_id.take() {
                        let value = MsgValue::from(normalize(translation));

                        parsing_header = id.is_empty() && translation.is_empty();

                        entries.push(MsgEntry { id, value });
                    }

                    current_id = None;
                    current_plural_id = None;
                }
                ["msgid_plural \"", plural_id, "\""] => {
                    current_plural_id = Some(normalize(plural_id));
                    parsing_header = false;
                }
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

                    variants.insert(variant_id, normalize(variant_msg));
                    parsing_header = false;
                }
                ["\"", header, "\\n\""] => {
                    if parsing_header {
                        if let Some(plural_formula) = parse_line(header, "Plural-Forms: ", ";") {
                            plural_form = Some(PluralForm::from_formula(plural_formula));
                        }
                    }
                }
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
                }
            }
        }

        Self {
            entries,
            plural_form,
        }
    }
}

impl IntoIterator for Translation {
    type Item = MsgEntry;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl PluralForm {
    /// Obtain an instance based on a known plural formula.
    ///
    /// Plural variants need to be obtained using a formula. However, some locales have known
    /// formulas, so they can be represented as a known plural form. This constructor can return a
    /// plural form based on the formulas that are known to be used in the project.
    pub fn from_formula(formula: &str) -> Self {
        match formula {
            "nplurals=1; plural=0" => PluralForm::Single,
            "nplurals=2; plural=(n != 1)" => PluralForm::SingularForOne,
            "nplurals=2; plural=(n > 1)" => PluralForm::SingularForZeroAndOne,
            "nplurals=4; plural=(n==1 ? 0 : (n%10>=2 && n%10<=4) && (n%100<12 || n%100>14) ? 1 : n!=1 && (n%10>=0 && n%10<=1) || (n%10>=5 && n%10<=9) || (n%100>=12 && n%100<=14) ? 2 : 3)" => {
                PluralForm::Polish
            }
            "nplurals=4; plural=((n%10==1 && n%100!=11) ? 0 : ((n%10 >= 2 && n%10 <=4 && (n%100 < 12 || n%100 > 14)) ? 1 : ((n%10 == 0 || (n%10 >= 5 && n%10 <=9)) || (n%100 >= 11 && n%100 <= 14)) ? 2 : 3))" => {
                PluralForm::Russian
            }
            other => panic!("Unknown plural formula: {}", other),
        }
    }
}

impl From<String> for MsgValue {
    fn from(string: String) -> Self {
        MsgValue::Invariant(string)
    }
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

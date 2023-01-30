use super::{Messages, MsgString, PluralForm};
use derive_more::{Display, Error};
use std::{collections::BTreeMap, mem};

/// A gettext messages file parser.
///
/// Can parse both translations files and template files.
///
/// # Usage
///
/// The parser works by parsing individual lines. After creating a [`Parser`] instance, the input
/// lines should be sent to it through repeated calls to [`Parser::parse_line`], and afterwards
/// calling [`Parser::finish`] to finish parsing and obtain the parsed result.
///
/// The only metadata that is parsed from the file is the "Plural-Form" header. It is assumed
/// that the header value is one of some hard-coded values, so if new languages that have new
/// plurals are added, the code will have to be updated.
///
/// # Input example
///
/// A gettext translation file has the format in the example below:
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
#[derive(Debug)]
pub enum Parser {
    /// Initial state.
    ///
    /// No useful information has been extracted yet.
    Start,

    /// Possible start of file header.
    ///
    /// Found an empty message ID, if the next line is an empty message string the header of the
    /// file has been found.
    HeaderStart,

    /// Start of file header found.
    Header,

    /// Skipping to the end of the header.
    ///
    /// The useful information has already been extracted.
    HeaderEnd(Messages),

    /// Waiting for a next message section.
    ///
    /// Parser has completed parsing either at least one valid entry or the file header.
    Idle(Messages),

    /// New message entry.
    ///
    /// Parsed a new message ID.
    NewEntry { id: MsgString, messages: Messages },

    /// Parsing a message entry.
    ///
    /// Parsed a message ID and a message string, but the string could be incomplete with the rest
    /// of it spread among more lines.
    InvariantEntry {
        id: MsgString,
        message: MsgString,
        messages: Messages,
    },

    /// Detected that entry is for a plural.
    ///
    /// Found a plural ID, may have parsed variants.
    NewPluralEntry {
        id: MsgString,
        plural_id: MsgString,
        variants: BTreeMap<usize, MsgString>,
        messages: Messages,
    },

    /// Parsing a plural entry variant.
    ///
    /// Parsed the start of a plural variant string, but the string could be incomplete with the
    /// rest of it spread among more lines.
    PluralEntry {
        id: MsgString,
        plural_id: MsgString,
        index: usize,
        variant: MsgString,
        variants: BTreeMap<usize, MsgString>,
        messages: Messages,
    },

    /// Internal transition state.
    ///
    /// Used while a line is being parsed.
    Parsing,
}

impl Parser {
    /// Create a new [`Parser`] instance.
    ///
    /// Parsing can then be done by feeding lines to the instance using [`Parser::parse_line`] and
    /// finishing with a call to [`Parser::finish`] to obtain the parsed result.
    pub fn new() -> Self {
        Parser::Start
    }

    /// Parse an input line.
    pub fn parse_line(&mut self, line: &str) -> Result<(), Error> {
        let state = mem::replace(self, Parser::Parsing);

        *self = match state {
            Parser::Start => Self::parse_start(line)?,
            Parser::HeaderStart => Self::parse_header_start(line)?,
            Parser::Header => Self::parse_header(line)?,
            Parser::HeaderEnd(messages) => Self::parse_header_end(line, messages)?,
            Parser::Idle(messages) => Self::parse_idle(line, messages)?,
            Parser::NewEntry { id, messages } => Self::parse_new_entry(line, id, messages)?,
            Parser::InvariantEntry {
                id,
                message,
                messages,
            } => Self::parse_invariant_entry(line, id, message, messages)?,
            Parser::NewPluralEntry {
                id,
                plural_id,
                variants,
                messages,
            } => Self::parse_new_plural_entry(line, id, plural_id, variants, messages)?,
            Parser::PluralEntry {
                id,
                plural_id,
                index,
                variant,
                variants,
                messages,
            } => Self::parse_plural_entry(line, id, plural_id, index, variant, variants, messages)?,
            Parser::Parsing => unreachable!("Parser should never stop on the Parsing state"),
        };

        Ok(())
    }

    /// Finish parsing and obtain the parsed [`Messages].
    pub fn finish(self) -> Result<Messages, Error> {
        match self {
            // Input file is empty
            Parser::Start => Ok(Messages::default()),

            // A single empty msgid was parsed, but no msgstr for that entry (or header)
            Parser::HeaderStart => Err(Error::IncompleteEntry(MsgString::empty())),

            // Input file only contains headers that were ignored
            Parser::Header => Ok(Messages::default()),

            // Input file only contains headers, but the plural form was successfully parsed
            Parser::HeaderEnd(messages) => Ok(messages),

            // Parsing successful
            Parser::Idle(messages) => Ok(messages),

            // Input file ends on an incomplete entry
            Parser::NewEntry { id, .. } => Err(Error::IncompleteEntry(id)),

            // Input file ends on an invariant entry
            Parser::InvariantEntry {
                id,
                message,
                mut messages,
            } => {
                messages.add(id, message);

                Ok(messages)
            }

            // Input file ends with an empty plural entry
            Parser::NewPluralEntry { id, .. } => Err(Error::IncompletePluralEntry(id)),

            // Input file ends with a plural entry (it might be missing variants)
            Parser::PluralEntry {
                id,
                plural_id,
                index,
                variant,
                mut variants,
                mut messages,
            } => {
                variants.insert(index, variant);

                let variants = collect_variants(&id, variants)?;

                messages.add_plural(id, plural_id, variants);

                Ok(messages)
            }

            Parser::Parsing => unreachable!("Parser should never stop on the Parsing state"),
        }
    }

    fn parse_start(line: &str) -> Result<Parser, Error> {
        let next_state = match_str! { (line)
            // Ignore empty lines and comment lines
            [""] | ["#", ..] => Parser::Start,

            // An empty message ID may indicate the start of the header
            ["msgid \"\""] => Parser::HeaderStart,

            // Headers don't have context, so skip it and get ready to parse entries
            ["msgctxt ", ..] => Parser::Idle(Messages::default()),

            // File has no header, went directly to the first entry
            ["msgid \"", msg_id, "\""] => Parser::NewEntry {
                id: MsgString::from_escaped(msg_id),
                messages: Messages::default()
            },

            other => return Err(Error::UnexpectedLine(other.to_owned())),
        };

        Ok(next_state)
    }

    fn parse_header_start(line: &str) -> Result<Parser, Error> {
        let next_state = match_str! { (line)
            // Ignore comment lines
            ["#", ..] => Parser::HeaderStart,

            // An empty message string confirms the start of the header
            ["msgstr \"\""] => Parser::Header,

            // A non-empty message string means an entry with an empty ID has been parsed
            ["msgstr \"", string, "\""] => Parser::Idle(
                Messages::starting_with(MsgString::empty(), MsgString::from_escaped(string))
            ),

            // A plural ID means this is the start of a plural entry with an empty ID
            ["msgid_plural \"", plural_id, "\""] => Parser::NewPluralEntry {
                id: MsgString::empty(),
                plural_id: MsgString::from_escaped(plural_id),
                variants: BTreeMap::new(),
                messages: Messages::default(),
            },

            other => return Err(Error::UnexpectedLine(other.to_owned())),
        };

        Ok(next_state)
    }

    fn parse_header(line: &str) -> Result<Parser, Error> {
        let next_state = match_str! { (line)
            // Ignore comment lines
            ["#", ..] => Parser::HeaderStart,

            // An empty line marks the end of the header
            [""] => Parser::Idle(Messages::default()),

            // The Plural-Forms header is the only header that's currently used, so after finding
            // it the parser can skip to the end of the headers
            ["\"Plural-Forms: ", plural_formula, ";\\n\""] => {
                let plural_form = PluralForm::from_formula(plural_formula)
                    .ok_or_else(|| Error::UnrecognizedPluralFormula(plural_formula.to_owned()))?;

                Parser::HeaderEnd(Messages::with_plural_form(plural_form))
            },

            // Skip other headers
            ["\"", .., "\\n\""] => Parser::Header,

            other => return Err(Error::UnexpectedLine(other.to_owned())),
        };

        Ok(next_state)
    }

    fn parse_header_end(line: &str, messages: Messages) -> Result<Parser, Error> {
        let next_state = match_str! { (line)
            // An empty line marks the end of the header
            [""] => Parser::Idle(messages),

            // Ignore comment lines
            ["#", ..] => Parser::HeaderEnd(messages),

            // Skip any other headers
            ["\"", .., "\\n\""] => Parser::HeaderEnd(messages),

            other => return Err(Error::UnexpectedLine(other.to_owned())),
        };

        Ok(next_state)
    }

    fn parse_idle(line: &str, messages: Messages) -> Result<Parser, Error> {
        let next_state = match_str! { (line)
            // Ignore empty lines, comment lines and message context lines
            [""] | ["#", ..] | ["msgctxt ", ..] => Parser::Idle(messages),

            // Start of a new message entry
            ["msgid \"", msg_id, "\""] => Parser::NewEntry {
                id: MsgString::from_escaped(msg_id),
                messages,
            },

            other => return Err(Error::UnexpectedLine(other.to_owned())),
        };

        Ok(next_state)
    }

    fn parse_new_entry(line: &str, id: MsgString, messages: Messages) -> Result<Parser, Error> {
        let next_state = match_str! { (line)
            // Ignore comment lines
            ["#", ..] => Parser::NewEntry { id, messages },

            // A message string for an invariant entry
            ["msgstr \"", string, "\""] => Parser::InvariantEntry {
                id,
                message: MsgString::from_escaped(string),
                messages,
            },

            // A plural ID means this is the start of a plural entry
            ["msgid_plural \"", plural_id, "\""] => Parser::NewPluralEntry {
                id,
                plural_id: MsgString::from_escaped(plural_id),
                variants: BTreeMap::new(),
                messages,
            },

            other => return Err(Error::UnexpectedLine(other.to_owned())),
        };

        Ok(next_state)
    }

    fn parse_invariant_entry(
        line: &str,
        id: MsgString,
        mut message: MsgString,
        mut messages: Messages,
    ) -> Result<Parser, Error> {
        let next_state = match_str! { (line)
            // Ignore comment lines
            ["#", ..] => Parser::InvariantEntry { id, message, messages },

            // The entry message string continues on this line
            ["\"", string, "\""] => {
                message += MsgString::from_escaped(string);

                Parser::InvariantEntry { id, message, messages }
            },

            // End of the entry
            [""] => {
                messages.add(id, message);

                Parser::Idle(messages)
            },

            other => return Err(Error::UnexpectedLine(other.to_owned())),
        };

        Ok(next_state)
    }

    fn parse_new_plural_entry(
        line: &str,
        id: MsgString,
        plural_id: MsgString,
        variants: BTreeMap<usize, MsgString>,
        mut messages: Messages,
    ) -> Result<Parser, Error> {
        let next_state = match_str! { (line)
            // Ignore comment lines
            ["#", ..] => Parser::NewPluralEntry { id, plural_id, variants, messages },

            // A message string for a plural variant
            ["msgstr[", index_and_string, "\""] => {
                let (index, variant) = extract_plural_variant(index_and_string)?;

                Parser::PluralEntry {
                    id,
                    plural_id,
                    index,
                    variant,
                    variants,
                    messages,
                }
            },

            // An empty line marks the end of the plural entry
            [""] => {
                let variants = collect_variants(&id, variants)?;

                messages.add_plural(id, plural_id, variants);

                Parser::Idle(messages)
            },

            other => return Err(Error::UnexpectedLine(other.to_owned())),
        };

        Ok(next_state)
    }

    fn parse_plural_entry(
        line: &str,
        id: MsgString,
        plural_id: MsgString,
        index: usize,
        mut variant: MsgString,
        mut variants: BTreeMap<usize, MsgString>,
        mut messages: Messages,
    ) -> Result<Parser, Error> {
        let next_state = match_str! { (line)
            // Ignore comment lines
            ["#", ..] => {
                Parser::PluralEntry { id, plural_id, index, variant, variants, messages }
            },

            // The variant message string continues on this line
            ["\"", string, "\""] => {
                variant += MsgString::from_escaped(string);

                Parser::PluralEntry {
                    id,
                    plural_id,
                    index,
                    variant,
                    variants,
                    messages
                }
            },

            // A message string indicating the end of the current variant and th start of another
            ["msgstr[", index_and_string, "\""] => {
                let (new_index, new_variant) = extract_plural_variant(index_and_string)?;

                variants.insert(index, variant);

                Parser::PluralEntry {
                    id,
                    plural_id,
                    index: new_index,
                    variant: new_variant,
                    variants,
                    messages,
                }
            },

            // An empty line marks the end of the plural entry (and hence the current variant as
            // well)
            [""] => {
                variants.insert(index, variant);

                let variants = collect_variants(&id, variants)?;

                messages.add_plural(id, plural_id, variants);

                Parser::Idle(messages)
            },

            other => return Err(Error::UnexpectedLine(other.to_owned())),
        };

        Ok(next_state)
    }
}

/// Helper function to extract the plural variant index and message.
///
/// The parser will try to parse a plural line of the form `msgstr[1] "%d tradukitaj mesaĝoj"`.
/// When matching the line to the expected template, it will remove the `msgstr[` prefix and the
/// `"` suffix. This function will then parse the rest of the string (`1] "%d tradukitaj mesaĝoj`)
/// by extracting the index (1), and then extracting the message string by skipping the separator
/// (`] "`).
fn extract_plural_variant(index_and_string: &str) -> Result<(usize, MsgString), Error> {
    let recreate_line = || format!("msgstr[{index_and_string}\"");

    let parts: Vec<_> = index_and_string.splitn(2, "] \"").collect();

    if parts.len() != 2 {
        return Err(Error::InvalidPluralVariant(recreate_line()));
    }

    let index_string = parts[0];
    let message_string = parts[1];

    let index = index_string
        .parse()
        .map_err(|_| Error::InvalidPluralIndex(recreate_line()))?;

    let variant_message = MsgString::from_escaped(message_string);

    Ok((index, variant_message))
}

/// Helper function to collect parsed variants.
///
/// This will return only the variant messages in index order. The function will return an error if
/// any variant index is missing.
fn collect_variants(
    id: &MsgString,
    variant_map: BTreeMap<usize, MsgString>,
) -> Result<Vec<MsgString>, Error> {
    let index_count = variant_map.len();

    for index in 0..index_count {
        if !variant_map.contains_key(&index) {
            return Err(Error::IncompletePluralEntry(id.clone()));
        }
    }

    Ok(variant_map.into_values().collect())
}

/// Parsing errors.
#[derive(Clone, Debug, Display, Error, Eq, PartialEq)]
pub enum Error {
    /// An unexpected line was read while parsing.
    #[display(fmt = "Unexpected line parsing gettext messages: {_0}")]
    UnexpectedLine(#[error(not(source))] String),

    /// Input uses an unrecognized plural forumal.
    #[display(fmt = "Input uses an unrecognized formula for the plural form: {_0}")]
    UnrecognizedPluralFormula(#[error(not(source))] String),

    /// Input ended with an incomplete entry.
    #[display(fmt = "Input ended with an incomplete gettext entry with ID: {_0}")]
    IncompleteEntry(#[error(not(source))] MsgString),

    /// Plural entry definition is missing a plural variant.
    #[display(fmt = "Plural entry is missing a plural variant: {_0}")]
    IncompletePluralEntry(#[error(not(source))] MsgString),

    /// Plural variant is invalid.
    #[display(fmt = "Plural variant line is invalid: {_0}")]
    InvalidPluralVariant(#[error(not(source))] String),

    /// Plural variant index was not parsable.
    #[display(fmt = "Plural variant line contains an invalid index: {_0}")]
    InvalidPluralIndex(#[error(not(source))] String),
}

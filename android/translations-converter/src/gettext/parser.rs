use super::{messages::Messages, msg_string::MsgString, parse_line, PluralForm};
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
pub struct Parser {
    parsing_header: bool,
    messages: Messages,
    current_id: Option<MsgString>,
    current_plural_id: Option<MsgString>,
    variants: BTreeMap<usize, MsgString>,
}

impl Parser {
    /// Create a new [`Parser`] instance.
    ///
    /// Parsing can then be done by feeding lines to the instance using [`Parser::parse_line`] and
    /// finishing with a call to [`Parser::finish`] to obtain the parsed result.
    pub fn new() -> Self {
        Parser {
            parsing_header: false,
            messages: Messages::default(),
            current_id: None,
            current_plural_id: None,
            variants: BTreeMap::new(),
        }
    }

    /// Parse an input line.
    pub fn parse_line(&mut self, line: &str) -> Result<(), Error> {
        match_str! { (line.trim())
            ["msgid \"", msg_id, "\""] => {
                self.current_id = Some(MsgString::from_escaped(msg_id));
            },
            ["msgstr \"", translation, "\""] => {
                if let Some(id) = self.current_id.take() {
                    self.parsing_header = id.is_empty() && translation.is_empty();
                    self.messages.add(id, MsgString::from_escaped(translation));
                }

                self.current_id = None;
                self.current_plural_id = None;
            },
            ["msgid_plural \"", plural_id, "\""] => {
                self.current_plural_id = Some(MsgString::from_escaped(plural_id));
                self.parsing_header = false;
            },
            ["msgstr[", plural_translation, "\""] => {
                let variant_id_end = plural_translation
                    .chars()
                    .position(|character| character == ']')
                    .ok_or_else(|| Error::InvalidPluralVariant(plural_translation.to_owned()))?;
                let variant_id: usize = plural_translation[..variant_id_end]
                    .parse()
                    .map_err(|_| {
                        Error::InvalidPluralIndex(plural_translation[..variant_id_end].to_owned())
                    })?;
                let variant_msg = parse_line(&plural_translation[variant_id_end..], "] \"", "")
                    .ok_or_else(|| Error::InvalidPluralVariant(plural_translation.to_owned()))?;

                self.variants.insert(variant_id, MsgString::from_escaped(variant_msg));
                self.parsing_header = false;
            },
            ["\"", header, "\\n\""] => {
                if self.parsing_header {
                    if let Some(plural_formula) = parse_line(header, "Plural-Forms: ", ";") {
                        self.messages.plural_form = PluralForm::from_formula(plural_formula);
                    }
                }
            },
            line => {
                if let Some(plural_id) = self.current_plural_id.take() {
                    let id = self.current_id.take()
                        .ok_or_else(|| Error::UnexpectedLine(line.to_owned()))?;

                    let values = mem::replace(&mut self.variants, BTreeMap::new())
                        .into_iter()
                        .enumerate()
                        .map(|(index, (variant_id, value))| {
                            if index == variant_id {
                                Ok(value)
                            } else {
                                Err(Error::IncompletePluralEntry(id.clone()))
                            }
                        })
                        .collect::<Result<Vec<_>, Error>>()?;

                    self.messages.add_plural(id, plural_id, values);
                }

                self.current_id = None;
                self.current_plural_id = None;
                self.variants.clear();
                self.parsing_header = false;
            },
        }

        Ok(())
    }

    /// Finish parsing and obtain the parsed [`Messages].
    pub fn finish(mut self) -> Result<Messages, Error> {
        self.parse_line("")?;

        Ok(self.messages)
    }
}

/// Parsing errors.
#[derive(Clone, Debug, Display, Error, Eq, PartialEq)]
pub enum Error {
    /// An unexpected line was read while parsing.
    #[display(fmt = "Unexpected line parsing gettext messages: {}", _0)]
    UnexpectedLine(#[error(not(source))] String),

    /// Plural entry definition is missing a plural variant.
    #[display(fmt = "Plural entry is missing a plural variant: {}", _0)]
    IncompletePluralEntry(#[error(not(source))] MsgString),

    /// Plural variant is invalid.
    #[display(fmt = "Plural variant line is invalid: {}", _0)]
    InvalidPluralVariant(#[error(not(source))] String),

    /// Plural variant index was not parsable.
    #[display(fmt = "Plural variant line contains an invalid index: {}", _0)]
    InvalidPluralIndex(#[error(not(source))] String),
}

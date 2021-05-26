use super::{messages::Messages, msg_string::MsgString, parse_line, PluralForm};
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
    ///
    /// # Panics
    ///
    /// The method will panic if the line can not be parsed.
    pub fn parse_line(&mut self, line: &str) {
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
                    .expect("Invalid plural msgstr");
                let variant_id: usize = plural_translation[..variant_id_end]
                    .parse()
                    .expect("Invalid variant index");
                let variant_msg = parse_line(&plural_translation[variant_id_end..], "] \"", "")
                    .expect("Invalid plural msgstr");

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
            _ => {
                if let Some(plural_id) = self.current_plural_id.take() {
                    let id = self.current_id.take().expect("Missing msgid for plural message");
                    let values = mem::replace(&mut self.variants, BTreeMap::new())
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

                    self.messages.add_plural(id, plural_id, values);
                }

                self.current_id = None;
                self.current_plural_id = None;
                self.variants.clear();
                self.parsing_header = false;
            },
        }
    }

    /// Finish parsing and obtain the parsed [`Messages].
    pub fn finish(mut self) -> Messages {
        self.parse_line("");
        self.messages
    }
}

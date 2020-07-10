//! Helper tool to convert translations from gettext messages to Android string resources.
//!
//! The procedure for converting the translations is relatively simple. The base Android string
//! resources file is first loaded, and then each gettext translation file is loaded and compared to
//! the Android base strings. For every translation string that matches exactly the Android base
//! string value, the translated string is used.
//!
//! Note that this conversion procedure is very raw and likely very brittle, so while it works for
//! most cases, it is important to keep in mind that this is just a helper tool and manual steps are
//! likely to be needed from time to time.

mod android;
mod gettext;

use regex::Regex;
use std::{
    collections::HashMap,
    fs::{self, File},
};

fn main() {
    let strings_file = File::open("../src/main/res/values/strings.xml")
        .expect("Failed to open string resources file");
    let string_resources: android::StringResources =
        serde_xml_rs::from_reader(strings_file).expect("Failed to read string resources file");

    let line_breaks = Regex::new(r"\s*\n\s*").unwrap();

    let known_strings: HashMap<_, _> = string_resources
        .into_iter()
        .map(|string| {
            let android_id = string.name;
            let string_value = line_breaks.replace_all(&string.value, " ").into_owned();

            (string_value, android_id)
        })
        .collect();

    let locale_files = fs::read_dir("../../gui/locales")
        .expect("Failed to open root locale directory")
        .filter_map(|dir_entry_result| dir_entry_result.ok().map(|dir_entry| dir_entry.path()))
        .filter(|dir_entry_path| dir_entry_path.is_dir())
        .map(|dir_path| dir_path.join("messages.po"))
        .filter(|file_path| file_path.exists());

    for locale_file in locale_files {
        generate_translations(&known_strings, gettext::load_file(&locale_file));
    }
}

/// Generate translated Android resource strings for a locale.
///
/// Based on the gettext translated message entries, it finds the messages with message IDs that
/// match known Android string resource values, and obtains the string resource ID for the
/// translation.
fn generate_translations(
    known_strings: &HashMap<String, String>,
    translations: Vec<gettext::MsgEntry>,
) {
    let mut localized_resource = android::StringResources::new();

    for translation in translations {
        if let Some(android_key) = known_strings.get(&translation.id) {
            localized_resource.push(android::StringResource {
                name: android_key.clone(),
                value: translation.value,
            });
        }
    }

    dbg!(localized_resource);
}

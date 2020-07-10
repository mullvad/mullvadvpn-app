//! Helper tool to convert translations from gettext messages to Android string resources.

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
        dbg!(gettext::load_file(locale_file));
    }
}

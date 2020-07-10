//! Helper tool to convert translations from gettext messages to Android string resources.

mod android;

use regex::Regex;
use std::{collections::HashMap, fs::File};

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

    dbg!(known_strings);
}

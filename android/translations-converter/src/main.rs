//! Helper tool to convert translations from gettext messages to Android string resources.

mod android;

use std::fs::File;

fn main() {
    let strings_file = File::open("../src/main/res/values/strings.xml")
        .expect("Failed to open string resources file");
    let string_resources: android::StringResources =
        serde_xml_rs::from_reader(strings_file).expect("Failed to read string resources file");

    dbg!(string_resources);
}

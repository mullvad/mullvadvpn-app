//! Helper tool to convert translations from gettext messages to Android string resources.
//!
//! The procedure for converting the translations is relatively simple. The base Android string
//! resources file is first loaded, and then each gettext translation file is loaded and compared to
//! the Android base strings. For every translation string that matches exactly the Android base
//! string value (after a normalization pass described below), the translated string is used in the
//! new Android strings file for the respective locale.
//!
//! To make the comparison work on most strings, the Android and gettext messages are normalized
//! first. This means that new lines in the XML files are removed and collapsed into a single space,
//! the message parameters are changed so that they are in a common format, and there is also a
//! small workaround for having different apostrophe characters in the GUI in some messages.
//!
//! One dangerous assumption for the normalization is that the named parameters for the GUI are
//! supplied in the declared order on Android. This is because it's not possible to figure out the
//! order when only named parameters are used, and Android strings only supported numbered
//! parameters.
//!
//! Android's plural resources aren't currently translated, but this tool will convert them to
//! gettext message templates and append them to the message template file. It's important to note
//! that the first quantity item for a plural will be used as the `msgid`, so it shouldn't
//! have any parameters. The last quantity item for a plural will be used as the `msgid_plural`,
//! and it can contain parameters. This assumes a plural resource will have at least two items.
//! While it would still work with a single item, this is an unlikely case for a plural resource.
//!
//! Note that this conversion procedure is very raw and likely very brittle, so while it works for
//! most cases, it is important to keep in mind that this is just a helper tool and manual steps are
//! likely to be needed from time to time.

mod android;
mod gettext;

use std::{
    collections::HashMap,
    fs::{self, File},
    path::Path,
};

fn main() {
    let resources_dir = Path::new("../src/main/res");

    let strings_file = File::open(resources_dir.join("values/strings.xml"))
        .expect("Failed to open string resources file");
    let mut string_resources: android::StringResources =
        serde_xml_rs::from_reader(strings_file).expect("Failed to read string resources file");

    string_resources.normalize();
    string_resources.retain(|string| string.translatable);

    let mut known_urls = HashMap::with_capacity(string_resources.len());
    let mut known_strings = HashMap::with_capacity(string_resources.len());

    for string in string_resources {
        let destination = if string.value.starts_with("https://mullvad.net/en/") {
            &mut known_urls
        } else {
            &mut known_strings
        };

        if destination
            .insert(string.value.clone(), string.name)
            .is_some()
        {
            panic!(
                "String {:?} has more than one Android resource ID",
                string.value
            );
        }
    }

    let mut missing_translations = known_strings.clone();

    let plurals_file = File::open(resources_dir.join("values/plurals.xml"))
        .expect("Failed to open plurals resources file");
    let plural_resources: android::PluralResources =
        serde_xml_rs::from_reader(plurals_file).expect("Failed to read plural resources file");

    let locale_dir = Path::new("../../gui/locales");
    let locale_files = fs::read_dir(&locale_dir)
        .expect("Failed to open root locale directory")
        .filter_map(|dir_entry_result| dir_entry_result.ok().map(|dir_entry| dir_entry.path()))
        .filter(|dir_entry_path| dir_entry_path.is_dir())
        .map(|dir_path| dir_path.join("messages.po"))
        .filter(|file_path| file_path.exists());

    for locale_file in locale_files {
        let locale = locale_file
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let destination_dir = resources_dir.join(&android_locale_directory(locale));

        if !destination_dir.exists() {
            fs::create_dir(&destination_dir).expect("Failed to create Android locale directory");
        }

        generate_translations(
            locale,
            known_urls.clone(),
            known_strings.clone(),
            gettext::load_file(&locale_file),
            destination_dir.join("strings.xml"),
            &mut missing_translations,
        );
    }

    let template_path = locale_dir.join("messages.pot");

    if !missing_translations.is_empty() {
        println!("Appending missing translations to template file:");

        gettext::append_to_template(
            &template_path,
            missing_translations
                .into_iter()
                .inspect(|(missing_translation, id)| println!("  {}: {}", id, missing_translation))
                .map(|(id, _)| gettext::MsgEntry {
                    id,
                    value: String::new().into(),
                }),
        )
        .expect("Failed to append missing translations to message template file");
    }

    if !plural_resources.is_empty() {
        gettext::append_to_template(
            &template_path,
            plural_resources
                .into_iter()
                .inspect(|plural| {
                    let last_item = &plural.items.last().expect("Plural items are empty").string;

                    println!("  {}: {}", plural.name, last_item);
                })
                .map(|mut plural| {
                    let plural_id = plural.items.pop().expect("Plural items are empty").string;
                    plural.items.truncate(1);
                    let id = plural.items.remove(0).string;

                    gettext::MsgEntry {
                        id,
                        value: gettext::MsgValue::Plural {
                            plural_id,
                            values: vec!["".to_owned(), "".to_owned()],
                        },
                    }
                }),
        )
        .expect("Failed to append missing plural translations to message template file");
    }
}

/// Determines the localized value resources directory name based on a locale specification.
///
/// This just makes sure a locale such as `en-US' gets correctly mapped to the directory name
/// `values-en-rUS`.
fn android_locale_directory(locale: &str) -> String {
    let mut directory = String::from("values-");
    let mut parts = locale.split("-");

    directory.push_str(parts.next().unwrap());

    if let Some(region) = parts.next() {
        directory.push_str("-r");
        directory.push_str(region);
    }

    directory
}

/// Generate translated Android resource strings for a locale.
///
/// Based on the gettext translated message entries, it finds the messages with message IDs that
/// match known Android string resource values, and obtains the string resource ID for the
/// translation. An Android string resource XML file is created with the translated strings.
///
/// URL strings are treated differently. The "translated" URLs have a locale specified in them. If
/// mapping from the translation locale to a website locale fails, the "translated" URL is not
/// generated, and the app falls back to the original URL value with the english locale.
///
/// The missing translations map is updated to only contain the strings that aren't present in the
/// current locale, which means that in the end the map contains only the translations that aren't
/// present in any locale.
fn generate_translations(
    locale: &str,
    known_urls: HashMap<String, String>,
    mut known_strings: HashMap<String, String>,
    translations: gettext::Translation,
    output_path: impl AsRef<Path>,
    missing_translations: &mut HashMap<String, String>,
) {
    let mut localized_resource = android::StringResources::new();

    for translation in translations {
        if let gettext::MsgValue::Invariant(translation_value) = translation.value {
            if let Some(android_key) = known_strings.remove(&translation.id) {
                localized_resource.push(android::StringResource::new(
                    android_key,
                    &translation_value,
                ));
            }
        }
    }

    if let Some(web_locale) = website_locale(locale) {
        let locale_path = format!("/{}/", web_locale);

        for (url, android_key) in known_urls {
            localized_resource.push(android::StringResource::new(
                android_key,
                &url.replacen("/en/", &locale_path, 1),
            ));
        }
    }

    localized_resource.sort();

    fs::write(output_path, localized_resource.to_string())
        .expect("Failed to create Android locale file");

    missing_translations.retain(|translation, _| known_strings.contains_key(translation));
}

/// Tries to map a translation locale to a locale used on the Mullvad website.
///
/// The mapping is trivial if no region is specified. Otherwise the region code must be manually
/// converted.
fn website_locale(locale: &str) -> Option<&str> {
    match locale {
        locale if !locale.contains("-") => Some(locale),
        "zh-TW" => Some("zh-hant"),
        unknown_locale => {
            eprintln!("Unknown locale: {}", unknown_locale);
            None
        }
    }
}

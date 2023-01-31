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
//! Android's plural resources are also translated using the same principle. It's important to note
//! that the singular quantity item (i.e., the item where `quantity="one"`) for each Android plural
//! resource will be used as the `msgid` to be search for in the gettext translations file.
//!
//! Missing translations are appended to the gettext messages template file (`messages.pot`). These
//! are the entries for which no translation in any locale was found. When missing plurals are
//! appended to the template file, the new message entries are created using the singular quantity
//! item as the `msgid` and the other quantity item as the `msgid_plural`. Because of this, it is
//! important to note that the former can't have parameters, while the latter can. Otherwise, the
//! entries will have to be manually added.
//!
//! Note that this conversion procedure is very raw and likely very brittle, so while it works for
//! most cases, it is important to keep in mind that this is just a helper tool and manual steps are
//! likely to be needed from time to time.

mod android;
mod gettext;
mod normalize;

use crate::normalize::Normalize;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

fn main() {
    let resources_dir = Path::new("../app/src/main/res");

    let strings_file = File::open(resources_dir.join("values/strings.xml"))
        .expect("Failed to open string resources file");
    let string_resources: android::StringResources =
        quick_xml::de::from_reader(BufReader::new(strings_file))
            .expect("Failed to read string resources file");

    let known_strings: HashMap<_, _> = string_resources
        .into_iter()
        .map(|resource| (resource.value.normalize(), resource.name))
        .collect();

    let plurals_file = File::open(resources_dir.join("values/plurals.xml"))
        .expect("Failed to open plurals resources file");
    let plural_resources: android::PluralResources =
        quick_xml::de::from_reader(BufReader::new(plurals_file))
            .expect("Failed to read plural resources file");

    let known_plurals: HashMap<_, _> = plural_resources
        .iter()
        .map(|plural| {
            let name = plural.name.clone();
            let singular = plural
                .items
                .iter()
                .find(|variant| variant.quantity == android::PluralQuantity::One)
                .map(|variant| variant.string.to_string())
                .expect("Missing singular plural variant");

            (singular, name)
        })
        .collect();

    let locale_dir = Path::new("../../gui/locales");
    let locale_files = fs::read_dir(locale_dir)
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
        let destination_dir = resources_dir.join(android_locale_directory(locale));

        if !destination_dir.exists() {
            fs::create_dir(&destination_dir).expect("Failed to create Android locale directory");
        }

        let translations = gettext::Messages::from_file(&locale_file)
            .expect("Failed to load translations for a locale");

        generate_translations(
            known_strings.clone(),
            known_plurals.clone(),
            translations,
            destination_dir.join("strings.xml"),
            destination_dir.join("plurals.xml"),
        );
    }

    let template_path = locale_dir.join("messages.pot");
    let template = gettext::Messages::from_file(&template_path)
        .expect("Failed to load messages template file");

    let mut missing_translations = known_strings;
    let mut missing_plurals: HashMap<_, _> = known_plurals;

    for message in template {
        match message.value {
            gettext::MsgValue::Invariant(_) => missing_translations.remove(&message.id.normalize()),
            gettext::MsgValue::Plural { .. } => missing_plurals.remove(&message.id.normalize()),
        };
    }

    if !missing_translations.is_empty() {
        println!("Appending missing translations to template file:");

        gettext::append_to_template(
            &template_path,
            missing_translations
                .into_iter()
                .inspect(|(missing_translation, id)| println!("  {id}: {missing_translation}"))
                .map(|(id, _)| gettext::MsgEntry {
                    id: gettext::MsgString::from_unescaped(&id),
                    value: gettext::MsgString::empty().into(),
                }),
        )
        .expect("Failed to append missing translations to message template file");
    }

    if !missing_plurals.is_empty() {
        println!("Appending missing plural translations to template file:");

        gettext::append_to_template(
            &template_path,
            missing_plurals
                .into_iter()
                .filter_map(|(_, name)| plural_resources.iter().find(|plural| plural.name == name))
                .cloned()
                .inspect(|plural| {
                    let other_item = &plural
                        .items
                        .iter()
                        .find(|plural| plural.quantity == android::PluralQuantity::Other)
                        .expect("Plural items are empty")
                        .string;

                    println!("  {}: {}", plural.name, other_item);
                })
                .map(|mut plural| {
                    let singular_position = plural
                        .items
                        .iter()
                        .position(|plural| plural.quantity == android::PluralQuantity::One)
                        .expect("Missing singular variant to use as msgid");
                    let id = gettext::MsgString::from_escaped(
                        plural.items.remove(singular_position).string.to_string(),
                    );

                    let other_position = plural
                        .items
                        .iter()
                        .position(|plural| plural.quantity == android::PluralQuantity::Other)
                        .expect("Missing other variant to use as msgid_plural");
                    let plural_id = gettext::MsgString::from_escaped(
                        plural.items.remove(other_position).string.to_string(),
                    );

                    gettext::MsgEntry {
                        id,
                        value: gettext::MsgValue::Plural {
                            plural_id,
                            values: vec![gettext::MsgString::empty(), gettext::MsgString::empty()],
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
    let mut parts = locale.split('-');

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
/// The missing translations map is updated to only contain the strings that aren't present in the
/// current locale, which means that in the end the map contains only the translations that aren't
/// present in any locale.
fn generate_translations(
    mut known_strings: HashMap<String, String>,
    mut known_plurals: HashMap<String, String>,
    translations: gettext::Messages,
    strings_output_path: impl AsRef<Path>,
    plurals_output_path: impl AsRef<Path>,
) {
    let mut localized_strings = android::StringResources::new();
    let mut localized_plurals = android::PluralResources::new();

    let plural_quantities = android_plural_quantities_from_gettext_plural_form(
        translations
            .plural_form
            .expect("Missing plural form for translation"),
    );

    for translation in translations {
        match translation.value {
            gettext::MsgValue::Invariant(translation_value) => {
                if let Some(android_key) = known_strings.remove(&translation.id.normalize()) {
                    localized_strings.push(android::StringResource::new(
                        android_key,
                        &translation_value.normalize(),
                    ));
                }
            }
            gettext::MsgValue::Plural { values, .. } => {
                if let Some(android_key) = known_plurals.remove(&translation.id.normalize()) {
                    let values = values.into_iter().map(|message| message.normalize());

                    localized_plurals.push(android::PluralResource::new(
                        android_key,
                        plural_quantities.clone().zip(values),
                    ));
                }
            }
        }
    }

    localized_strings.sort();

    fs::write(strings_output_path, localized_strings.to_string())
        .expect("Failed to create Android locale file");

    fs::write(plurals_output_path, localized_plurals.to_string())
        .expect("Failed to create Android plurals file");
}

/// Converts a gettext plural form into the plural quantities used by Android.
///
/// Returns an iterator that can be zipped with the gettext plural variants to produce the Android
/// plural string items.
fn android_plural_quantities_from_gettext_plural_form(
    plural_form: gettext::PluralForm,
) -> impl Iterator<Item = android::PluralQuantity> + Clone {
    use android::PluralQuantity::*;
    use gettext::PluralForm;

    match plural_form {
        PluralForm::Single => vec![Other],
        PluralForm::SingularForOne | PluralForm::SingularForZeroAndOne => vec![One, Other],
        PluralForm::Polish | PluralForm::Russian => vec![One, Few, Many, Other],
    }
    .into_iter()
}

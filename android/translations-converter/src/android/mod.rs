mod plurals;
mod string_value;
mod strings;

pub use self::{
    plurals::{PluralQuantity, PluralResource, PluralResources},
    string_value::StringValue,
    strings::{StringResource, StringResources},
};

fn tag_name_to_string(tag_name: roxmltree::ExpandedName<'_, '_>) -> String {
    match tag_name.namespace() {
        Some(namespace) => format!("{}:{}", namespace, tag_name.name()),
        None => tag_name.name().to_owned(),
    }
}

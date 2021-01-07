use std::collections::BTreeMap;

pub const PRODUCT_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

pub fn collect() -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert("id".to_owned(), uuid::Uuid::new_v4().to_string());
    metadata.insert(
        "mullvad-product-version".to_owned(),
        PRODUCT_VERSION.to_owned(),
    );
    metadata.insert("os".to_owned(), mullvad_platform_metadata::version());
    metadata.extend(mullvad_platform_metadata::extra_metadata());
    metadata
}

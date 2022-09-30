use std::collections::BTreeMap;

pub fn collect() -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert("id".to_owned(), uuid::Uuid::new_v4().to_string());
    metadata.insert(
        "mullvad-product-version".to_owned(),
        mullvad_version::VERSION.to_owned(),
    );
    metadata.insert("os".to_owned(), talpid_platform_metadata::version());
    metadata.extend(talpid_platform_metadata::extra_metadata());
    metadata
}

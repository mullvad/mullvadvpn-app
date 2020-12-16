use mullvad_rpc::os;
use std::collections::BTreeMap;

pub const PRODUCT_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

pub fn collect() -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert("id".to_owned(), uuid::Uuid::new_v4().to_string());
    metadata.insert(
        "mullvad-product-version".to_owned(),
        PRODUCT_VERSION.to_owned(),
    );
    metadata.insert("os".to_owned(), os::os::version());
    metadata.extend(os::os::extra_metadata());
    metadata
}

use std::collections::{BTreeMap, HashMap};
use std::sync::{LazyLock, RwLock};

mod command;
use command::command_stdout_lossy;

pub fn version() -> String {
    let version = os_version();
    let api_level = get_prop("ro.build.version.sdk").unwrap_or_else(|| "N/A".to_owned());

    let manufacturer =
        get_prop("ro.product.manufacturer").unwrap_or_else(|| "Unknown brand".to_owned());
    let product = get_prop("ro.product.model").unwrap_or_else(|| "Unknown model".to_owned());

    format!(
        "Android {} (API: {}) - {} {}",
        version, api_level, manufacturer, product
    )
}

pub fn short_version() -> String {
    let version = os_version();

    format!("Android {}", version)
}

fn os_version() -> String {
    get_prop("ro.build.version.release").unwrap_or_else(|| "N/A".to_owned())
}

pub fn extra_metadata() -> impl Iterator<Item = (String, String)> {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "abi".to_owned(),
        get_prop("ro.product.cpu.abilist").unwrap_or_else(|| "N/A".to_owned()),
    );
    let extra = EXTRA_METADATA.read().unwrap();
    for (k, v) in extra.iter() {
        metadata.insert(k.clone(), v.clone());
    }
    metadata.into_iter()
}

fn get_prop(property: &str) -> Option<String> {
    command_stdout_lossy("getprop", &[property]).ok()
}

pub fn set_extra_metadata(extra: HashMap<String, String>) {
    *EXTRA_METADATA.write().unwrap() = extra;
}

static EXTRA_METADATA: LazyLock<RwLock<HashMap<String, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

use std::collections::HashMap;

mod command;
use command::command_stdout_lossy;

pub fn version() -> String {
    let version = get_prop("ro.build.version.release").unwrap_or_else(|| "N/A".to_owned());
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
    version()
}

pub fn extra_metadata() -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    metadata.insert(
        "abi".to_owned(),
        get_prop("ro.product.cpu.abilist").unwrap_or_else(|| "N/A".to_owned()),
    );
    metadata
}

fn get_prop(property: &str) -> Option<String> {
    command_stdout_lossy("getprop", &[property])
}

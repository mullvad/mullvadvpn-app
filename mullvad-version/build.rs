use regex::Regex;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let (metadata_path, version_regex) = match env::var("CARGO_CFG_TARGET_OS")
        .expect("CARGO_CFG_TARGET_OS should be set")
        .as_str()
    {
        "android" => (
            "../android/app/build.gradle.kts",
            Regex::new("versionName = \"([^\"]*)\"").unwrap(),
        ),
        "linux" | "windows" | "macos" => (
            "../gui/package.json",
            Regex::new("\"version\": \"([^\"]*)\"").unwrap(),
        ),
        target_os => panic!("Unsupported target OS: {target_os}"),
    };

    let metadata_content =
        fs::read_to_string(metadata_path).expect(&format!("Failed to read {metadata_path}"));
    let mut version_capture = version_regex.captures_iter(&metadata_content);
    let product_version = version_capture.next().expect("failed to find version")[1].to_owned();
    assert!(version_capture.next().is_none());

    fs::write(out_dir.join("product-version.txt"), &product_version).unwrap();
}

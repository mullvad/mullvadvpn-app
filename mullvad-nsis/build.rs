extern crate cbindgen;
use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config: cbindgen::Config = Default::default();
    config.language = cbindgen::Language::Cxx;
    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file("include/mullvad-nsis.h");
}

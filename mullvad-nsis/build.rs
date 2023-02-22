fn main() {
    #[cfg(all(target_arch = "x86", target_os = "windows"))]
    {
        extern crate cbindgen;
        use std::env;

        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut config: cbindgen::Config = Default::default();
        config.language = cbindgen::Language::Cxx;
        cbindgen::generate_with_config(&crate_dir, config)
            .unwrap()
            .write_to_file("include/mullvad-nsis.h");
    }
}

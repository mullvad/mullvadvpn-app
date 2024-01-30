fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    if std::env::var("TARGET").unwrap() == "aarch64-apple-ios" {
        cbindgen::Builder::new()
            .with_crate(crate_dir)
            .with_language(cbindgen::Language::C)
            .generate()
            .expect("failed to generate bindings")
            .write_to_file("include/mullvad-api.h");
    }
}

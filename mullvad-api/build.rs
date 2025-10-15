fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    match std::env::var("TARGET").unwrap().as_str() {
        "aarch64-apple-ios" | "aarch64-apple-ios-sim" => {
            cbindgen::Builder::new()
                .with_crate(crate_dir)
                .with_language(cbindgen::Language::C)
                .generate()
                .expect("failed to generate bindings")
                .write_to_file("include/mullvad-api.h");
        }
        _ => (),
    }
}

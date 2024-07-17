fn main() {
    #[cfg(target_os = "macos")]
    match std::env::var("TARGET").unwrap().as_str() {
        "aarch64-apple-ios" | "aarch64-apple-ios-sim" => {
            let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            cbindgen::Builder::new()
                .with_autogen_warning("// This file is generated automatically. To update it forcefully, run `cargo run -p mullvad-ios --target aarch64-apple-ios`.")
                .with_crate(crate_dir)
                .with_language(cbindgen::Language::C)
                .generate()
                .expect("failed to generate bindings")
                .write_to_file("../ios/MullvadRustRuntime/include/mullvad_rust_runtime.h");
        }
        &_ => (),
    }
}

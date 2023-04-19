#[cfg(any(target_os = "macos", target_os = "ios"))]
fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("failed to generate bindings")
        .write_to_file("include/shadowsocks.h");
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
fn main() {}

#[cfg(target_os = "windows")]
mod inner {
    use cbindgen::Language;

    pub fn main() {
        let target_triple = std::env::var("TARGET").expect("Missing 'TARGET'");

        if !target_triple.contains("i686-pc-windows") {
            return;
        }

        let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        cbindgen::Builder::new()
            .with_language(Language::Cxx)
            .with_crate(crate_dir)
            .generate()
            .unwrap()
            .write_to_file("include/mullvad-nsis.h");
    }
}

#[cfg(not(target_os = "windows"))]
mod inner {
    pub fn main() {}
}

fn main() {
    inner::main()
}

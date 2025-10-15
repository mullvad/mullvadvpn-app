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

    // In-app upgrade is a Windows & macOS-specific feature.
    println!("cargo::rustc-check-cfg=cfg(in_app_upgrade)");
    if matches!(target_os(), Os::Windows | Os::Macos) {
        println!(r#"cargo::rustc-cfg=in_app_upgrade"#);
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Os {
    Windows,
    Macos,
    Linux,
    Android,
}

fn target_os() -> Os {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    match target_os.as_str() {
        "windows" => Os::Windows,
        "macos" => Os::Macos,
        "linux" => Os::Linux,
        "android" => Os::Android,
        _ => panic!("Unsupported target os: {target_os}"),
    }
}

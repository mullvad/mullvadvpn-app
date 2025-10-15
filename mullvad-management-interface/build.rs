fn main() {
    tonic_build::compile_protos("proto/management_interface.proto").unwrap();

    // Enable DAITA by default on desktop and android
    println!("cargo::rustc-check-cfg=cfg(daita)");
    println!(r#"cargo::rustc-cfg=daita"#);

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

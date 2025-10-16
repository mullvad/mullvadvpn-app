fn main() {
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
    IOS,
}

fn target_os() -> Os {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    use Os::*;
    match target_os.as_str() {
        "windows" => Windows,
        "macos" => Macos,
        "linux" => Linux,
        "android" => Android,
        "ios" => IOS,
        _ => panic!("Unsupported target os: {target_os}"),
    }
}

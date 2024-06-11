fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");

    // Enable Daita by default on Linux and Windows.
    println!("cargo::rustc-check-cfg=cfg(daita)");
    if let "linux" | "windows" | "macos" = target_os.as_str() {
        println!(r#"cargo::rustc-cfg=daita"#);
    }
}

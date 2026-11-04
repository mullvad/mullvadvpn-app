use std::env;

// English (United States): `MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US)`
// Sourced from winnt.h: https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-makelangid
const LANG_ID_EN_US: u16 = 0x0409;

fn main() {
    if matches!(target_os(), Os::Windows) {
        let mut res = winresource::WindowsResource::new();
        res.set("ProductVersion", mullvad_version::VERSION);
        res.set_icon("../dist-assets/icon.ico");
        res.set_language(LANG_ID_EN_US);
        println!("cargo::rerun-if-env-changed=MULLVAD_ADD_MANIFEST");
        if env::var("MULLVAD_ADD_MANIFEST")
            .map(|s| s != "0")
            .unwrap_or(false)
        {
            res.set_manifest_file("mullvad-daemon.manifest");
        } else {
            println!("cargo::warning=Skipping mullvad-daemon manifest");
        }
        res.compile().expect("Unable to generate windows resources");
    }

    // Enable in-app upgrades on macOS and Windows
    println!("cargo::rustc-check-cfg=cfg(in_app_upgrade)");
    if matches!(target_os(), Os::Windows | Os::Macos) {
        println!(r#"cargo::rustc-cfg=in_app_upgrade"#);
    }

    if matches!(target_os(), Os::Macos) {
        // Set the minimum version of macOS on which mullvad-daemon can run.
        cfg_select! {
            target_arch = "x86_64"  => { println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.12"); }
            target_arch = "aarch64" => { println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=11.0"); }
            _ => {}
        }
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
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    match target_os.as_str() {
        "windows" => Os::Windows,
        "macos" => Os::Macos,
        "linux" => Os::Linux,
        "android" => Os::Android,
        _ => panic!("Unsupported target os: {target_os}"),
    }
}

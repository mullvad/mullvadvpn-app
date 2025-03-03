use anyhow::Context;
use std::env;

fn main() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        return Ok(());
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").context("Missing 'CARGO_CFG_TARGET_OS")?;
    match target_os.as_str() {
        "windows" => win_main(),
        _ => Ok(()),
    }
}

fn win_main() -> anyhow::Result<()> {
    use anyhow::Context;

    let mut res = winres::WindowsResource::new();

    res.set_language(make_lang_id(
        windows_sys::Win32::System::SystemServices::LANG_ENGLISH as u16,
        windows_sys::Win32::System::SystemServices::SUBLANG_ENGLISH_US as u16,
    ));

    println!("cargo:rerun-if-changed=loader.manifest");
    res.set_manifest_file("loader.manifest");

    res.compile().context("Failed to compile resources")
}

// Sourced from winnt.h: https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-makelangid
fn make_lang_id(p: u16, s: u16) -> u16 {
    (s << 10) | p
}

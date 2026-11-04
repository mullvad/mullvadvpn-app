use anyhow::Context;
use std::env;

// English (United States): `MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US)`
// Sourced from winnt.h: https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-makelangid
const LANG_ID_EN_US: u16 = 0x0409;

fn main() -> anyhow::Result<()> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").context("Missing 'CARGO_CFG_TARGET_OS")?;
    match target_os.as_str() {
        "windows" => win_main(),
        _ => Ok(()),
    }
}

fn win_main() -> anyhow::Result<()> {
    use anyhow::Context;

    let mut res = winresource::WindowsResource::new();

    res.set_language(LANG_ID_EN_US);

    if !cfg!(debug_assertions) {
        println!("cargo:rerun-if-changed=loader.manifest");
        res.set_manifest_file("loader.manifest");
    }
    res.set_icon("../dist-assets/icon.ico");

    res.compile().context("Failed to compile resources")
}

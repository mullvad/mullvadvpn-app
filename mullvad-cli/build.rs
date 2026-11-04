use std::env;

// English (United States): `MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US)`
// Sourced from winnt.h: https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-makelangid
const LANG_ID_EN_US: u16 = 0x0409;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "windows") {
        let mut res = winresource::WindowsResource::new();
        res.set("ProductVersion", mullvad_version::VERSION);
        res.set_icon("../dist-assets/icon.ico");
        res.set_language(LANG_ID_EN_US);
        res.compile().expect("Unable to generate windows resources");
    }
}

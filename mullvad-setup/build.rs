use anyhow::Context;

fn main() -> anyhow::Result<()> {
    if !std::env::var("TARGET")
        .context("missing TARGET")?
        .as_str()
        .contains("windows")
    {
        return Ok(());
    }

    let mut res = winres::WindowsResource::new();
    res.set("ProductVersion", mullvad_version::VERSION);
    res.set_icon("../dist-assets/icon.ico");
    res.set_language(make_lang_id(
        windows_sys::Win32::System::SystemServices::LANG_ENGLISH as u16,
        windows_sys::Win32::System::SystemServices::SUBLANG_ENGLISH_US as u16,
    ));

    res.set("FileDescription", "Mullvad VPN setup");

    res.compile().context("Failed to compile resources")
}

// Sourced from winnt.h: https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-makelangid
fn make_lang_id(p: u16, s: u16) -> u16 {
    (s << 10) | p
}

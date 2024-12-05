use std::{io, path::Path};

use anyhow::Context;

const IDB_X64EXE: usize = 1;
const IDB_ARM64EXE: usize = 2;

fn main() -> anyhow::Result<()> {
    if !std::env::var("TARGET")
        .context("missing TARGET")?
        .as_str()
        .starts_with("x86_64-pc-windows-")
    {
        // This crate only makes sense on x64 Windows
        return Ok(());
    }

    let x64_installer =
        std::env::var("WIN_X64_INSTALLER").context("Must set WIN_X64_INSTALLER path")?;
    let arm64_installer =
        std::env::var("WIN_ARM64_INSTALLER").context("Must set WIN_ARM64_INSTALLER path")?;

    build_resource_rust_header().context("failed to write resource.rs")?;

    let mut res = winres::WindowsResource::new();
    res.append_rc_content(&format!(
        r#"
#define IDB_X64EXE {IDB_X64EXE}
#define IDB_ARM64EXE {IDB_ARM64EXE}

IDB_X64EXE BINARY "{x64_installer}"
IDB_ARM64EXE BINARY "{arm64_installer}"
"#
    ));
    res.set("ProductVersion", mullvad_version::VERSION);
    res.set_icon("../dist-assets/icon.ico");
    res.set_language(make_lang_id(
        windows_sys::Win32::System::SystemServices::LANG_ENGLISH as u16,
        windows_sys::Win32::System::SystemServices::SUBLANG_ENGLISH_US as u16,
    ));

    println!("cargo:rerun-if-changed=windows-installer.manifest");
    res.set_manifest_file("windows-installer.manifest");
    res.set("FileDescription", "Mullvad VPN installer");

    res.compile().context("Failed to compile resources")
}

fn build_resource_rust_header() -> io::Result<()> {
    let resource_header = Path::new(&std::env::var("OUT_DIR").unwrap()).join("resource.rs");
    std::fs::write(
        resource_header,
        format!(
            "pub const IDB_X64EXE: usize = {IDB_X64EXE};\n
pub const IDB_ARM64EXE: usize = {IDB_ARM64EXE};\n"
        ),
    )
}

// Sourced from winnt.h: https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-makelangid
fn make_lang_id(p: u16, s: u16) -> u16 {
    (s << 10) | p
}

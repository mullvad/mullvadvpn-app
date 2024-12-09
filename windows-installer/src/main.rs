//! Universal Windows installer. This can only be built for x86 Windows. This is because the
//! installer must run on both x86 and ARM64. x86 binaries can run on ARM64, but not vice versa.
//!
//! Building this requires two inputs into build.rs:
//! * `WIN_X64_INSTALLER` - a path to the x64 Windows installer
//! * `WIN_ARM64_INSTALLER` - a path to the ARM64 Windows installer
#![windows_subsystem = "windows"]

use anyhow::{bail, Context};
use std::{
    ffi::{c_ushort, OsStr},
    io::{self, Write},
    process::{Command, ExitStatus},
};
use tempfile::TempPath;
use windows_sys::{
    w,
    Win32::System::{
        LibraryLoader::{FindResourceW, LoadResource, LockResource, SizeofResource},
        SystemInformation::{IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_ARM64},
        Threading::IsWow64Process2,
    },
};

mod resource {
    include!(concat!(env!("OUT_DIR"), "/resource.rs"));
}

fn main() -> anyhow::Result<()> {
    let resource_id = match get_native_arch()? {
        Architecture::X64 => ResourceId::X86Bin,
        Architecture::Arm64 => ResourceId::Arm64Bin,
        Architecture::Unsupported(arch) => {
            bail!("unsupported processor architecture {arch}");
        }
    };

    let exe_data = find_binary_data(resource_id)?;
    let path = write_file_to_temp(&exe_data)?;

    let status = run_with_forwarded_args(&path).context("Failed to run unpacked installer")?;

    // We cannot rely on drop here since we need to `exit`, so remove explicitly
    if let Err(error) = std::fs::remove_file(path) {
        eprintln!("Failed to remove unpacked installer: {error}");
    }

    std::process::exit(status.code().unwrap());
}

/// Run path and pass all arguments from `argv[1..]` to it
fn run_with_forwarded_args(path: impl AsRef<OsStr>) -> io::Result<ExitStatus> {
    let mut command = Command::new(path);

    let args = std::env::args().skip(1);
    command.args(args).status()
}

/// Write file to a temporary file and return its path
fn write_file_to_temp(data: &[u8]) -> anyhow::Result<TempPath> {
    let mut file = tempfile::NamedTempFile::new().context("Failed to create tempfile")?;
    file.write_all(data)
        .context("Failed to extract temporary installer")?;
    Ok(file.into_temp_path())
}

#[repr(usize)]
enum ResourceId {
    X86Bin = resource::IDB_X64EXE,
    Arm64Bin = resource::IDB_ARM64EXE,
}

/// Return a slice of data for the given resource
fn find_binary_data(resource_id: ResourceId) -> anyhow::Result<&'static [u8]> {
    // SAFETY: Looks unsafe but is actually safe. The cast is equivalent to `MAKEINTRESOURCE`,
    // which is not available in windows-sys, as it is a macro.
    // `resource_id` is guaranteed by the build script to refer to an actual resource.
    // See https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-findresourcew
    let resource_info = unsafe { FindResourceW(0, resource_id as usize as _, w!("BINARY")) };
    if resource_info == 0 {
        bail!("Failed to find resource: {}", io::Error::last_os_error());
    }

    // SAFETY: We have a valid resource info handle
    // NOTE: Resources loaded with LoadResource should not be freed.
    // See https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadresource
    let resource = unsafe { LoadResource(0, resource_info) };
    if resource.is_null() {
        bail!("Failed to load resource: {}", io::Error::last_os_error());
    }

    // SAFETY: We have a valid resource info handle
    let resource_size = unsafe { SizeofResource(0, resource_info) };
    if resource_size == 0 {
        bail!(
            "Failed to get resource size: {}",
            io::Error::last_os_error()
        );
    }

    // SAFETY: We have a valid resource info handle
    // NOTE: We do not need to unload this handle, because it doesn't actually lock anything.
    // See https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-lockresource
    let resource_data = unsafe { LockResource(resource as _) };
    if resource_data.is_null() {
        bail!(
            "Failed to get resource data: {}",
            io::Error::last_os_error()
        );
    }

    debug_assert!(resource_data.is_aligned());

    // SAFETY: The pointer is non-null, valid and constant for the remainder of the process lifetime
    let resource_slice = unsafe {
        std::slice::from_raw_parts(
            resource_data as *const u8,
            usize::try_from(resource_size).unwrap(),
        )
    };

    Ok(resource_slice)
}

#[derive(Debug)]
enum Architecture {
    X64,
    Arm64,
    Unsupported(u16),
}

/// Return native architecture (ignoring WOW64)
fn get_native_arch() -> io::Result<Architecture> {
    let mut running_arch: c_ushort = 0;
    let mut native_arch: c_ushort = 0;

    // SAFETY: Trivially safe, since we provide the required arguments. `hprocess == 0` is
    // undocumented but refers to the current process.
    let result = unsafe { IsWow64Process2(0, &mut running_arch, &mut native_arch) };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    match native_arch {
        IMAGE_FILE_MACHINE_AMD64 => Ok(Architecture::X64),
        IMAGE_FILE_MACHINE_ARM64 => Ok(Architecture::Arm64),
        other => Ok(Architecture::Unsupported(other)),
    }
}

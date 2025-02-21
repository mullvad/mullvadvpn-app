//! Detect the running platform's CPU architecture.

/// CPU architectures supported by the talpid family of crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    /// x86-64 architecture
    X86,
    /// ARM64 architecture
    Arm64,
}

/// Return native architecture (ignoring WOW64). If the native architecture can not be detected,
/// [`None`] is returned. This should never be the case on working X86_64 or Arm64 systems.
#[cfg(target_os = "windows")]
pub fn get_native_arch() -> Result<Option<Architecture>, std::io::Error> {
    use core::ffi::c_ushort;
    use windows_sys::Win32::System::SystemInformation::{
        IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_ARM64,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, IsWow64Process2};

    let native_arch = {
        let mut running_arch: c_ushort = 0;
        let mut native_arch: c_ushort = 0;

        // SAFETY: Trivially safe. The current process handle is a glorified constant.
        let current_process = unsafe { GetCurrentProcess() };

        // IsWow64Process2:
        // Determines whether the specified process is running under WOW64; also returns additional machine process and architecture information.
        //
        // SAFETY: Trivially safe, since we provide the required arguments.
        if 0 == unsafe { IsWow64Process2(current_process, &mut running_arch, &mut native_arch) } {
            return Err(std::io::Error::last_os_error());
        }

        native_arch
    };

    match native_arch {
        IMAGE_FILE_MACHINE_AMD64 => Ok(Some(Architecture::X86)),
        IMAGE_FILE_MACHINE_ARM64 => Ok(Some(Architecture::Arm64)),
        _other => Ok(None),
    }
}

/// Return native architecture.
#[cfg(not(target_os = "windows"))]
pub fn get_native_arch() -> Result<Option<Architecture>, std::io::Error> {
    const TARGET_ARCH: Option<Architecture> = if cfg!(any(target_arch = "x86_64",)) {
        Some(Architecture::X86)
    } else if cfg!(target_arch = "aarch64") {
        Some(Architecture::Arm64)
    } else {
        None
    };
    Ok(TARGET_ARCH)
}

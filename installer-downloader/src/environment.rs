use mullvad_update::version::VersionArchitecture;

/// The environment consists of globals and/or constants which need to be computed at runtime.
pub struct Environment {
    pub architecture: mullvad_update::format::Architecture,
}

pub enum Error {
    /// Failed to get the host's CPU architecture.
    // TODO: Attach underlying error
    Arch,
}

impl Environment {
    /// Try to load the environment.
    pub fn load() -> Result<Self, Error> {
        let architecture = Self::get_arch()?;

        Ok(Environment { architecture })
    }

    /// Try to map the host's CPU architecture to one of the CPU architectures the Mullvad VPN app
    /// supports.
    fn get_arch() -> Result<VersionArchitecture, Error> {
        let arch = talpid_platform_metadata::get_native_arch()
            .map_err(|_| Error::Arch)?
            .ok_or(Error::Arch)?;

        let arch = match arch {
            talpid_platform_metadata::Architecture::X86 => VersionArchitecture::X86,
            talpid_platform_metadata::Architecture::Arm64 => VersionArchitecture::Arm64,
        };

        Ok(arch)
    }
}

use std::path::PathBuf;

/// A process that is being excluded from the tunnel.
#[derive(Debug, Clone)]
pub struct ExcludedProcess {
    /// Process identifier.
    pub pid: u32,
    /// Path to the image that this process is an instance of.
    pub image: PathBuf,
    /// If true, then the process is split because its parent was split,
    /// not due to its path being in the config.
    pub inherited: bool,
}

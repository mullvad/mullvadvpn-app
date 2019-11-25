use super::Tun;

/// Windows tunnel implementation
pub struct WinTun {
    /// Name of tunnel interface
    pub interface_name: String,
}

impl Tun for WinTun {
    fn interface_name(&self) -> &str {
        &self.interface_name
    }
}

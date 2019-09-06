use super::{Tun, TunConfig, TunProvider};
use talpid_types::BoxedError;

/// Factory stub of tunnel devices.
pub struct StubTunProvider;

impl Default for StubTunProvider {
    fn default() -> Self {
        StubTunProvider
    }
}

impl TunProvider for StubTunProvider {
    fn create_tun(&self, _: TunConfig) -> Result<Box<dyn Tun>, BoxedError> {
        unimplemented!();
    }

    #[cfg(target_os = "android")]
    fn open_tun(&self) -> Result<(), BoxedError> {
        unimplemented!();
    }

    #[cfg(target_os = "android")]
    fn close_tun(&self) -> Result<(), BoxedError> {
        unimplemented!();
    }
}

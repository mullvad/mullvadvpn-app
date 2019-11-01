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
    fn get_tun(&mut self, _: TunConfig) -> Result<Box<dyn Tun>, BoxedError> {
        unimplemented!();
    }

    #[cfg(target_os = "android")]
    fn create_tun(&mut self) -> Result<(), BoxedError> {
        unimplemented!();
    }

    #[cfg(target_os = "android")]
    fn create_tun_if_closed(&mut self) -> Result<(), BoxedError> {
        unimplemented!();
    }

    #[cfg(target_os = "android")]
    fn close_tun(&mut self) {
        unimplemented!();
    }
}

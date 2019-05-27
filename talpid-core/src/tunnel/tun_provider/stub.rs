use super::{Tun, TunConfig, TunProvider};
use talpid_types::BoxedError;

/// Factory stub of tunnel devices.
pub struct StubTunProvider;

impl TunProvider for StubTunProvider {
    fn create_tun(&self, _: TunConfig) -> Result<Box<dyn Tun>, BoxedError> {
        unimplemented!();
    }
}

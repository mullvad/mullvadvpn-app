use super::TunConfig;

/// Error stub.
pub enum Error {}

/// Factory stub of tunnel devices.
pub struct StubTunProvider;

impl StubTunProvider {
    pub fn new() -> Self {
        StubTunProvider
    }

    pub fn get_tun(&mut self, _: TunConfig) -> Result<(), Error> {
        unimplemented!();
    }
}

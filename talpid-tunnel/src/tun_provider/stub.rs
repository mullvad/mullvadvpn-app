use super::TunConfig;

/// Error stub.
pub enum Error {}

/// Factory stub of tunnel devices.
pub struct StubTunProvider;

impl StubTunProvider {
    pub fn new(_: TunConfig) -> Self {
        StubTunProvider
    }

    pub fn open_tun(&mut self) -> Result<(), Error> {
        unimplemented!();
    }
}

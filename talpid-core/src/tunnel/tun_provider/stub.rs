use super::TunConfig;

/// Error stub.
pub enum Error {}

/// Factory stub of tunnel devices.
pub struct StubTunProvider;

impl StubTunProvider {
    fn new() -> Self {
        StubTunProvider
    }

    fn get_tun(&mut self, _: TunConfig) -> Result<StubTun, Error> {
        unimplemented!();
    }
}

/// Stub of a tunnel device.
pub struct StubTun;

impl StubTun {
    fn interface_name(&self) -> &str {
        unimplemented!();
    }
}

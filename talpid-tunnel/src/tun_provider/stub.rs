use super::TunConfig;

#[derive(Debug, thiserror::Error)]
/// Error stub.
pub enum Error {
    /// IO error
    #[error("IO error")]
    Io(#[from] std::io::Error),
}

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

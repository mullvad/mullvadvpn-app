use super::TestWrapperFunction;
use test_rpc::{meta::Os, mullvad_daemon::MullvadClientVersion};

#[derive(Clone)]
pub struct TestMetadata {
    pub name: &'static str,
    pub targets: &'static [Os],
    pub mullvad_client_version: MullvadClientVersion,
    pub func: TestWrapperFunction,
    pub priority: Option<i32>,
}

// Register our test metadata struct with inventory to allow submitting tests of this type.
inventory::collect!(TestMetadata);

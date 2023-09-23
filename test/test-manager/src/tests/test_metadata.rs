use super::TestWrapperFunction;
use test_rpc::mullvad_daemon::MullvadClientVersion;

pub struct TestMetadata {
    pub name: &'static str,
    pub command: &'static str,
    pub mullvad_client_version: MullvadClientVersion,
    pub func: TestWrapperFunction,
    pub priority: Option<i32>,
    pub always_run: bool,
    pub must_succeed: bool,
    pub cleanup: bool,
}

// Register our test metadata struct with inventory to allow submitting tests of this type.
inventory::collect!(TestMetadata);

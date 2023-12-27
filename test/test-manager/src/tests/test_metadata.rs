use super::TestWrapperFunction;
use test_rpc::meta::Os;
use test_rpc::mullvad_daemon::MullvadClientVersion;

pub struct TestMetadata {
    pub name: &'static str,
    pub command: &'static str,
    pub target_os: Option<Os>,
    pub mullvad_client_version: MullvadClientVersion,
    pub func: TestWrapperFunction,
    pub priority: Option<i32>,
    pub always_run: bool,
    pub must_succeed: bool,
    pub cleanup: bool,
}

impl TestMetadata {
    pub fn should_run_on_os(&self, os: Os) -> bool {
        self.target_os
            .map(|target_os| target_os == os)
            .unwrap_or(true)
    }
}

// Register our test metadata struct with inventory to allow submitting tests of this type.
inventory::collect!(TestMetadata);

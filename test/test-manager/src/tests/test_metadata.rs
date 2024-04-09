use super::TestWrapperFunction;
use test_rpc::{meta::Os, mullvad_daemon::MullvadClientVersion};

pub struct TestMetadata {
    pub name: &'static str,
    pub command: &'static str,
    pub targets: &'static [Os],
    pub mullvad_client_version: MullvadClientVersion,
    pub func: TestWrapperFunction,
    pub priority: Option<i32>,
    pub always_run: bool,
    pub must_succeed: bool,
    pub cleanup: bool,
}

impl TestMetadata {
    pub fn should_run_on_os(&self, os: Os) -> bool {
        self.targets.is_empty() || self.targets.contains(&os)
    }
}

// Register our test metadata struct with inventory to allow submitting tests of this type.
inventory::collect!(TestMetadata);

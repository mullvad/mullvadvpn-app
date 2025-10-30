use std::{net::Ipv4Addr, ops::Deref, sync::OnceLock};
use test_rpc::meta::Os;

pub static TEST_CONFIG: TestConfigContainer = TestConfigContainer::new();

/// Default `mullvad_host`. This should match the production env.
pub const DEFAULT_MULLVAD_HOST: &str = "mullvad.net";
/// Script for bootstrapping the test-runner after the test-manager has successfully logged in.
pub const BOOTSTRAP_SCRIPT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../scripts/",
    "ssh-setup.sh"
));

/// Constants that are accessible from each test via `TEST_CONFIG`.
/// The constants must be initialized before running any tests using `TEST_CONFIG.init()`.
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub account_number: String,

    pub artifacts_dir: String,
    pub app_package_filename: String,
    pub app_package_to_upgrade_from_filename: Option<String>,
    pub ui_e2e_tests_filename: Option<String>,

    /// Used to override MULLVAD_API_*, for conncheck,
    /// and for resolving relay IPs.
    pub mullvad_host: String,

    pub host_bridge_name: String,
    pub host_bridge_ip: Ipv4Addr,
    pub os: Os,
}

impl TestConfig {
    #[allow(clippy::too_many_arguments)]
    // TODO: This argument list is very long, we should strive to shorten it if possible.
    pub const fn new(
        account_number: String,
        artifacts_dir: String,
        app_package_filename: String,
        app_package_to_upgrade_from_filename: Option<String>,
        ui_e2e_tests_filename: Option<String>,
        mullvad_host: String,
        host_bridge_name: String,
        host_bridge_ip: Ipv4Addr,
        os: Os,
    ) -> Self {
        Self {
            account_number,
            artifacts_dir,
            app_package_filename,
            app_package_to_upgrade_from_filename,
            ui_e2e_tests_filename,
            mullvad_host,
            host_bridge_name,
            host_bridge_ip,
            os,
        }
    }
}

/// A script which should be run *in* the test runner before the test run begins.
#[derive(Clone, Debug)]
pub struct BootstrapScript(Vec<u8>);

impl Deref for BootstrapScript {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for BootstrapScript {
    fn default() -> Self {
        Self(Vec::from(BOOTSTRAP_SCRIPT))
    }
}

#[derive(Debug, Clone)]
pub struct TestConfigContainer(OnceLock<TestConfig>);

impl TestConfigContainer {
    const fn new() -> Self {
        TestConfigContainer(OnceLock::new())
    }

    /// Initializes the constants.
    ///
    /// # Panics
    ///
    /// This panics if the config has already been initialized.
    pub fn init(&self, inner: TestConfig) {
        self.0.set(inner).unwrap()
    }
}

impl Deref for TestConfigContainer {
    type Target = TestConfig;

    fn deref(&self) -> &Self::Target {
        self.0.get().unwrap()
    }
}

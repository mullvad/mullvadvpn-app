use once_cell::sync::OnceCell;
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};
use test_rpc::meta::Os;

// Default `mullvad_host`. This should match the production env.
pub const DEFAULT_MULLVAD_HOST: &str = "mullvad.net";

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

    pub os: Os,
    /// The OpenVPN CA certificate to use with the the installed Mullvad App.
    ///
    /// # Note
    /// This is expected to be *just the filename* of the CA certificate, i.e. not a full path.
    openvpn_certificate: String,
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
        os: Os,
        openvpn_certificate: String,
    ) -> Self {
        Self {
            account_number,
            artifacts_dir,
            app_package_filename,
            app_package_to_upgrade_from_filename,
            ui_e2e_tests_filename,
            mullvad_host,
            host_bridge_name,
            os,
            openvpn_certificate,
        }
    }

    /// The OpenVPN CA certificate to use with the installed Mullvad App.
    /// Note that this is the path *in the test-runner* where the new CA certificate is stored.
    pub fn openvpn_cert(&self) -> PathBuf {
        Path::new(&self.artifacts_dir).join(self.openvpn_certificate.clone())
    }
}

#[derive(Debug, Clone)]
pub struct TestConfigContainer(OnceCell<TestConfig>);

impl TestConfigContainer {
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

pub static TEST_CONFIG: TestConfigContainer = TestConfigContainer(OnceCell::new());

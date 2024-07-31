mod access_methods;
mod account;
pub mod config;
mod cve_2019_14899;
mod dns;
mod helpers;
mod install;
mod relay_ip_overrides;
mod settings;
mod software;
mod split_tunnel;
mod test_metadata;
mod tunnel;
mod tunnel_state;
mod ui;

use crate::{
    mullvad_daemon::{MullvadClientArgument, RpcClientProvider},
    package::get_version_from_path,
};
use anyhow::Context;
use config::TEST_CONFIG;
use helpers::install_app;
pub use install::test_upgrade_app;
use mullvad_management_interface::MullvadProxyClient;
pub use test_metadata::TestMetadata;
use test_rpc::ServiceClient;

use futures::future::BoxFuture;

use std::time::Duration;

const WAIT_FOR_TUNNEL_STATE_TIMEOUT: Duration = Duration::from_secs(40);

#[derive(Clone)]
pub struct TestContext {
    pub rpc_provider: RpcClientProvider,
}

pub type TestWrapperFunction =
    fn(TestContext, ServiceClient, MullvadClientArgument) -> BoxFuture<'static, anyhow::Result<()>>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("RPC call failed")]
    Rpc(#[from] test_rpc::Error),

    #[error("geoip lookup failed")]
    GeoipLookup(#[source] test_rpc::Error),

    #[error("Found running daemon unexpectedly")]
    DaemonRunning,

    #[error("Daemon unexpectedly not running")]
    DaemonNotRunning,

    #[error("The daemon returned an error: {0}")]
    Daemon(String),

    #[error("The daemon ended up in the error state")]
    UnexpectedErrorState(talpid_types::tunnel::ErrorState),

    #[error("The gRPC client ran into an error: {0}")]
    ManagementInterface(#[from] mullvad_management_interface::Error),

    #[error("GUI test binary missing")]
    MissingGuiTest,

    #[cfg(target_os = "macos")]
    #[error("An error occurred: {0}")]
    Other(String),
}

/// Get a list of all tests, sorted by priority.
pub fn get_tests() -> Vec<&'static TestMetadata> {
    let mut tests: Vec<_> = inventory::iter::<TestMetadata>().collect();
    tests.sort_by_key(|test| test.priority.unwrap_or(0));
    tests
}

/// Make sure the daemon is installed and logged in. and restore settings to the defaults.
pub async fn prepare_daemon(
    rpc: &ServiceClient,
    rpc_provider: &RpcClientProvider,
) -> anyhow::Result<()> {
    log::debug!("Cleaning up daemon in test cleanup");
    // Check if daemon should be restarted

    let mut mullvad_client = restart_daemon(rpc, rpc_provider)
        .await
        .context("Failed to restart daemon")?;

    helpers::ensure_logged_in(&mut mullvad_client).await?;

    mullvad_client
        .reset_settings()
        .await
        .context("Failed to reset settings")?;
    helpers::disconnect_and_wait(&mut mullvad_client)
        .await
        .context("Failed to disconnect daemon after test")?;

    Ok(())
}

/// Reset the daemons environment.
///
/// Will and restart or reinstall it if necessary.
async fn restart_daemon(
    rpc: &ServiceClient,
    rpc_provider: &RpcClientProvider,
) -> anyhow::Result<MullvadProxyClient> {
    let mut mullvad_client = rpc_provider.new_client().await;
    let app_package_filename = &TEST_CONFIG.app_package_filename;

    use mullvad_management_interface::Error::*;
    match mullvad_client.get_current_version().await {
        // Failing to reach the daemon is a sign that it is not installed
        Err(Rpc(..)) => {
            log::info!("Could not reach active daemon before test, (re)installing app");
            // NOTE: Reinstalling the app resets the daemon environment
            mullvad_client = install_app(rpc, app_package_filename, rpc_provider)
                .await
                .with_context(|| format!("Failed to install app '{app_package_filename}'"))?;
        }
        Err(e) => return Err(anyhow::anyhow!(e).context("Failed to get app version")),
        Ok(version) => {
            if version != get_version_from_path(std::path::Path::new(app_package_filename))? {
                log::info!("Daemon version mismatch, re-installing app");
                mullvad_client = install_app(rpc, app_package_filename, rpc_provider)
                    .await
                    .context("Failed to install app")?;
            }
            helpers::ensure_daemon_environment(rpc)
                .await
                .context("Failed to reset daemon environment")?;
        }
    }
    Ok(mullvad_client)
}

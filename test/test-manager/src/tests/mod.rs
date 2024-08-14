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

pub use test_metadata::TestMetadata;

use anyhow::Context;
use futures::future::BoxFuture;
use std::time::Duration;

use crate::{
    mullvad_daemon::{MullvadClientArgument, RpcClientProvider},
    tests::helpers::get_app_env,
};
use test_rpc::ServiceClient;

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

/// Restore settings to the defaults.
pub async fn cleanup_after_test(
    rpc: ServiceClient,
    rpc_provider: &RpcClientProvider,
) -> anyhow::Result<()> {
    log::debug!("Resetting daemon settings after test");
    // Check if daemon should be restarted.
    restart_daemon(rpc).await?;
    let mut mullvad_client = rpc_provider.new_client().await;
    mullvad_client
        .reset_settings()
        .await
        .context("Failed to reset settings")?;
    helpers::disconnect_and_wait(&mut mullvad_client).await?;
    Ok(())
}

/// Conditionally restart the running daemon
///
/// If the daemon was started with non-standard environment variables, subsequent tests may break
/// due to assuming a default configuration. In that case, reset the environment variables and
/// restart.
async fn restart_daemon(rpc: ServiceClient) -> anyhow::Result<()> {
    let current_env = rpc.get_daemon_environment().await?;
    let default_env = get_app_env().await?;
    if current_env != default_env {
        log::debug!("Restarting daemon due changed environment variables. Values since last test {current_env:?}");
        rpc.set_daemon_environment(default_env).await?;
    }
    Ok(())
}

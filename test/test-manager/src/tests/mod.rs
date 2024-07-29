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
    tests::helpers::get_app_env,
};
use anyhow::Context;
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

    #[error("DNS lookup failed")]
    DnsLookup(#[source] std::io::Error),

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
    mut rpc: ServiceClient,
    rpc_provider: &RpcClientProvider,
) -> anyhow::Result<()> {
    log::debug!("Cleaning up daemon in test cleanup");
    // Check if daemon should be restarted.
    // TODO: Move this shizzle up one level?
    // TODO: The daemon needs to be up and running after this line.
    if let Err(daemon_restart_error) = restart_daemon(&rpc).await {
        match daemon_restart_error {
            // Something went wrong in the communication between test-manager <-> test-runner.
            Error::Rpc(rpc_error) => {
                log::warn!("Could not restart the daemon due to RPC-error: {rpc_error}");
                // TODO: Try to create a new gRPC client. Need to move this logic up one level to
                // do this.

                // Restart the test-runner
                // HACK: Accomplish this by restarting the virtual machine. This should not be
                // necessary.
                rpc.reboot().await.inspect_err(|_| {
                    log::error!("Failed to reboot test runner virtual machine!");
                })?;
                rpc.reset_daemon_environment().await.inspect_err(|daemon_restart_failure| {
                    log::warn!("Rebooting the test runner virtual machine did not work: {daemon_restart_failure}")
                })?;
            }
            // Something wen't wrong in the daemon.
            daemon_error @ (Error::DaemonNotRunning
            | Error::Daemon(_)
            | Error::UnexpectedErrorState(_)
            | Error::ManagementInterface(_)) => {
                log::warn!("Could not restart daemon due to daemon error: {daemon_error}");
                log::warn!("Rebooting the test runner virtual machine");
                // First, reboot the test-runner VM.
                rpc.reboot().await.inspect_err(|_| {
                    log::error!("Failed to reboot test runner virtual machine!");
                })?;
                if let Err(daemon_restart_failure) = rpc.reset_daemon_environment().await {
                    log::warn!("Rebooting the test runner virtual machine did not work: {daemon_restart_failure}");
                    log::warn!("Reinstalling the app");
                    // TODO: If rebooting the VM did not work, try to re-install the app.
                }
            }
            // We don't really care about these errors in this context.
            non_fatal @ (Error::DaemonRunning
            | Error::GeoipLookup(_)
            | Error::DnsLookup(_)
            | Error::MissingGuiTest) => {
                log::warn!("Could not restart daemon due to non-fatal error: {non_fatal}");
                log::warn!("Restarting dameon one more time");
                rpc.reset_daemon_environment().await?;
            }
            #[cfg(target_os = "macos")]
            Other(_) => todo!("Remove this variant, we can't handle this error properly"),
        }
    }
    let mut mullvad_client = rpc_provider.new_client().await;

    helpers::disconnect_and_wait(&mut mullvad_client).await?;

    // Bring all the settings into scope so we remember to reset them.
    let mullvad_types::settings::Settings {
        relay_settings,
        bridge_settings,
        obfuscation_settings,
        bridge_state,
        custom_lists,
        api_access_methods,
        allow_lan,
        block_when_disconnected,
        auto_connect,
        tunnel_options,
        relay_overrides,
        show_beta_releases,
        #[cfg(target_os = "macos")]
            split_tunnel: _,
        settings_version: _, // N/A
    } = Default::default();

    mullvad_client
        .clear_custom_access_methods()
        .await
        .context("Could not clear custom api access methods")?;
    for access_method in api_access_methods.iter() {
        mullvad_client
            .update_access_method(access_method.clone())
            .await
            .context("Could not reset default access method")?;
    }

    mullvad_client
        .set_relay_settings(relay_settings)
        .await
        .context("Could not set relay settings")?;

    let _ = relay_overrides;
    mullvad_client
        .clear_all_relay_overrides()
        .await
        .context("Could not set relay overrides")?;

    mullvad_client
        .set_auto_connect(auto_connect)
        .await
        .context("Could not set auto connect in cleanup")?;

    mullvad_client
        .set_allow_lan(allow_lan)
        .await
        .context("Could not set allow lan in cleanup")?;

    mullvad_client
        .set_show_beta_releases(show_beta_releases)
        .await
        .context("Could not set show beta releases in cleanup")?;

    mullvad_client
        .set_bridge_state(bridge_state)
        .await
        .context("Could not set bridge state in cleanup")?;

    mullvad_client
        .set_bridge_settings(bridge_settings.clone())
        .await
        .context("Could not set bridge settings in cleanup")?;

    mullvad_client
        .set_obfuscation_settings(obfuscation_settings.clone())
        .await
        .context("Could set obfuscation settings in cleanup")?;

    mullvad_client
        .set_block_when_disconnected(block_when_disconnected)
        .await
        .context("Could not set block when disconnected setting in cleanup")?;

    mullvad_client
        .clear_split_tunnel_apps()
        .await
        .context("Could not clear split tunnel apps in cleanup")?;

    mullvad_client
        .clear_split_tunnel_processes()
        .await
        .context("Could not clear split tunnel processes in cleanup")?;

    mullvad_client
        .set_dns_options(tunnel_options.dns_options.clone())
        .await
        .context("Could not clear dns options in cleanup")?;

    mullvad_client
        .set_quantum_resistant_tunnel(tunnel_options.wireguard.quantum_resistant)
        .await
        .context("Could not clear PQ options in cleanup")?;

    let _ = custom_lists;
    mullvad_client
        .clear_custom_lists()
        .await
        .context("Could not remove custom list")?;

    Ok(())
}

/// Conditionally restart the running daemon
///
/// If the daemon was started with non-standard environment variables, subsequent tests may break
/// due to assuming a default configuration. In that case, reset the environment variables and
/// restart.
async fn restart_daemon(rpc: &ServiceClient) -> Result<(), Error> {
    let current_env = rpc.get_daemon_environment().await?;
    let default_env = get_app_env().await?;
    if current_env != default_env {
        log::debug!("Restarting daemon due changed environment variables. Values since last test {current_env:?}");
        rpc.set_daemon_environment(default_env).await?;
    }
    Ok(())
}

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

/// Restore settings to the defaults.
pub async fn reset_before_test(
    rpc: ServiceClient,
    rpc_provider: &RpcClientProvider,
) -> anyhow::Result<()> {
    log::debug!("Cleaning up daemon in test cleanup");
    // Check if daemon should be restarted

    let mut mullvad_client = rpc_provider.new_client().await;

    restart_daemon(&rpc, &mut mullvad_client)
        .await
        .context("Failed to restart daemon")?;

    helpers::disconnect_and_wait(&mut mullvad_client)
        .await
        .context("Failed to disconnect daemon after test")?;

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

/// Reset the daemons environment.
///
/// Will and restart or reinstall it if necessary.
async fn restart_daemon(
    rpc: &ServiceClient,
    mullvad_client: &mut MullvadProxyClient,
) -> anyhow::Result<()> {
    let app_package_filename = &TEST_CONFIG.app_package_filename;

    use mullvad_management_interface::Error::*;
    match mullvad_client.get_current_version().await {
        // Failing to reach the daemon is a sign that it is not installed
        Err(Rpc(..)) => {
            log::info!("Failed to reach daemon before test, re-installing app");
            // NOTE: Reinstalling the app resets the daemon environment
            install_app(rpc, app_package_filename)
                .await
                .context("Failed to install app")?;
        }
        Err(e) => return Err(e.into()),
        Ok(version) => {
            if version != get_version_from_path(std::path::Path::new(app_package_filename))? {
                log::info!("Daemon version mismatch, re-installing app");
                install_app(rpc, app_package_filename)
                    .await
                    .context("Failed to install app")?;
            }
            helpers::reset_daemon_environment(rpc).await?;
        }
    }
    Ok(())
}

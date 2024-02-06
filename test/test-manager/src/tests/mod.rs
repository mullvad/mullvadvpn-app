mod access_methods;
mod account;
pub mod config;
mod dns;
mod helpers;
mod install;
mod settings;
mod software;
mod test_metadata;
mod tunnel;
mod tunnel_state;
mod ui;

use crate::mullvad_daemon::RpcClientProvider;
use anyhow::Context;
pub use test_metadata::TestMetadata;
use test_rpc::ServiceClient;

use futures::future::BoxFuture;

use mullvad_management_interface::MullvadProxyClient;
use std::time::Duration;

const PING_TIMEOUT: Duration = Duration::from_secs(3);
const WAIT_FOR_TUNNEL_STATE_TIMEOUT: Duration = Duration::from_secs(40);

#[derive(Clone)]
pub struct TestContext {
    pub rpc_provider: RpcClientProvider,
}

pub type TestWrapperFunction = Box<
    dyn Fn(
        TestContext,
        ServiceClient,
        Box<dyn std::any::Any + Send>,
    ) -> BoxFuture<'static, Result<(), Error>>,
>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "RPC call failed")]
    Rpc(#[source] test_rpc::Error),

    #[error(display = "Timeout waiting for ping")]
    PingTimeout,

    #[error(display = "geoip lookup failed")]
    GeoipLookup(test_rpc::Error),

    #[error(display = "Found running daemon unexpectedly")]
    DaemonRunning,

    #[error(display = "Daemon unexpectedly not running")]
    DaemonNotRunning,

    #[error(display = "The daemon returned an error: {}", _0)]
    Daemon(String),

    #[error(display = "The daemon ended up in the error state")]
    UnexpectedErrorState(talpid_types::tunnel::ErrorState),

    #[error(display = "The gRPC client ran into an error: {}", _0)]
    ManagementInterface(#[source] mullvad_management_interface::Error),

    #[cfg(target_os = "macos")]
    #[error(display = "An error occurred: {}", _0)]
    Other(String),
}

/// Restore settings to the defaults.
pub async fn cleanup_after_test(mullvad_client: &mut MullvadProxyClient) -> anyhow::Result<()> {
    log::debug!("Cleaning up daemon in test cleanup");

    helpers::disconnect_and_wait(mullvad_client).await?;

    let default_settings = mullvad_types::settings::Settings::default();

    mullvad_client
        .set_relay_settings(default_settings.relay_settings)
        .await
        .context("Could not set relay settings")?;
    mullvad_client
        .set_auto_connect(default_settings.auto_connect)
        .await
        .context("Could not set auto connect in cleanup")?;
    mullvad_client
        .set_allow_lan(default_settings.allow_lan)
        .await
        .context("Could not set allow lan in cleanup")?;
    mullvad_client
        .set_show_beta_releases(default_settings.show_beta_releases)
        .await
        .context("Could not set show beta releases in cleanup")?;
    mullvad_client
        .set_bridge_state(default_settings.bridge_state)
        .await
        .context("Could not set bridge state in cleanup")?;
    mullvad_client
        .set_bridge_settings(default_settings.bridge_settings.clone())
        .await
        .context("Could not set bridge settings in cleanup")?;
    mullvad_client
        .set_obfuscation_settings(default_settings.obfuscation_settings.clone())
        .await
        .context("Could set obfuscation settings in cleanup")?;
    mullvad_client
        .set_block_when_disconnected(default_settings.block_when_disconnected)
        .await
        .context("Could not set block when disconnected setting in cleanup")?;
    #[cfg(target_os = "windows")]
    mullvad_client
        .clear_split_tunnel_apps()
        .await
        .context("Could not clear split tunnel apps in cleanup")?;
    #[cfg(target_os = "linux")]
    mullvad_client
        .clear_split_tunnel_processes()
        .await
        .context("Could not clear split tunnel processes in cleanup")?;
    mullvad_client
        .set_dns_options(default_settings.tunnel_options.dns_options.clone())
        .await
        .context("Could not clear dns options in cleanup")?;
    mullvad_client
        .set_quantum_resistant_tunnel(default_settings.tunnel_options.wireguard.quantum_resistant)
        .await
        .context("Could not clear PQ options in cleanup")?;

    Ok(())
}

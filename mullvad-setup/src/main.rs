use clap::Parser;
use once_cell::sync::Lazy;
use std::{path::PathBuf, process, str::FromStr, time::Duration};

use mullvad_api::{self, proxy::ApiConnectionMode, DEVICE_NOT_FOUND};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::version::ParsedAppVersion;
use talpid_core::firewall::{self, Firewall};
use talpid_future::retry::{retry_future, ConstantInterval};
use talpid_types::ErrorExt;

static APP_VERSION: Lazy<ParsedAppVersion> =
    Lazy::new(|| ParsedAppVersion::from_str(mullvad_version::VERSION).unwrap());

const DEVICE_REMOVAL_STRATEGY: ConstantInterval = ConstantInterval::new(Duration::ZERO, Some(5));

#[repr(i32)]
enum ExitStatus {
    Ok = 0,
    Error = 1,
    VersionNotOlder = 2,
    DaemonNotRunning = 3,
}

impl From<Error> for ExitStatus {
    fn from(error: Error) -> ExitStatus {
        match error {
            Error::RpcConnectionError(_) => ExitStatus::DaemonNotRunning,
            _ => ExitStatus::Error,
        }
    }
}

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to connect to RPC client")]
    RpcConnectionError(#[error(source)] mullvad_management_interface::Error),

    #[error(display = "RPC call failed")]
    DaemonRpcError(#[error(source)] mullvad_management_interface::Error),

    #[error(display = "This command cannot be run if the daemon is active")]
    DaemonIsRunning,

    #[error(display = "Firewall error")]
    FirewallError(#[error(source)] firewall::Error),

    #[error(display = "Failed to initialize mullvad RPC runtime")]
    RpcInitializationError(#[error(source)] mullvad_api::Error),

    #[error(display = "Failed to remove device from account")]
    RemoveDeviceError(#[error(source)] mullvad_api::rest::Error),

    #[error(display = "Failed to obtain settings directory path")]
    SettingsPathError(#[error(source)] mullvad_paths::Error),

    #[error(display = "Failed to obtain cache directory path")]
    CachePathError(#[error(source)] mullvad_paths::Error),

    #[error(display = "Failed to read the device cache")]
    ReadDeviceCacheError(#[error(source)] mullvad_daemon::device::Error),

    #[error(display = "Failed to write the device cache")]
    WriteDeviceCacheError(#[error(source)] mullvad_daemon::device::Error),

    #[error(display = "Cannot parse the version string")]
    ParseVersionStringError,
}

#[derive(Debug, Parser)]
#[command(author, version = mullvad_version::VERSION, about, long_about = None)]
#[command(propagate_version = true)]
#[command(
    arg_required_else_help = true,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
enum Cli {
    /// Move a running daemon into a blocking state and save its target state
    PrepareRestart,
    /// Remove any firewall rules introduced by the daemon
    ResetFirewall,
    /// Remove the current device from the active account
    RemoveDevice,
    /// Checks whether the given version is older than the current version
    IsOlderVersion {
        /// Version string to compare the current version
        #[arg(required = true)]
        old_version: String,
    },
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let result = match Cli::parse() {
        Cli::PrepareRestart => prepare_restart().await,
        Cli::ResetFirewall => reset_firewall().await,
        Cli::RemoveDevice => remove_device().await,
        Cli::IsOlderVersion { old_version } => {
            match is_older_version(&old_version) {
                // Returning exit status
                Ok(status) => process::exit(status as i32),
                Err(error) => Err(error),
            }
        }
    };

    if let Err(e) = result {
        eprintln!("{}", e.display_chain());
        process::exit(ExitStatus::from(e) as i32);
    }
}

fn is_older_version(old_version: &str) -> Result<ExitStatus, Error> {
    let parsed_version =
        ParsedAppVersion::from_str(old_version).map_err(|_| Error::ParseVersionStringError)?;

    Ok(if parsed_version < *APP_VERSION {
        ExitStatus::Ok
    } else {
        ExitStatus::VersionNotOlder
    })
}

async fn prepare_restart() -> Result<(), Error> {
    let mut rpc = MullvadProxyClient::new()
        .await
        .map_err(Error::RpcConnectionError)?;
    rpc.prepare_restart().await.map_err(Error::DaemonRpcError)?;
    Ok(())
}

async fn reset_firewall() -> Result<(), Error> {
    // Ensure that the daemon isn't running
    if MullvadProxyClient::new().await.is_ok() {
        return Err(Error::DaemonIsRunning);
    }

    Firewall::new(
        #[cfg(target_os = "linux")]
        mullvad_types::TUNNEL_FWMARK,
    )
    .map_err(Error::FirewallError)?
    .reset_policy()
    .map_err(Error::FirewallError)
}

async fn remove_device() -> Result<(), Error> {
    let (cache_path, settings_path) = get_paths()?;
    let (cacher, state) = mullvad_daemon::device::DeviceCacher::new(&settings_path)
        .await
        .map_err(Error::ReadDeviceCacheError)?;
    if let Some(device) = state.into_device() {
        let api_runtime = mullvad_api::Runtime::with_cache(&cache_path, false)
            .await
            .map_err(Error::RpcInitializationError)?;

        let proxy = mullvad_api::DevicesProxy::new(
            api_runtime
                .mullvad_rest_handle(
                    ApiConnectionMode::try_from_cache(&cache_path)
                        .await
                        .into_repeat(),
                )
                .await,
        );

        let device_removal = retry_future(
            move || proxy.remove(device.account_token.clone(), device.device.id.clone()),
            move |result| match result {
                Err(error) => error.is_network_error(),
                _ => false,
            },
            DEVICE_REMOVAL_STRATEGY,
        )
        .await;

        // `DEVICE_NOT_FOUND` is not considered to be an error in this context.
        match device_removal {
            Ok(_) => Ok(()),
            Err(mullvad_api::rest::Error::ApiError(_status, code)) if code == DEVICE_NOT_FOUND => {
                Ok(())
            }
            Err(e) => Err(Error::RemoveDeviceError(e)),
        }?;

        cacher
            .remove()
            .await
            .map_err(Error::WriteDeviceCacheError)?;
    }

    Ok(())
}

fn get_paths() -> Result<(PathBuf, PathBuf), Error> {
    let cache_path = mullvad_paths::cache_dir().map_err(Error::CachePathError)?;
    let settings_path = mullvad_paths::settings_dir().map_err(Error::SettingsPathError)?;
    Ok((cache_path, settings_path))
}

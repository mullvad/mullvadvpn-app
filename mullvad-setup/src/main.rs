use clap::{crate_authors, crate_description, crate_name, App};
use mullvad_api::{self, proxy::ApiConnectionMode};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::version::ParsedAppVersion;
use std::{path::PathBuf, process, str::FromStr, time::Duration};
use talpid_core::{
    firewall::{self, Firewall},
    future_retry::{constant_interval, retry_future_n},
};
use talpid_types::ErrorExt;

lazy_static::lazy_static! {
    static ref APP_VERSION: ParsedAppVersion = ParsedAppVersion::from_str(mullvad_version::VERSION).unwrap();
    static ref IS_DEV_BUILD: bool = APP_VERSION.is_dev();
}

const KEY_RETRY_INTERVAL: Duration = Duration::ZERO;
const KEY_RETRY_MAX_RETRIES: usize = 4;

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

#[tokio::main]
async fn main() {
    env_logger::init();

    let subcommands = vec![
        App::new("prepare-restart")
            .about("Move a running daemon into a blocking state and save its target state"),
        App::new("reset-firewall").about("Remove any firewall rules introduced by the daemon"),
        App::new("remove-device").about("Remove the current device from the active account"),
        App::new("is-older-version")
            .about("Checks whether the given version is older than the current version")
            .arg(
                clap::Arg::new("OLDVERSION")
                    .required(true)
                    .help("Version string to compare the current version"),
            ),
    ];

    let app = clap::App::new(crate_name!())
        .version(mullvad_version::VERSION)
        .author(crate_authors!())
        .about(crate_description!())
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .global_setting(clap::AppSettings::DisableHelpSubcommand)
        .global_setting(clap::AppSettings::DisableVersionFlag)
        .subcommands(subcommands);

    let matches = app.get_matches();
    let result = match matches.subcommand() {
        Some(("prepare-restart", _)) => prepare_restart().await,
        Some(("reset-firewall", _)) => reset_firewall().await,
        Some(("remove-device", _)) => remove_device().await,
        Some(("is-older-version", sub_matches)) => {
            let old_version = sub_matches.value_of("OLDVERSION").unwrap();
            match is_older_version(old_version) {
                // Returning exit status
                Ok(status) => process::exit(status as i32),
                Err(error) => Err(error),
            }
        }
        _ => unreachable!("No command matched"),
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
                    |_| async { true },
                )
                .await,
        );
        retry_future_n(
            move || proxy.remove(device.account_token.clone(), device.device.id.clone()),
            move |result| match result {
                Err(error) => error.is_network_error(),
                _ => false,
            },
            constant_interval(KEY_RETRY_INTERVAL),
            KEY_RETRY_MAX_RETRIES,
        )
        .await
        .map_err(Error::RemoveDeviceError)?;

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

use clap::{crate_authors, crate_description, crate_name, SubCommand};
use mullvad_daemon::account_history;
use mullvad_management_interface::new_rpc_client;
use mullvad_rpc::MullvadRpcRuntime;
use std::{path::PathBuf, process};
use talpid_core::firewall::{self, Firewall, FirewallArguments};
use talpid_types::ErrorExt;

pub const PRODUCT_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

#[cfg(windows)]
mod daemon_paths;

#[cfg(not(windows))]
type PathError = mullvad_paths::Error;

#[cfg(windows)]
type PathError = std::io::Error;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to connect to RPC client")]
    RpcConnectionError(#[error(source)] mullvad_management_interface::Error),

    #[error(display = "RPC call failed")]
    DaemonRpcError(#[error(source)] mullvad_management_interface::Status),

    #[error(display = "This command cannot be run if the daemon is active")]
    DaemonIsRunning,

    #[error(display = "Firewall error")]
    FirewallError(#[error(source)] firewall::Error),

    #[error(display = "Failed to initialize mullvad RPC runtime")]
    RpcInitializationError(#[error(source)] mullvad_rpc::Error),

    #[error(display = "Failed to obtain settings directory path")]
    SettingsPathError(#[error(source)] PathError),

    #[error(display = "Failed to obtain cache directory path")]
    CachePathError(#[error(source)] PathError),

    #[error(display = "Failed to initialize account history")]
    InitializeAccountHistoryError(#[error(source)] account_history::Error),

    #[error(display = "Failed to initialize account history")]
    ClearAccountHistoryError(#[error(source)] account_history::Error),
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let subcommands = vec![
        SubCommand::with_name("prepare-restart")
            .about("Move a running daemon into a blocking state and save its target state"),
        SubCommand::with_name("reset-firewall")
            .about("Remove any firewall rules introduced by the daemon"),
        SubCommand::with_name("clear-history").about("Clear account history"),
    ];

    let app = clap::App::new(crate_name!())
        .version(PRODUCT_VERSION)
        .author(crate_authors!())
        .about(crate_description!())
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .global_settings(&[
            clap::AppSettings::DisableHelpSubcommand,
            clap::AppSettings::VersionlessSubcommands,
        ])
        .subcommands(subcommands);

    let matches = app.get_matches();
    let result = match matches.subcommand_name().expect("Subcommand has no name") {
        "prepare-restart" => prepare_restart().await,
        "reset-firewall" => reset_firewall().await,
        "clear-history" => clear_history().await,
        _ => unreachable!("No command matched"),
    };

    if let Err(e) = result {
        eprintln!("{}", e.display_chain());
        process::exit(1);
    }
}

async fn prepare_restart() -> Result<(), Error> {
    let mut rpc = new_rpc_client().await.map_err(Error::RpcConnectionError)?;
    rpc.prepare_restart(())
        .await
        .map_err(Error::DaemonRpcError)?;
    Ok(())
}

async fn reset_firewall() -> Result<(), Error> {
    // Ensure that the daemon isn't running
    if let Ok(_) = new_rpc_client().await {
        return Err(Error::DaemonIsRunning);
    }

    let mut firewall = Firewall::new(FirewallArguments {
        initialize_blocked: false,
        allow_lan: true,
    })
    .map_err(Error::FirewallError)?;

    firewall.reset_policy().map_err(Error::FirewallError)
}

async fn clear_history() -> Result<(), Error> {
    let (cache_path, resource_path, settings_path) = get_paths()?;

    let mut rpc_runtime = MullvadRpcRuntime::with_cache(
        tokio::runtime::Handle::current(),
        &resource_path,
        Some(&cache_path),
    )
    .await
    .map_err(Error::RpcInitializationError)?;

    let mut account_history = account_history::AccountHistory::new(
        &cache_path,
        &settings_path,
        rpc_runtime.mullvad_rest_handle(),
    )
    .await
    .map_err(Error::InitializeAccountHistoryError)?;
    account_history
        .clear()
        .await
        .map_err(Error::ClearAccountHistoryError)?;
    Ok(())
}

#[cfg(not(windows))]
fn get_paths() -> Result<(PathBuf, PathBuf, PathBuf), Error> {
    let cache_path = mullvad_paths::cache_dir().map_err(Error::CachePathError)?;
    let resource_path = mullvad_paths::get_resource_dir();
    let settings_path = mullvad_paths::settings_dir().map_err(Error::SettingsPathError)?;
    Ok((cache_path, resource_path, settings_path))
}

#[cfg(windows)]
fn get_paths() -> Result<(PathBuf, PathBuf, PathBuf), Error> {
    let settings_path =
        daemon_paths::get_mullvad_daemon_settings_path().map_err(Error::CachePathError)?;
    let resource_path = mullvad_paths::get_resource_dir();
    let cache_path =
        daemon_paths::get_mullvad_daemon_cache_path().map_err(Error::SettingsPathError)?;

    Ok((cache_path, resource_path, settings_path))
}

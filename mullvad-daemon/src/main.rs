//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

#![deny(rust_2018_idioms)]

use log::{debug, error, info, warn};
use mullvad_daemon::{logging, version, Daemon};
use std::{path::PathBuf, thread, time::Duration};
use talpid_types::ErrorExt;

mod cli;
mod shutdown;
#[cfg(windows)]
mod system_service;

const DAEMON_LOG_FILENAME: &str = "daemon.log";

fn main() {
    let config = cli::get_config();
    let log_dir = init_logging(config).unwrap_or_else(|error| {
        eprintln!("{}", error);
        std::process::exit(1)
    });
    let exit_code = match run_platform(config, log_dir) {
        Ok(_) => 0,
        Err(error) => {
            error!("{}", error);
            1
        }
    };
    debug!("Process exiting with code {}", exit_code);
    std::process::exit(exit_code);
}

fn init_logging(config: &cli::Config) -> Result<Option<PathBuf>, String> {
    let log_dir = get_log_dir(config)?;
    let log_file = log_dir.as_ref().map(|dir| dir.join(DAEMON_LOG_FILENAME));

    logging::init_logger(
        config.log_level,
        log_file.as_ref(),
        config.log_stdout_timestamps,
    )
    .map_err(|e| e.display_chain_with_msg("Unable to initialize logger"))?;
    log_panics::init();
    log_version();
    if let Some(ref log_dir) = log_dir {
        info!("Logging to {}", log_dir.display());
    }
    Ok(log_dir)
}

fn get_log_dir(config: &cli::Config) -> Result<Option<PathBuf>, String> {
    if config.log_to_file {
        Ok(Some(mullvad_paths::log_dir().map_err(|e| {
            e.display_chain_with_msg("Unable to get log directory")
        })?))
    } else {
        Ok(None)
    }
}

#[cfg(windows)]
fn run_platform(config: &cli::Config, log_dir: Option<PathBuf>) -> Result<(), String> {
    if config.run_as_service {
        system_service::run()
    } else {
        if config.register_service {
            let install_result = system_service::install_service().map_err(|e| e.display_chain());
            if install_result.is_ok() {
                println!("Installed the service.");
            }
            install_result
        } else {
            run_standalone(log_dir)
        }
    }
}

#[cfg(not(windows))]
fn run_platform(_config: &cli::Config, log_dir: Option<PathBuf>) -> Result<(), String> {
    run_standalone(log_dir)
}

fn run_standalone(log_dir: Option<PathBuf>) -> Result<(), String> {
    if !running_as_admin() {
        warn!("Running daemon as a non-administrator user, clients might refuse to connect");
    }

    let daemon = create_daemon(log_dir)?;

    let shutdown_handle = daemon.shutdown_handle();
    shutdown::set_shutdown_signal_handler(move || shutdown_handle.shutdown())
        .map_err(|e| e.display_chain())?;

    daemon.run().map_err(|e| e.display_chain())?;

    info!("Mullvad daemon is quitting");
    thread::sleep(Duration::from_millis(500));
    Ok(())
}

fn create_daemon(log_dir: Option<PathBuf>) -> Result<Daemon, String> {
    let resource_dir = mullvad_paths::get_resource_dir();
    let cache_dir = mullvad_paths::cache_dir()
        .map_err(|e| e.display_chain_with_msg("Unable to get cache dir"))?;

    Daemon::start(
        log_dir,
        resource_dir,
        cache_dir,
        version::PRODUCT_VERSION.to_owned(),
    )
    .map_err(|e| e.display_chain_with_msg("Unable to initialize daemon"))
}

fn log_version() {
    info!(
        "Starting {} - {} {}",
        env!("CARGO_PKG_NAME"),
        version::PRODUCT_VERSION,
        version::COMMIT_DATE,
    )
}

#[cfg(unix)]
fn running_as_admin() -> bool {
    let uid = unsafe { libc::getuid() };
    uid == 0
}

#[cfg(windows)]
fn running_as_admin() -> bool {
    // TODO: Check if user is administrator correctly on Windows.
    true
}

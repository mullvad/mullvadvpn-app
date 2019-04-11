//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate error_chain;


use error_chain::ChainedError;
use log::{debug, error, info, warn};
use mullvad_daemon::Daemon;
use std::{thread, time::Duration};

mod cli;
mod logging;
mod shutdown;
#[cfg(windows)]
mod system_service;
mod version;

const DAEMON_LOG_FILENAME: &str = "daemon.log";

error_chain! {
    errors {
        LogError(msg: &'static str) {
            description("Error setting up log")
            display("Error setting up log: {}", msg)
        }
    }
    foreign_links {
        DaemonError(mullvad_daemon::Error);
    }
}

fn main() {
    let exit_code = match run() {
        Ok(_) => 0,
        Err(error) => {
            if let ErrorKind::LogError(_) = error.kind() {
                eprintln!("{}", error.display_chain());
            } else {
                error!("{}", error.display_chain());
            }
            1
        }
    };
    debug!("Process exiting with code {}", exit_code);
    ::std::process::exit(exit_code);
}

fn run() -> Result<()> {
    let config = cli::get_config();
    let log_dir = if config.log_to_file {
        Some(
            mullvad_paths::log_dir()
                .chain_err(|| ErrorKind::LogError("Unable to get log directory"))?,
        )
    } else {
        None
    };
    let log_file = log_dir.as_ref().map(|dir| dir.join(DAEMON_LOG_FILENAME));

    logging::init_logger(
        config.log_level,
        log_file.as_ref(),
        config.log_stdout_timestamps,
    )
    .chain_err(|| ErrorKind::LogError("Unable to initialize logger"))?;
    log_panics::init();
    log_version();
    if let Some(ref log_dir) = log_dir {
        info!("Logging to {}", log_dir.display());
    }

    run_platform(&config)
}

#[cfg(windows)]
fn run_platform(config: &cli::Config) -> Result<()> {
    if config.run_as_service {
        system_service::run()
    } else {
        if config.register_service {
            let install_result =
                system_service::install_service().chain_err(|| "Unable to install the service");
            if install_result.is_ok() {
                println!("Installed the service.");
            }
            install_result
        } else {
            run_standalone(config)
        }
    }
}

#[cfg(not(windows))]
fn run_platform(config: &cli::Config) -> Result<()> {
    run_standalone(config)
}

fn run_standalone(config: &cli::Config) -> Result<()> {
    if !running_as_admin() {
        warn!("Running daemon as a non-administrator user, clients might refuse to connect");
    }

    let daemon = create_daemon(config)?;

    let shutdown_handle = daemon.shutdown_handle();
    shutdown::set_shutdown_signal_handler(move || shutdown_handle.shutdown())
        .chain_err(|| "Unable to attach shutdown signal handler")?;

    daemon.run()?;

    info!("Mullvad daemon is quitting");
    thread::sleep(Duration::from_millis(500));
    Ok(())
}

fn create_daemon(config: &cli::Config) -> Result<Daemon> {
    let log_dir = if config.log_to_file {
        Some(mullvad_paths::log_dir().chain_err(|| "Unable to get log directory")?)
    } else {
        None
    };
    let resource_dir = mullvad_paths::get_resource_dir();
    let cache_dir = mullvad_paths::cache_dir().chain_err(|| "Unable to get cache dir")?;

    Daemon::start(
        log_dir,
        resource_dir,
        cache_dir,
        version::PRODUCT_VERSION.to_owned(),
    )
    .chain_err(|| "Unable to initialize daemon")
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

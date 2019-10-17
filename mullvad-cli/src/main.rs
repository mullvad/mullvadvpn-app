//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

#![deny(rust_2018_idioms)]

use clap::{crate_authors, crate_description, crate_name};
use mullvad_ipc_client::{new_standalone_ipc_client, DaemonRpcClient};
use std::io;
use talpid_types::ErrorExt;

mod cmds;
mod location;

pub const PRODUCT_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to connect to daemon")]
    DaemonNotRunning(#[error(source)] io::Error),

    #[error(display = "Can't subscribe to daemon states")]
    CantSubscribe(#[error(source)] mullvad_ipc_client::PubSubError),

    #[error(display = "Failed to communicate with mullvad-daemon over RPC")]
    RpcClientError(#[error(source)] mullvad_ipc_client::Error),

    /// The given command is not correct in some way
    #[error(display = "Invalid command: {}", _0)]
    InvalidCommand(&'static str),
}

pub fn new_rpc_client() -> Result<DaemonRpcClient> {
    match new_standalone_ipc_client(&mullvad_paths::get_rpc_socket_path()) {
        Err(e) => Err(Error::DaemonNotRunning(e)),
        Ok(client) => Ok(client),
    }
}

fn main() {
    let exit_code = match run() {
        Ok(_) => 0,
        Err(error) => {
            eprintln!("{}", error.display_chain());
            1
        }
    };
    std::process::exit(exit_code);
}

fn run() -> Result<()> {
    env_logger::init();

    let commands = cmds::get_commands();

    let app = clap::App::new(crate_name!())
        .version(PRODUCT_VERSION)
        .author(crate_authors!())
        .about(crate_description!())
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .global_settings(&[
            clap::AppSettings::DisableHelpSubcommand,
            clap::AppSettings::VersionlessSubcommands,
        ])
        .subcommands(commands.values().map(|cmd| cmd.clap_subcommand()));

    let app_matches = app.get_matches();
    let (subcommand_name, subcommand_matches) = app_matches.subcommand();
    if let Some(cmd) = commands.get(subcommand_name) {
        cmd.run(subcommand_matches.expect("No command matched"))
    } else {
        unreachable!("No command matched");
    }
}

pub trait Command {
    fn name(&self) -> &'static str;

    fn clap_subcommand(&self) -> clap::App<'static, 'static>;

    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()>;
}

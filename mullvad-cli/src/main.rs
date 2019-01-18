//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

#[macro_use]
extern crate error_chain;

use clap::{crate_authors, crate_description, crate_name};
use error_chain::ChainedError;
use mullvad_ipc_client::{new_standalone_ipc_client, DaemonRpcClient};
use std::io;

mod cmds;

pub const PRODUCT_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));


error_chain! {

    errors {
        DaemonNotRunning(err: mullvad_ipc_client::Error) {
            description("Failed to connect to daemon")
            display("Failed to connect to daemon: {}Is the daemon running?", err.display_chain())
        }
    }

    foreign_links {
        Io(io::Error);
        ParseIntError(::std::num::ParseIntError);
    }

    links {
        RpcClientError(mullvad_ipc_client::Error, mullvad_ipc_client::ErrorKind);
    }
}

pub fn new_rpc_client() -> Result<DaemonRpcClient> {
    match new_standalone_ipc_client(&mullvad_paths::get_rpc_socket_path()) {
        Err(e) => Err(ErrorKind::DaemonNotRunning(e).into()),
        Ok(client) => Ok(client),
    }
}

quick_main!(run);

fn run() -> Result<()> {
    env_logger::init();

    let commands = cmds::get_commands();

    let app = clap::App::new(crate_name!())
        .version(PRODUCT_VERSION)
        .author(crate_authors!())
        .about(crate_description!())
        .setting(clap::AppSettings::SubcommandRequired)
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

    fn run(&self, matches: &clap::ArgMatches) -> Result<()>;
}

#![deny(rust_2018_idioms)]

use clap::{crate_authors, crate_description};
use mullvad_ipc_client::{new_standalone_ipc_client, DaemonRpcClient};
use std::{collections::HashMap, io};
use talpid_types::ErrorExt;

mod cmds;
mod location;

pub const BIN_NAME: &str = "mullvad";
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
    let app = build_cli(&commands);
    let app = app.subcommand(
        clap::SubCommand::with_name("shell-completions")
            .about("Generates completion scripts for your shell")
            .arg(
                clap::Arg::with_name("SHELL")
                    .required(true)
                    .possible_values(&clap::Shell::variants()[..])
                    .help("The shell to generate the script for"),
            )
            .arg(
                clap::Arg::with_name("DIR")
                    .default_value("./")
                    .help("Output directory where the shell completions are written"),
            )
            .setting(clap::AppSettings::Hidden),
    );

    let app_matches = app.get_matches();
    match app_matches.subcommand() {
        ("shell-completions", Some(sub_matches)) => {
            let shell = sub_matches
                .value_of("SHELL")
                .unwrap()
                .parse()
                .expect("Invalid shell");
            let out_dir = sub_matches.value_of_os("DIR").unwrap();
            build_cli(&commands).gen_completions(BIN_NAME, shell, out_dir);
            Ok(())
        }
        (sub_name, Some(sub_matches)) => {
            if let Some(cmd) = commands.get(sub_name) {
                cmd.run(sub_matches)
            } else {
                unreachable!("No command matched");
            }
        }
        (_, None) => {
            unreachable!("No subcommand matches");
        }
    }
}

fn build_cli(commands: &HashMap<&'static str, Box<dyn Command>>) -> clap::App<'static, 'static> {
    clap::App::new(BIN_NAME)
        .version(PRODUCT_VERSION)
        .author(crate_authors!())
        .about(crate_description!())
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .global_settings(&[
            clap::AppSettings::DisableHelpSubcommand,
            clap::AppSettings::VersionlessSubcommands,
        ])
        .subcommands(commands.values().map(|cmd| cmd.clap_subcommand()))
}

pub trait Command {
    fn name(&self) -> &'static str;

    fn clap_subcommand(&self) -> clap::App<'static, 'static>;

    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()>;
}

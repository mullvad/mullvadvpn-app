#![deny(rust_2018_idioms)]

use clap::{crate_authors, crate_description};
#[cfg(all(unix, not(target_os = "android")))]
use clap_complete::{generator::generate_to, Shell};
use mullvad_management_interface::async_trait;
use std::{collections::HashMap, io};
use talpid_types::ErrorExt;

pub use mullvad_management_interface::{self, new_rpc_client};

mod cmds;
mod format;
mod location;
mod state;

pub const BIN_NAME: &str = "mullvad";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to connect to daemon")]
    DaemonNotRunning(#[error(source)] io::Error),

    #[error(display = "Management interface error")]
    ManagementInterfaceError(#[error(source)] mullvad_management_interface::Error),

    #[error(display = "RPC failed")]
    RpcFailed(#[error(source)] mullvad_management_interface::Status),

    #[error(display = "RPC failed: {}", _0)]
    RpcFailedExt(
        &'static str,
        #[error(source)] mullvad_management_interface::Status,
    ),

    /// The given command is not correct in some way
    #[error(display = "Invalid command: {}", _0)]
    InvalidCommand(&'static str),

    #[error(display = "Command failed: {}", _0)]
    CommandFailed(&'static str),

    #[error(display = "Failed to listen for status updates")]
    StatusListenerFailed,

    //#[cfg(all(unix, not(target_os = "android"))
    #[error(display = "Failed to generate shell completions")]
    CompletionsError(#[error(source, no_from)] io::Error),

    #[error(display = "{}", _0)]
    Other(&'static str),
}

#[tokio::main]
async fn main() {
    let exit_code = match run().await {
        Ok(_) => 0,
        Err(error) => {
            match &error {
                Error::RpcFailed(status) => {
                    eprintln!("{}: {:?}: {}", error, status.code(), status.message())
                }
                Error::RpcFailedExt(_message, status) => eprintln!(
                    "{}\nCaused by: {:?}: {}",
                    error,
                    status.code(),
                    status.message()
                ),
                error => eprintln!("{}", error.display_chain()),
            }
            1
        }
    };
    std::process::exit(exit_code);
}

async fn run() -> Result<()> {
    env_logger::init();

    let commands = cmds::get_commands();
    let app = build_cli(&commands);

    #[cfg(all(unix, not(target_os = "android")))]
    let app = app.subcommand(
        clap::App::new("shell-completions")
            .about("Generates completion scripts for your shell")
            .arg(
                clap::Arg::new("SHELL")
                    .required(true)
                    .possible_values(Shell::possible_values())
                    .help("The shell to generate the script for"),
            )
            .arg(
                clap::Arg::new("DIR")
                    .allow_invalid_utf8(true)
                    .default_value("./")
                    .help("Output directory where the shell completions are written"),
            )
            .setting(clap::AppSettings::Hidden),
    );

    let app_matches = app.get_matches();
    match app_matches.subcommand() {
        #[cfg(all(unix, not(target_os = "android")))]
        Some(("shell-completions", sub_matches)) => {
            let shell: Shell = sub_matches
                .value_of("SHELL")
                .unwrap()
                .parse()
                .expect("Invalid shell");
            let out_dir = sub_matches.value_of_os("DIR").unwrap();
            let mut app = build_cli(&commands);
            generate_to(shell, &mut app, BIN_NAME, out_dir)
                .map(|_output_file| ())
                .map_err(Error::CompletionsError)
        }
        Some((sub_name, sub_matches)) => {
            if let Some(cmd) = commands.get(sub_name) {
                cmd.run(sub_matches).await
            } else {
                unreachable!("No command matched");
            }
        }
        _ => {
            unreachable!("No subcommand matches");
        }
    }
}

fn build_cli(commands: &HashMap<&'static str, Box<dyn Command>>) -> clap::App<'static> {
    clap::App::new(BIN_NAME)
        .version(mullvad_version::VERSION)
        .author(crate_authors!())
        .about(crate_description!())
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .global_setting(clap::AppSettings::DisableHelpSubcommand)
        .global_setting(clap::AppSettings::DisableVersionFlag)
        .subcommands(commands.values().map(|cmd| cmd.clap_subcommand()))
}

#[async_trait]
pub trait Command {
    fn name(&self) -> &'static str;

    fn clap_subcommand(&self) -> clap::App<'static>;

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()>;
}

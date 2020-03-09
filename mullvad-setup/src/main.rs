use clap::{crate_authors, crate_description, crate_name, SubCommand};
use std::process;
use talpid_core::firewall::{self, Firewall, FirewallArguments};
use talpid_types::ErrorExt;

pub const PRODUCT_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Firewall error")]
    FirewallError(#[error(source)] firewall::Error),
}

fn main() {
    env_logger::init();

    let subcommands = vec![
        SubCommand::with_name("prepare-restart")
            .about("Move a running daemon into a blocking state and save its target state"),
        SubCommand::with_name("reset-firewall")
            .about("Remove any firewall rules introduced by the daemon"),
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
        "prepare-restart" => prepare_restart(),
        "reset-firewall" => reset_firewall(),
        _ => unreachable!("No command matched"),
    };

    if let Err(e) = result {
        eprintln!("{}", e.display_chain());
        process::exit(1);
    }
}

fn prepare_restart() -> Result<(), Error> {
    Ok(())
}

fn reset_firewall() -> Result<(), Error> {
    // TODO: ensure daemon isn't running

    let mut firewall = Firewall::new(FirewallArguments {
        initialize_blocked: false,
        allow_lan: None,
    }).map_err(Error::FirewallError)?;

    firewall.reset_policy().map_err(Error::FirewallError)
}

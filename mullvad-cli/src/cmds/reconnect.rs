use crate::{new_rpc_client, Command, Result};
use talpid_types::ErrorExt;

pub struct Reconnect;

impl Command for Reconnect {
    fn name(&self) -> &'static str {
        "reconnect"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name()).about("Command the client to reconnect")
    }

    fn run(&self, _matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        if let Err(e) = rpc.reconnect() {
            eprintln!("{}", e.display_chain());
        }
        Ok(())
    }
}

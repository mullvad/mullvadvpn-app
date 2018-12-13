use crate::{new_rpc_client, Command, Result};

pub struct Disconnect;

impl Command for Disconnect {
    fn name(&self) -> &'static str {
        "disconnect"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Command the client to disconnect the VPN tunnel")
    }

    fn run(&self, _matches: &clap::ArgMatches) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        rpc.disconnect()?;
        Ok(())
    }
}

use clap;
use Command;
use Result;

use mullvad_ipc_client::new_standalone_ipc_client;

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
        let mut rpc = new_standalone_ipc_client()?;
        rpc.disconnect()?;
        Ok(())
    }
}

use clap;
use Command;
use Result;

use mullvad_ipc_client::DaemonRpcClient;

pub struct Connect;

impl Command for Connect {
    fn name(&self) -> &'static str {
        "connect"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Command the client to start establishing a VPN tunnel")
    }

    fn run(&self, _matches: &clap::ArgMatches) -> Result<()> {
        let mut rpc = DaemonRpcClient::new()?;
        rpc.connect()?;
        Ok(())
    }
}

use crate::{new_rpc_client, Command, Result};
use talpid_types::ErrorExt;

pub struct Reconnect;

#[mullvad_management_interface::async_trait]
impl Command for Reconnect {
    fn name(&self) -> &'static str {
        "reconnect"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name()).about("Command the client to reconnect")
    }

    async fn run(&self, _: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        if let Err(e) = rpc.reconnect_tunnel(()).await {
            eprintln!("{}", e.display_chain());
        }
        Ok(())
    }
}

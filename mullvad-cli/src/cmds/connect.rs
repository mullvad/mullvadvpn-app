use crate::{new_grpc_client, Command, Result};
use talpid_types::ErrorExt;

pub struct Connect;

#[async_trait::async_trait]
impl Command for Connect {
    fn name(&self) -> &'static str {
        "connect"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Command the client to start establishing a VPN tunnel")
    }

    async fn run(&self, _: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        if let Err(e) = rpc.connect_daemon(()).await {
            eprintln!("{}", e.display_chain());
        }
        Ok(())
    }
}

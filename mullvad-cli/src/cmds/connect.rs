use crate::{format, new_rpc_client, state, Command, Result};
use mullvad_management_interface::types::tunnel_state::State::{Connected, Error};
use talpid_types::ErrorExt;

pub struct Connect;

#[mullvad_management_interface::async_trait]
impl Command for Connect {
    fn name(&self) -> &'static str {
        "connect"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Command the client to start establishing a VPN tunnel")
            .arg(
                clap::Arg::with_name("wait")
                    .long("wait")
                    .short("w")
                    .help("Wait until connected before exiting"),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_rpc_client().await?;

        let status_listen_handle = if matches.is_present("wait") {
            state::state_listen(
                &mut rpc,
                |state| match state {
                    Connected(_) => false,
                    _ => true,
                },
                Box::new(|state| match state {
                    Connected(_) => false,
                    Error(_) => false,
                    _ => true,
                }),
                Box::new(format::print_state),
            )
            .await?
        } else {
            None
        };

        if let Err(e) = rpc.connect_tunnel(()).await {
            eprintln!("{}", e.display_chain());
        }
        if let Some(handle) = status_listen_handle {
            handle.await.expect("Failed to listen to status updates");
        }

        Ok(())
    }
}

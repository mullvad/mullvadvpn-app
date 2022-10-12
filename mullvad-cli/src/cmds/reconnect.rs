use crate::{format, new_rpc_client, state, Command, Error, Result};
use futures::StreamExt;
use mullvad_types::states::TunnelState;

pub struct Reconnect;

#[mullvad_management_interface::async_trait]
impl Command for Reconnect {
    fn name(&self) -> &'static str {
        "reconnect"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about("Command the client to reconnect")
            .arg(
                clap::Arg::new("wait")
                    .long("wait")
                    .short('w')
                    .help("Wait until reconnected before exiting"),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        let mut rpc = new_rpc_client().await?;

        let receiver_option = if matches.is_present("wait") {
            Some(state::state_listen(rpc.clone()))
        } else {
            None
        };

        if rpc.reconnect_tunnel(()).await?.into_inner() {
            if let Some(mut receiver) = receiver_option {
                while let Some(state) = receiver.next().await {
                    let state = state?;
                    format::print_state(&state, false);
                    match state {
                        TunnelState::Connected { .. } => return Ok(()),
                        TunnelState::Error { .. } => return Err(Error::CommandFailed("reconnect")),
                        _ => {}
                    }
                }
                return Err(Error::StatusListenerFailed);
            }
        }

        Ok(())
    }
}

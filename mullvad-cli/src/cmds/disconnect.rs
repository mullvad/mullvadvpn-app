use crate::{format, new_rpc_client, state, Command, Error, Result};
use futures::StreamExt;
use mullvad_management_interface::types::tunnel_state::State::Disconnected;

pub struct Disconnect;

#[mullvad_management_interface::async_trait]
impl Command for Disconnect {
    fn name(&self) -> &'static str {
        "disconnect"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about("Command the client to disconnect the VPN tunnel")
            .arg(
                clap::Arg::new("wait")
                    .long("wait")
                    .short('w')
                    .help("Wait until disconnected before exiting"),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        let mut rpc = new_rpc_client().await?;

        let receiver_option = if matches.is_present("wait") {
            Some(state::state_listen(rpc.clone()))
        } else {
            None
        };

        if rpc.disconnect_tunnel(()).await?.into_inner() {
            if let Some(mut receiver) = receiver_option {
                while let Some(state) = receiver.next().await {
                    let state = state?;
                    format::print_state(&state);
                    match state.state.unwrap() {
                        Disconnected(_) => return Ok(()),
                        _ => {}
                    }
                }
                return Err(Error::StatusListenerFailed);
            }
        }

        Ok(())
    }
}

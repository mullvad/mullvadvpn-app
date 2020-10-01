use crate::{new_rpc_client, state, Command, Error, Result};
use mullvad_management_interface::types::tunnel_state::State;

pub struct Reconnect;

#[mullvad_management_interface::async_trait]
impl Command for Reconnect {
    fn name(&self) -> &'static str {
        "reconnect"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Command the client to reconnect")
            .arg(
                clap::Arg::with_name("wait")
                    .long("wait")
                    .short("w")
                    .help("Wait until reconnected before exiting"),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_rpc_client().await?;

        let status_listen_handle = if matches.is_present("wait") {
            Some(
                state::state_listen(&mut rpc, |state| match state {
                    State::Connected(_) => Ok(false),
                    State::Error(_) => Err(Error::CommandFailed("connect")),
                    _ => Ok(true),
                })
                .await?,
            )
        } else {
            None
        };

        if rpc.reconnect_tunnel(()).await?.into_inner() {
            if let Some(handle) = status_listen_handle {
                handle.await.expect("Failed to listen to status updates")?;
            }
        }

        Ok(())
    }
}

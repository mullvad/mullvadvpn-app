use Command;
use Result;
use clap;

use mullvad_types::states::{DaemonState, SecurityState, TargetState};
use rpc;

pub struct Status;

impl Command for Status {
    fn name(&self) -> &'static str {
        "status"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name()).about("View the state of the VPN tunnel")
    }

    fn run(&self, _matches: &clap::ArgMatches) -> Result<()> {
        let state: DaemonState = rpc::call("get_state", &[] as &[u8; 0])?;
        match (state.state, state.target_state) {
            (SecurityState::Unsecured, TargetState::Unsecured) => println!("Disconnected"),
            (SecurityState::Unsecured, TargetState::Secured) => println!("Connecting..."),
            (SecurityState::Secured, TargetState::Unsecured) => println!("Disconnecting..."),
            (SecurityState::Secured, TargetState::Secured) => println!("Connected"),
        }
        Ok(())
    }
}

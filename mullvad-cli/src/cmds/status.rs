use Command;
use Result;
use clap;

use mullvad_types::location::Location;
use mullvad_types::states::{DaemonState, SecurityState, TargetState};
use rpc;

use std::net::IpAddr;

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
        print!("Tunnel status: ");
        match (state.state, state.target_state) {
            (SecurityState::Unsecured, TargetState::Unsecured) => println!("Disconnected"),
            (SecurityState::Unsecured, TargetState::Secured) => println!("Connecting..."),
            (SecurityState::Secured, TargetState::Unsecured) => println!("Disconnecting..."),
            (SecurityState::Secured, TargetState::Secured) => println!("Connected"),
        }

        let location: Location = rpc::call("get_current_location", &[] as &[u8; 0])?;
        println!("Location: {}, {}", location.city, location.country);
        println!(
            "Position: {:.5}°N, {:.5}°W",
            location.position[0], location.position[1]
        );

        let ip: IpAddr = rpc::call("get_public_ip", &[] as &[u8; 0])?;
        println!("IP: {}", ip);
        Ok(())
    }
}

use clap;
use Command;
use Result;

use mullvad_ipc_client::DaemonRpcClient;
use mullvad_types::states::{SecurityState, TargetState};

pub struct Status;

impl Command for Status {
    fn name(&self) -> &'static str {
        "status"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name()).about("View the state of the VPN tunnel")
    }

    fn run(&self, _matches: &clap::ArgMatches) -> Result<()> {
        let mut rpc = DaemonRpcClient::new()?;
        let state = rpc.get_state()?;
        print!("Tunnel status: ");
        match (state.state, state.target_state) {
            (SecurityState::Unsecured, TargetState::Unsecured) => println!("Disconnected"),
            (SecurityState::Unsecured, TargetState::Secured) => println!("Connecting..."),
            (SecurityState::Secured, TargetState::Unsecured) => println!("Disconnecting..."),
            (SecurityState::Secured, TargetState::Secured) => println!("Connected"),
        }

        let location = rpc.get_current_location()?;
        let city_and_country = if let Some(city) = location.city {
            format!("{}, {}", city, location.country)
        } else {
            format!("{}", location.country)
        };
        println!("Location: {}", city_and_country);
        println!(
            "Position: {:.5}°N, {:.5}°W",
            location.latitude, location.longitude
        );
        println!("IP: {}", location.ip);
        Ok(())
    }
}

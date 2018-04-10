use clap;
use Command;
use Result;

use mullvad_types::location::GeoIpLocation;
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
        print!("Tunnel status: ");
        match (state.state, state.target_state) {
            (SecurityState::Unsecured, TargetState::Unsecured) => println!("Disconnected"),
            (SecurityState::Unsecured, TargetState::Secured) => println!("Connecting..."),
            (SecurityState::Secured, TargetState::Unsecured) => println!("Disconnecting..."),
            (SecurityState::Secured, TargetState::Secured) => println!("Connected"),
        }

        let location: GeoIpLocation = rpc::call("get_current_location", &[] as &[u8; 0])?;
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

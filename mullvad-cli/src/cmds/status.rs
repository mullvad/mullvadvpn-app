use clap;
use new_rpc_client;
use Command;
use Result;

use mullvad_ipc_client::DaemonRpcClient;
use mullvad_types::states::{DaemonState, SecurityState, TargetState};

const DISCONNECTED: DaemonState = DaemonState {
    state: SecurityState::Unsecured,
    target_state: TargetState::Unsecured,
};

const CONNECTED: DaemonState = DaemonState {
    state: SecurityState::Secured,
    target_state: TargetState::Secured,
};

pub struct Status;

impl Command for Status {
    fn name(&self) -> &'static str {
        "status"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("View the state of the VPN tunnel")
            .subcommand(
                clap::SubCommand::with_name("listen").about("Listen for VPN tunnel state changes"),
            )
    }

    fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        let state = rpc.get_state()?;

        print_state(state);
        print_location(&mut rpc)?;
        if matches.subcommand_matches("listen").is_some() {
            for new_state in rpc.new_state_subscribe()? {
                print_state(new_state);

                if new_state == CONNECTED || new_state == DISCONNECTED {
                    print_location(&mut rpc)?;
                }
            }
        }
        Ok(())
    }
}

fn print_state(state: DaemonState) {
    print!("Tunnel status: ");
    match (state.state, state.target_state) {
        (SecurityState::Unsecured, TargetState::Unsecured) => println!("Disconnected"),
        (SecurityState::Unsecured, TargetState::Secured) => println!("Connecting..."),
        (SecurityState::Secured, TargetState::Unsecured) => println!("Disconnecting..."),
        (SecurityState::Secured, TargetState::Secured) => println!("Connected"),
    }
}

fn print_location(rpc: &mut DaemonRpcClient) -> Result<()> {
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

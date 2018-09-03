use clap;
use new_rpc_client;
use Command;
use Result;

use mullvad_ipc_client::DaemonRpcClient;
use talpid_types::tunnel::TunnelStateTransition::{self, *};

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

        print_state(&state);
        print_location(&mut rpc)?;
        if matches.subcommand_matches("listen").is_some() {
            for new_state in rpc.new_state_subscribe()? {
                print_state(&new_state);

                if new_state == Connected || new_state == Disconnected {
                    print_location(&mut rpc)?;
                }
            }
        }
        Ok(())
    }
}

fn print_state(state: &TunnelStateTransition) {
    print!("Tunnel status: ");
    match state {
        Blocked(reason) => println!("Blocked ({})", reason),
        Connected => println!("Connected"),
        Connecting => println!("Connecting..."),
        Disconnected => println!("Disconnected"),
        Disconnecting => println!("Disconnecting..."),
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

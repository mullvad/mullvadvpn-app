use crate::{new_rpc_client, Command, Result};
use mullvad_ipc_client::DaemonRpcClient;
use mullvad_types::auth_failed::AuthFailed;
use talpid_types::tunnel::{BlockReason, TunnelStateTransition};

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

                use self::TunnelStateTransition::*;
                match new_state {
                    Connected(_) | Disconnected => print_location(&mut rpc)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

fn print_state(state: &TunnelStateTransition) {
    use self::TunnelStateTransition::*;
    print!("Tunnel status: ");
    match state {
        Blocked(reason) => print_blocked_reason(reason),
        Connected(_) => println!("Connected"),
        Connecting(_) => println!("Connecting..."),
        Disconnected => println!("Disconnected"),
        Disconnecting(_) => println!("Disconnecting..."),
    }
}

fn print_blocked_reason(reason: &BlockReason) {
    match reason {
        BlockReason::AuthFailed(ref auth_failure) => {
            let auth_failure_str = auth_failure
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("Account authentication failed");
            println!("Blocked: {}", AuthFailed::from(auth_failure_str));
        }
        other => println!("Blocked: {}", other),
    }
}

fn print_location(rpc: &mut DaemonRpcClient) -> Result<()> {
    let location = match rpc.get_current_location()? {
        Some(loc) => loc,
        None => {
            println!("Location data unavailable");
            return Ok(());
        }
    };
    let city_and_country = if let Some(city) = location.city {
        format!("{}, {}", city, location.country)
    } else {
        format!("{}", location.country)
    };
    if let Some(hostname) = location.hostname {
        println!("Relay: {}", hostname);
    }
    println!("Location: {}", city_and_country);
    println!(
        "Position: {:.5}°N, {:.5}°W",
        location.latitude, location.longitude
    );
    Ok(())
}

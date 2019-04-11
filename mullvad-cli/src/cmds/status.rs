use crate::{new_rpc_client, Command, Error, Result};
use futures::{Future, Stream};
use mullvad_ipc_client::DaemonRpcClient;
use mullvad_types::{auth_failed::AuthFailed, DaemonEvent};
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
                clap::SubCommand::with_name("listen")
                    .about("Listen for VPN tunnel state changes")
                    .arg(
                        clap::Arg::with_name("verbose")
                            .short("v")
                            .help("Enables verbose output"),
                    ),
            )
    }

    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        let state = rpc.get_state()?;

        print_state(&state);
        print_location(&mut rpc)?;
        if let Some(listen_matches) = matches.subcommand_matches("listen") {
            let verbose = listen_matches.is_present("verbose");
            let subscription = rpc
                .daemon_event_subscribe()
                .wait()
                .map_err(Error::CantSubscribe)?;
            for event in subscription.wait() {
                match event? {
                    DaemonEvent::StateTransition(new_state) => {
                        print_state(&new_state);
                        use self::TunnelStateTransition::*;
                        match new_state {
                            Connected(_) | Disconnected => print_location(&mut rpc)?,
                            _ => {}
                        }
                    }
                    DaemonEvent::Settings(settings) => {
                        if verbose {
                            println!("New settings: {:#?}", settings);
                        }
                    }
                    DaemonEvent::RelayList(relay_list) => {
                        if verbose {
                            println!("New relay list: {:#?}", relay_list);
                        }
                    }
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
    if let Some(hostname) = location.hostname {
        println!("Relay: {}", hostname);
    }
    if let Some(ipv4) = location.ipv4 {
        println!("IPv4: {}", ipv4);
    }
    if let Some(ipv6) = location.ipv6 {
        println!("IPv6: {}", ipv6);
    }

    print!("Location: ");
    if let Some(city) = location.city {
        print!("{}, ", city);
    }
    println!("{}", location.country);

    println!(
        "Position: {:.5}°N, {:.5}°W",
        location.latitude, location.longitude
    );
    Ok(())
}

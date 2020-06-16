use crate::{new_rpc_client, Command, Error, Result};
use futures::{Future, Stream};
use mullvad_ipc_client::DaemonRpcClient;
use mullvad_types::{auth_failed::AuthFailed, states::TunnelState, DaemonEvent};
use talpid_types::tunnel::{ErrorState, ErrorStateCause};

pub struct Status;

#[async_trait::async_trait]
impl Command for Status {
    fn name(&self) -> &'static str {
        "status"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("View the state of the VPN tunnel")
            .arg(
                clap::Arg::with_name("location")
                    .long("location")
                    .short("l")
                    .help("Prints the current location and IP. Based on GeoIP lookups"),
            )
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

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        let state = rpc.get_state()?;

        print_state(&state);
        if matches.is_present("location") {
            print_location(&mut rpc)?;
        }

        if let Some(listen_matches) = matches.subcommand_matches("listen") {
            let verbose = listen_matches.is_present("verbose");
            let subscription = rpc
                .daemon_event_subscribe()
                .wait()
                .map_err(Error::CantSubscribe)?;
            for event in subscription.wait() {
                match event? {
                    DaemonEvent::TunnelState(new_state) => {
                        print_state(&new_state);
                        use self::TunnelState::*;
                        match new_state {
                            Connected { .. } | Disconnected => {
                                if matches.is_present("location") {
                                    print_location(&mut rpc)?;
                                }
                            }
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
                    DaemonEvent::AppVersionInfo(app_version_info) => {
                        if verbose {
                            println!("New app version info: {:#?}", app_version_info);
                        }
                    }
                    DaemonEvent::WireguardKey(key_event) => {
                        if verbose {
                            println!("{}", key_event);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn print_state(state: &TunnelState) {
    use self::TunnelState::*;
    print!("Tunnel status: ");
    match state {
        Error(reason) => print_error_state(reason),
        Connected { endpoint, .. } => {
            println!("Connected to {}", endpoint);
        }
        Connecting { endpoint, .. } => println!("Connecting to {}...", endpoint),
        Disconnected => println!("Disconnected"),
        Disconnecting(_) => println!("Disconnecting..."),
    }
}

fn print_error_state(error_state: &ErrorState) {
    if !error_state.is_blocking() {
        eprintln!("Mullvad daemon failed to setup firewall rules!");
        eprintln!("Deamon cannot block traffic from flowing, non-local traffic will leak");
    }

    print_blocked_reason(error_state.cause());
}

fn print_blocked_reason(reason: &ErrorStateCause) {
    match reason {
        ErrorStateCause::AuthFailed(ref auth_failure) => {
            let auth_failure_str = auth_failure
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("Account authentication failed");
            println!("Blocked: {}", AuthFailed::from(auth_failure_str));
        }
        #[cfg(target_os = "linux")]
        ErrorStateCause::SetFirewallPolicyError(error) => {
            println!(
                "Blocked: {}",
                ErrorStateCause::SetFirewallPolicyError(error.clone())
            );
            println!("Your kernel might be terribly out of date or missing nftables");
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

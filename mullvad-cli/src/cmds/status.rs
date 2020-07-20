use crate::{new_grpc_client, proto, Command, Error, Result};
use mullvad_types::auth_failed::AuthFailed;
use proto::management_service_client::ManagementServiceClient;
use proto::{error_state::{Cause as ErrorStateCause, GenerationError}, daemon_event::Event as EventType, ErrorState, TunnelState};

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
        let mut rpc = new_grpc_client().await?;
        let state = rpc.get_state(()).await.map_err(Error::GrpcClientError)?.into_inner();

        print_state(&state);
        if matches.is_present("location") {
            print_location(&mut rpc).await?;
        }

        if let Some(listen_matches) = matches.subcommand_matches("listen") {
            let verbose = listen_matches.is_present("verbose");

            let mut events = rpc.events_listen(())
                .await
                .map_err(Error::GrpcClientError)?
                .into_inner();

            while let Some(event) = events.message().await? {
                // TODO: fix formatting
                match event.event.unwrap() {
                    EventType::TunnelState(new_state) => {
                        print_state(&new_state);
                        use proto::tunnel_state::State::*;
                        match new_state.state.unwrap() {
                            Connected(..) | Disconnected(..) => {
                                if matches.is_present("location") {
                                    print_location(&mut rpc).await?;
                                }
                            }
                            _ => {}
                        }
                    }
                    EventType::Settings(settings) => {
                        if verbose {
                            println!("New settings: {:#?}", settings);
                        }
                    }
                    EventType::RelayList(relay_list) => {
                        if verbose {
                            println!("New relay list: {:#?}", relay_list);
                        }
                    }
                    EventType::VersionInfo(app_version_info) => {
                        if verbose {
                            println!("New app version info: {:#?}", app_version_info);
                        }
                    }
                    EventType::KeyEvent(key_event) => {
                        if verbose {
                            println!("{:#?}", key_event);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn print_state(state: &TunnelState) {
    // TODO: fix formatting
    use proto::tunnel_state;
    use proto::tunnel_state::State::*;

    print!("Tunnel status: ");
    match state.state.as_ref().unwrap() {
        Error(error) => print_error_state(error.error_state.as_ref().unwrap()),
        Connected(tunnel_state::Connected { relay_info }) => {
            // TODO: compare output

            let endpoint = relay_info.as_ref().unwrap().tunnel_endpoint.as_ref().unwrap();
            println!(
                "Connected to {} {} over {}",
                // TODO: as string
                endpoint.tunnel_type,
                endpoint.address,
                // TODO: as string
                endpoint.protocol,
            );

            // TODO: optional proxy endpoint
            /*
            if let Some(ref proxy) = self.proxy {
                write!(
                    f,
                    " via {} {} over {}",
                    proxy.proxy_type, proxy.endpoint.address, proxy.endpoint.protocol
                )?;
            }
            */
        }
        Connecting(tunnel_state::Connecting { relay_info }) => {
            let endpoint = relay_info.as_ref().unwrap().tunnel_endpoint.as_ref().unwrap();
            println!("Connecting to {:?}...", endpoint);
        }
        Disconnected(_) => println!("Disconnected"),
        Disconnecting(_) => println!("Disconnecting..."),
    }
}

fn print_error_state(error_state: &ErrorState) {
    if !error_state.is_blocking {
        eprintln!("Mullvad daemon failed to setup firewall rules!");
        eprintln!("Deamon cannot block traffic from flowing, non-local traffic will leak");
    }

    match error_state.cause {
        x if x == ErrorStateCause::AuthFailed as i32 => {
            println!("Blocked: {}", AuthFailed::from(error_state.auth_fail_reason.as_ref()));
        }
        #[cfg(target_os = "linux")]
        cause if cause == ErrorStateCause::SetFirewallPolicyError as i32 => {
            println!("Blocked: {}", error_state_to_string(error_state));
            println!("Your kernel might be terribly out of date or missing nftables");
        }
        _ => println!("Blocked: {}", error_state_to_string(error_state)),
    }
}

fn error_state_to_string(error_state: &ErrorState) -> String {
    use ErrorStateCause::*;

    let error_str = match error_state.cause {
        x if x == AuthFailed as i32 => {
            // TODO: format correctly?
            return if error_state.auth_fail_reason.is_empty() {
                "Authentication with remote server failed".to_string()
            } else {
                format!("Authentication with remote server failed: {}", error_state.auth_fail_reason)
            };
        }
        x if x == Ipv6Unavailable as i32 => "Failed to configure IPv6 because it's disabled in the platform",
        x if x == SetFirewallPolicyError as i32 => "Failed to set firewall policy",
        x if x == SetDnsError as i32 => "Failed to set system DNS server",
        x if x == StartTunnelError as i32 => "Failed to start connection to remote server",
        x if x == TunnelParameterError as i32 => {
            return format!("Failure to generate tunnel parameters: {}", tunnel_parameter_error_to_string(
                error_state.parameter_error
            ));
        }
        x if x == IsOffline as i32 => "This device is offline, no tunnels can be established",
        x if x == TapAdapterProblem as i32 => "A problem with the TAP adapter has been detected",
        #[cfg(target_os = "android")]
        x if x == VpnPermissionDenied as i32 => "The Android VPN permission was denied when creating the tunnel",
        _ => unreachable!("unknown error state cause"),
    };

    error_str.to_string()
}

fn tunnel_parameter_error_to_string(parameter_error: i32) -> &'static str {
    match parameter_error {
        x if x == GenerationError::NoMatchingRelay as i32 => "Failure to select a matching tunnel relay",
        x if x == GenerationError::NoMatchingBridgeRelay as i32 => "Failure to select a matching bridge relay",
        x if x == GenerationError::NoWireguardKey as i32 => "No wireguard key available",
        x if x == GenerationError::CustomTunnelHostResolutionError as i32 => "Can't resolve hostname for custom tunnel host",
        _ => unreachable!("unknown tunnel parameter error"),
    }
}

async fn print_location(rpc: &mut ManagementServiceClient<tonic::transport::Channel>) -> Result<()> {
    // TODO: RPC should return an optional location
    let location = rpc.get_current_location(()).await.map_err(Error::GrpcClientError)?.into_inner();
    if !location.hostname.is_empty() {
        println!("Relay: {}", location.hostname);
    }
    if !location.ipv4.is_empty() {
        println!("IPv4: {}", location.ipv4);
    }
    if !location.ipv6.is_empty() {
        println!("IPv6: {}", location.ipv6);
    }

    print!("Location: ");
    if !location.city.is_empty() {
        print!("{}, ", location.city);
    }
    println!("{}", location.country);

    println!(
        "Position: {:.5}°N, {:.5}°W",
        location.latitude, location.longitude
    );
    Ok(())
}

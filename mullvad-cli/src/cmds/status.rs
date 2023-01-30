use crate::{format, new_rpc_client, Command, Error, Result};
use mullvad_management_interface::{
    types::daemon_event::Event as EventType, ManagementServiceClient,
};
use mullvad_types::{location::GeoIpLocation, states::TunnelState};

pub struct Status;

#[mullvad_management_interface::async_trait]
impl Command for Status {
    fn name(&self) -> &'static str {
        "status"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about("View the state of the VPN tunnel")
            .arg(
                clap::Arg::new("verbose")
                    .short('v')
                    .help("Enables verbose output"),
            )
            .arg(
                clap::Arg::new("location")
                    .long("location")
                    .short('l')
                    .help("Prints the current location and IP. Based on GeoIP lookups"),
            )
            .arg(
                clap::Arg::new("debug")
                    .long("debug")
                    .global(true)
                    .help("Enables debug output"),
            )
            .subcommand(clap::App::new("listen").about("Listen for VPN tunnel state changes"))
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        let debug = matches.is_present("debug");
        let verbose = matches.is_present("verbose");
        let show_full_location = matches.is_present("location");

        let mut rpc = new_rpc_client().await?;
        let state = rpc.get_tunnel_state(()).await?.into_inner();

        if debug {
            println!("Tunnel state: {state:#?}");
        } else {
            let state = TunnelState::try_from(state).expect("invalid tunnel state");
            format::print_state(&state, verbose);
        }

        if show_full_location {
            print_location(&mut rpc).await?;
        }

        if matches.subcommand_matches("listen").is_some() {
            let mut events = rpc.events_listen(()).await?.into_inner();

            while let Some(event) = events.message().await? {
                match event.event.unwrap() {
                    EventType::TunnelState(new_state) => {
                        let new_state =
                            TunnelState::try_from(new_state).expect("invalid tunnel state");

                        if debug {
                            println!("New tunnel state: {new_state:#?}");
                        } else {
                            format::print_state(&new_state, verbose);
                        }

                        match new_state {
                            TunnelState::Connected { .. } | TunnelState::Disconnected => {
                                if show_full_location {
                                    print_location(&mut rpc).await?;
                                }
                            }
                            _ => {}
                        }
                    }
                    EventType::Settings(settings) => {
                        if debug {
                            println!("New settings: {settings:#?}");
                        }
                    }
                    EventType::RelayList(relay_list) => {
                        if debug {
                            println!("New relay list: {relay_list:#?}");
                        }
                    }
                    EventType::VersionInfo(app_version_info) => {
                        if debug {
                            println!("New app version info: {app_version_info:#?}");
                        }
                    }
                    EventType::Device(device) => {
                        if debug {
                            println!("Device event: {device:#?}");
                        }
                    }
                    EventType::RemoveDevice(device) => {
                        if debug {
                            println!("Remove device event: {device:#?}");
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

async fn print_location(rpc: &mut ManagementServiceClient) -> Result<()> {
    let location = match rpc.get_current_location(()).await {
        Ok(response) => GeoIpLocation::try_from(response.into_inner()).expect("invalid geoip data"),
        Err(status) => {
            if status.code() == mullvad_management_interface::Code::NotFound {
                println!("Location data unavailable");
                return Ok(());
            } else {
                return Err(Error::RpcFailed(status));
            }
        }
    };
    if let Some(ipv4) = location.ipv4 {
        println!("IPv4: {ipv4}");
    }
    if let Some(ipv6) = location.ipv6 {
        println!("IPv6: {ipv6}");
    }

    println!(
        "Position: {:.5}°N, {:.5}°W",
        location.latitude, location.longitude
    );
    Ok(())
}

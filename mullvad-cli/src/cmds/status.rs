use crate::{format, Command, Error, MullvadProxyClient, Result};
use futures::StreamExt;
use mullvad_management_interface::client::DaemonEvent;
use mullvad_types::states::TunnelState;

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

        let mut rpc = MullvadProxyClient::new().await?;
        let state = rpc.get_tunnel_state().await?;

        if debug {
            println!("Tunnel state: {state:#?}");
        } else {
            format::print_state(&state, verbose);
        }

        if show_full_location {
            print_location(&mut rpc).await?;
        }

        if matches.subcommand_matches("listen").is_some() {
            while let Some(event) = rpc.events_listen().await?.next().await {
                match event? {
                    DaemonEvent::TunnelState(new_state) => {
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
                    DaemonEvent::Settings(settings) => {
                        if debug {
                            println!("New settings: {settings:#?}");
                        }
                    }
                    DaemonEvent::RelayList(relay_list) => {
                        if debug {
                            println!("New relay list: {relay_list:#?}");
                        }
                    }
                    DaemonEvent::AppVersionInfo(app_version_info) => {
                        if debug {
                            println!("New app version info: {app_version_info:#?}");
                        }
                    }
                    DaemonEvent::Device(device) => {
                        if debug {
                            println!("Device event: {device:#?}");
                        }
                    }
                    DaemonEvent::RemoveDevice(device) => {
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

async fn print_location(rpc: &mut MullvadProxyClient) -> Result<()> {
    let location = match rpc.get_current_location().await {
        Ok(location) => location,
        Err(error) => match &error {
            mullvad_management_interface::Error::NoLocationData => {
                println!("Location data unavailable");
                return Ok(());
            }
            _ => return Err(Error::ManagementInterfaceError(error)),
        },
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

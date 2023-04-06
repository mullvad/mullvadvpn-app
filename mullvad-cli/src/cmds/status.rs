use anyhow::Result;
use clap::{Args, Subcommand};
use futures::StreamExt;
use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_types::states::TunnelState;

use crate::format;

#[derive(Subcommand, Debug, PartialEq)]
pub enum Status {
    /// Listen for tunnel state changes
    Listen,
}

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Enable verbose output
    #[arg(long, short = 'v')]
    verbose: bool,

    /// Print the current location and IP, based on GeoIP lookups
    #[arg(long, short = 'l')]
    location: bool,

    /// Enable debug output
    #[arg(long, short = 'd')]
    debug: bool,
}

impl Status {
    pub async fn listen(mut rpc: MullvadProxyClient, args: StatusArgs) -> Result<()> {
        while let Some(event) = rpc.events_listen().await?.next().await {
            match event? {
                DaemonEvent::TunnelState(new_state) => {
                    if args.debug {
                        println!("New tunnel state: {new_state:#?}");
                    } else {
                        format::print_state(&new_state, args.verbose);
                    }

                    match new_state {
                        TunnelState::Connected { .. } | TunnelState::Disconnected => {
                            if args.location {
                                print_location(&mut rpc).await?;
                            }
                        }
                        _ => {}
                    }
                }
                DaemonEvent::Settings(settings) => {
                    if args.debug {
                        println!("New settings: {settings:#?}");
                    }
                }
                DaemonEvent::RelayList(relay_list) => {
                    if args.debug {
                        println!("New relay list: {relay_list:#?}");
                    }
                }
                DaemonEvent::AppVersionInfo(app_version_info) => {
                    if args.debug {
                        println!("New app version info: {app_version_info:#?}");
                    }
                }
                DaemonEvent::Device(device) => {
                    if args.debug {
                        println!("Device event: {device:#?}");
                    }
                }
                DaemonEvent::RemoveDevice(device) => {
                    if args.debug {
                        println!("Remove device event: {device:#?}");
                    }
                }
            }
        }
        Ok(())
    }
}

pub async fn handle(cmd: Option<Status>, args: StatusArgs) -> Result<()> {
    let mut rpc = MullvadProxyClient::new().await?;
    let state = rpc.get_tunnel_state().await?;

    if args.debug {
        println!("Tunnel state: {state:#?}");
    } else {
        format::print_state(&state, args.verbose);
    }

    if args.location {
        print_location(&mut rpc).await?;
    }

    if cmd == Some(Status::Listen) {
        Status::listen(rpc, args).await?;
    }
    Ok(())
}

async fn print_location(rpc: &mut MullvadProxyClient) -> Result<()> {
    let location = match rpc.get_current_location().await {
        Ok(location) => location,
        Err(error) => match &error {
            mullvad_management_interface::Error::NoLocationData => {
                println!("Location data unavailable");
                return Ok(());
            }
            _ => return Err(error.into()),
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

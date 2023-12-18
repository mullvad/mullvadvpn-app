use anyhow::Result;
use clap::{Args, Subcommand};
use futures::StreamExt;
use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_types::{device::DeviceState, states::TunnelState};

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

    // TODO: changelog about removing location flag
    /// Enable debug output
    #[arg(long, short = 'd')]
    debug: bool,
}

impl Status {
    pub async fn listen(mut rpc: MullvadProxyClient, args: StatusArgs) -> Result<()> {
        let mut previous_tunnel_state = None;

        while let Some(event) = rpc.events_listen().await?.next().await {
            match event? {
                DaemonEvent::TunnelState(new_state) => {
                    if args.debug {
                        println!("New tunnel state: {new_state:#?}");
                    } else {
                        // When we enter the connected or disconnected state, am.i.mullvad.net will
                        // be polled to get IP information. When it arrives, we will get another
                        // tunnel state of the same enum type, but with the IP  filled in. This
                        // match statement checks for duplicate tunnel states and skips the second
                        // print to avoid spamming the user.
                        match (&previous_tunnel_state, &new_state) {
                            (Some(TunnelState::Disconnected(_)), TunnelState::Disconnected(_))
                            | (
                                Some(TunnelState::Connected { .. }),
                                TunnelState::Connected { .. },
                            ) => continue,
                            _ => {}
                        }
                        format::print_state(&new_state, args.verbose);
                        previous_tunnel_state = Some(new_state);
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
    let device = rpc.get_device().await?;

    print_account_loggedout(&state, &device);

    if args.debug {
        println!("Tunnel state: {state:#?}");
    } else {
        // TODO: respect location arg?
        format::print_state(&state, args.verbose);
        format::print_location(&state);
    }

    if cmd == Some(Status::Listen) {
        Status::listen(rpc, args).await?;
    }
    Ok(())
}

fn print_account_loggedout(state: &TunnelState, device: &DeviceState) {
    match state {
        TunnelState::Connecting { .. } | TunnelState::Connected { .. } | TunnelState::Error(_) => {
            match device {
                DeviceState::LoggedOut => {
                    println!("Warning: You are not logged in to an account.")
                }
                DeviceState::Revoked => println!("Warning: This device has been revoked."),
                DeviceState::LoggedIn(_) => (),
            }
        }
        TunnelState::Disconnected(_) | TunnelState::Disconnecting(_) => (),
    }
}

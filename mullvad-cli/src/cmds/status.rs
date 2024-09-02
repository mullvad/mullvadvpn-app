use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use futures::StreamExt;
use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_types::{device::DeviceState, states::TunnelState};
use serde::Serialize;
use std::fmt::Debug;

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

    /// Enable debug output
    #[arg(long, short = 'd', conflicts_with_all = ["verbose", "json"])]
    debug: bool,

    /// Format output as JSON
    #[arg(long, short = 'j', conflicts_with_all = ["verbose", "debug"])]
    json: bool,
}

impl Status {
    pub async fn listen(
        mut rpc: MullvadProxyClient,
        args: StatusArgs,
        mut previous_tunnel_state: TunnelState,
    ) -> Result<()> {
        let mut event_stream = rpc.events_listen().await?;
        while let Some(event) = event_stream.next().await {
            match event? {
                DaemonEvent::TunnelState(new_state) => {
                    if args.debug {
                        println!("New tunnel state: {new_state:#?}");
                    } else if args.json {
                        let json = serde_json::to_string(&new_state)
                            .context("Failed to format output as JSON")?;
                        println!("{json}");
                    } else {
                        format::print_state(&new_state, Some(&previous_tunnel_state), args.verbose);
                        previous_tunnel_state = new_state;
                    }
                }
                DaemonEvent::Settings(settings) => {
                    print_debug_or_json(&args, "New settings", &settings)?;
                }
                DaemonEvent::RelayList(relay_list) => {
                    print_debug_or_json(&args, "New relay list", &relay_list)?;
                }
                DaemonEvent::AppVersionInfo(app_version_info) => {
                    print_debug_or_json(&args, "New app version info", &app_version_info)?;
                }
                DaemonEvent::Device(device) => {
                    print_debug_or_json(&args, "Device event", &device)?;
                }
                DaemonEvent::RemoveDevice(device) => {
                    print_debug_or_json(&args, "Remove device event", &device)?;
                }
                DaemonEvent::NewAccessMethod(access_method) => {
                    print_debug_or_json(&args, "New access method", &access_method)?;
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

    print_account_logged_out(&state, &device);

    if args.debug {
        println!("Tunnel state: {state:#?}");
    } else if args.json {
        let json = serde_json::to_string(&state).context("Failed to format output as JSON")?;
        println!("{json}");
    } else {
        format::print_state(&state, None, args.verbose);
    }

    if cmd == Some(Status::Listen) {
        Status::listen(rpc, args, state).await?;
    }
    Ok(())
}

fn print_account_logged_out(state: &TunnelState, device: &DeviceState) {
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
        TunnelState::Disconnected { .. } | TunnelState::Disconnecting(_) => (),
    }
}

fn print_debug_or_json<T: Debug + Serialize>(
    args: &StatusArgs,
    debug_message: &str,
    t: &T,
) -> Result<()> {
    if args.debug {
        println!("{debug_message}: {t:#?}");
    } else if args.json {
        let json = serde_json::to_string(&t).context("Failed to format output as JSON")?;
        println!("{json}");
    }

    Ok(())
}

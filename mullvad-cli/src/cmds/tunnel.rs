use crate::{new_rpc_client, Command, Error, Result};
use mullvad_management_interface::types::{self, Timestamp, TunnelOptions};
use mullvad_types::wireguard::DEFAULT_ROTATION_INTERVAL;
use std::{convert::TryFrom, time::Duration};

pub struct Tunnel;

#[mullvad_management_interface::async_trait]
impl Command for Tunnel {
    fn name(&self) -> &'static str {
        "tunnel"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about("Manage tunnel specific options")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(create_openvpn_subcommand())
            .subcommand(create_wireguard_subcommand())
            .subcommand(create_ipv6_subcommand())
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("openvpn", openvpn_matches)) => Self::handle_openvpn_cmd(openvpn_matches).await,
            Some(("wireguard", wg_matches)) => Self::handle_wireguard_cmd(wg_matches).await,
            Some(("ipv6", ipv6_matches)) => Self::handle_ipv6_cmd(ipv6_matches).await,
            _ => {
                unreachable!("unhandled comand");
            }
        }
    }
}

fn create_wireguard_subcommand() -> clap::App<'static> {
    let subcmd = clap::App::new("wireguard")
        .about("Manage options for Wireguard tunnels")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(create_wireguard_mtu_subcommand())
        .subcommand(create_wireguard_quantum_resistant_tunnel_subcommand())
        .subcommand(create_wireguard_keys_subcommand());
    #[cfg(windows)]
    {
        subcmd.subcommand(create_wireguard_use_wg_nt_subcommand())
    }
    #[cfg(not(windows))]
    {
        subcmd
    }
}

fn create_wireguard_mtu_subcommand() -> clap::App<'static> {
    clap::App::new("mtu")
        .about("Configure the MTU of the wireguard tunnel")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::App::new("get"))
        .subcommand(clap::App::new("unset"))
        .subcommand(clap::App::new("set").arg(clap::Arg::new("mtu").required(true)))
}

fn create_wireguard_quantum_resistant_tunnel_subcommand() -> clap::App<'static> {
    clap::App::new("quantum-resistant-tunnel")
        .about("Controls the quantum-resistant PSK exchange in the tunnel")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::App::new("get"))
        .subcommand(
            clap::App::new("set").arg(
                clap::Arg::new("policy")
                    .required(true)
                    .possible_values(["on", "off", "auto"]),
            ),
        )
}

fn create_wireguard_keys_subcommand() -> clap::App<'static> {
    clap::App::new("key")
        .about("Manage your wireguard key")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::App::new("check"))
        .subcommand(clap::App::new("regenerate"))
        .subcommand(create_wireguard_keys_rotation_interval_subcommand())
}

#[cfg(windows)]
fn create_wireguard_use_wg_nt_subcommand() -> clap::App<'static> {
    clap::App::new("use-wireguard-nt")
        .about("Enable or disable wireguard-nt")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::App::new("get"))
        .subcommand(
            clap::App::new("set").arg(
                clap::Arg::new("policy")
                    .required(true)
                    .takes_value(true)
                    .possible_values(&["on", "off"]),
            ),
        )
}

fn create_wireguard_keys_rotation_interval_subcommand() -> clap::App<'static> {
    clap::App::new("rotation-interval")
        .about("Manage automatic key rotation (given in hours)")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::App::new("get"))
        .subcommand(clap::App::new("reset").about("Use the default rotation interval"))
        .subcommand(clap::App::new("set").arg(clap::Arg::new("interval").required(true)))
}

fn create_openvpn_subcommand() -> clap::App<'static> {
    clap::App::new("openvpn")
        .about("Manage options for OpenVPN tunnels")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(create_openvpn_mssfix_subcommand())
}

fn create_openvpn_mssfix_subcommand() -> clap::App<'static> {
    clap::App::new("mssfix")
        .about("Configure the optional mssfix parameter")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::App::new("get"))
        .subcommand(clap::App::new("unset"))
        .subcommand(clap::App::new("set").arg(clap::Arg::new("mssfix").required(true)))
}

fn create_ipv6_subcommand() -> clap::App<'static> {
    clap::App::new("ipv6")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::App::new("get"))
        .subcommand(
            clap::App::new("set").arg(
                clap::Arg::new("policy")
                    .required(true)
                    .takes_value(true)
                    .possible_values(["on", "off"]),
            ),
        )
}

impl Tunnel {
    async fn handle_openvpn_cmd(matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("mssfix", mssfix_matches)) => {
                Self::handle_openvpn_mssfix_cmd(mssfix_matches).await
            }
            _ => unreachable!("unhandled command"),
        }
    }

    async fn handle_openvpn_mssfix_cmd(matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("get", _)) => Self::process_openvpn_mssfix_get().await,
            Some(("unset", _)) => Self::process_openvpn_mssfix_unset().await,
            Some(("set", set_matches)) => Self::process_openvpn_mssfix_set(set_matches).await,
            _ => unreachable!("unhandled command"),
        }
    }

    async fn handle_wireguard_cmd(matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("mtu", matches)) => match matches.subcommand() {
                Some(("get", _)) => Self::process_wireguard_mtu_get().await,
                Some(("set", matches)) => Self::process_wireguard_mtu_set(matches).await,
                Some(("unset", _)) => Self::process_wireguard_mtu_unset().await,
                _ => unreachable!("unhandled command"),
            },

            Some(("key", matches)) => match matches.subcommand() {
                Some(("check", _)) => Self::process_wireguard_key_check().await,
                Some(("regenerate", _)) => Self::process_wireguard_key_generate().await,
                Some(("rotation-interval", matches)) => match matches.subcommand() {
                    Some(("get", _)) => Self::process_wireguard_rotation_interval_get().await,
                    Some(("set", matches)) => {
                        Self::process_wireguard_rotation_interval_set(matches).await
                    }
                    Some(("reset", _)) => Self::process_wireguard_rotation_interval_reset().await,
                    _ => unreachable!("unhandled command"),
                },
                _ => unreachable!("unhandled command"),
            },

            Some(("quantum-resistant-tunnel", matches)) => match matches.subcommand() {
                Some(("get", _)) => Self::process_wireguard_quantum_resistant_tunnel_get().await,
                Some(("set", matches)) => {
                    Self::process_wireguard_quantum_resistant_tunnel_set(matches).await
                }
                _ => unreachable!("unhandled command"),
            },

            #[cfg(windows)]
            Some(("use-wireguard-nt", matches)) => match matches.subcommand() {
                Some(("get", _)) => Self::process_wireguard_use_wg_nt_get().await,
                Some(("set", matches)) => Self::process_wireguard_use_wg_nt_set(matches).await,
                _ => unreachable!("unhandled command"),
            },

            _ => unreachable!("unhandled command"),
        }
    }

    async fn process_wireguard_mtu_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options().await?;
        let mtu = tunnel_options.wireguard.unwrap().mtu;
        println!(
            "mtu: {}",
            if mtu != 0 {
                mtu.to_string()
            } else {
                "unset".to_string()
            },
        );
        Ok(())
    }

    async fn process_wireguard_mtu_set(matches: &clap::ArgMatches) -> Result<()> {
        let mtu = matches.value_of_t_or_exit::<u16>("mtu");
        let mut rpc = new_rpc_client().await?;
        rpc.set_wireguard_mtu(mtu as u32).await?;
        println!("Wireguard MTU has been updated");
        Ok(())
    }

    async fn process_wireguard_mtu_unset() -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.set_wireguard_mtu(0).await?;
        println!("Wireguard MTU has been unset");
        Ok(())
    }

    async fn process_wireguard_quantum_resistant_tunnel_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options().await?;
        match tunnel_options
            .wireguard
            .unwrap()
            .quantum_resistant
            .and_then(|state| types::quantum_resistant_state::State::from_i32(state.state))
        {
            Some(types::quantum_resistant_state::State::On) => println!("enabled"),
            Some(types::quantum_resistant_state::State::Off) => println!("disabled"),
            None | Some(types::quantum_resistant_state::State::Auto) => println!("auto"),
        }
        Ok(())
    }

    async fn process_wireguard_quantum_resistant_tunnel_set(
        matches: &clap::ArgMatches,
    ) -> Result<()> {
        let quantum_resistant = match matches.value_of("policy").unwrap() {
            "auto" => types::quantum_resistant_state::State::Auto,
            "on" => types::quantum_resistant_state::State::On,
            "off" => types::quantum_resistant_state::State::Off,
            _ => unreachable!("invalid PQ state"),
        };
        let mut rpc = new_rpc_client().await?;
        let settings = rpc.get_settings(()).await?;
        if quantum_resistant == types::quantum_resistant_state::State::On {
            let multihop_is_enabled = settings
                .into_inner()
                .relay_settings
                .unwrap()
                .endpoint
                .and_then(|endpoint| {
                    if let types::relay_settings::Endpoint::Normal(settings) = endpoint {
                        Some(settings.wireguard_constraints.unwrap().use_multihop)
                    } else {
                        None
                    }
                })
                .unwrap_or(false);
            if multihop_is_enabled {
                return Err(Error::CommandFailed(
                    "Quantum resistant tunnels do not work when multihop is enabled",
                ));
            }
        }
        rpc.set_quantum_resistant_tunnel(types::QuantumResistantState {
            state: i32::from(quantum_resistant),
        })
        .await?;
        println!("Updated quantum resistant tunnel setting");
        Ok(())
    }

    #[cfg(windows)]
    async fn process_wireguard_use_wg_nt_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options().await?;
        if tunnel_options.wireguard.unwrap().use_wireguard_nt {
            println!("enabled");
        } else {
            println!("disabled");
        }
        Ok(())
    }

    #[cfg(windows)]
    async fn process_wireguard_use_wg_nt_set(matches: &clap::ArgMatches) -> Result<()> {
        let new_state = matches.value_of("policy").unwrap() == "on";
        let mut rpc = new_rpc_client().await?;
        rpc.set_use_wireguard_nt(new_state).await?;
        println!("Updated wireguard-nt setting");
        Ok(())
    }

    async fn process_wireguard_key_check() -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let key = rpc.get_wireguard_key(()).await;
        let key = match key {
            Ok(response) => Some(response.into_inner()),
            Err(status) => {
                if status.code() == mullvad_management_interface::Code::NotFound {
                    None
                } else {
                    return Err(Error::RpcFailedExt("Failed to obtain key", status));
                }
            }
        };
        if let Some(key) = key {
            println!("Current key    : {}", base64::encode(&key.key));
            println!(
                "Key created on : {}",
                Self::format_key_timestamp(&key.created.unwrap())
            );
        } else {
            println!("No key is set");
            return Ok(());
        }
        Ok(())
    }

    async fn process_wireguard_key_generate() -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.rotate_wireguard_key(()).await?;
        println!("Rotated WireGuard key");
        Ok(())
    }

    async fn process_wireguard_rotation_interval_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options().await?;
        match tunnel_options.wireguard.unwrap().rotation_interval {
            Some(interval) => {
                let hours = duration_hours(&Duration::try_from(interval).unwrap());
                println!("Rotation interval: {hours} hour(s)");
            }
            None => println!(
                "Rotation interval: default ({} hours)",
                duration_hours(&DEFAULT_ROTATION_INTERVAL)
            ),
        }
        Ok(())
    }

    async fn process_wireguard_rotation_interval_set(matches: &clap::ArgMatches) -> Result<()> {
        let rotate_interval = matches.value_of_t_or_exit::<u64>("interval");
        let mut rpc = new_rpc_client().await?;
        rpc.set_wireguard_rotation_interval(
            types::Duration::try_from(Duration::from_secs(60 * 60 * rotate_interval))
                .expect("Failed to convert rotation interval to prost_types::Duration"),
        )
        .await?;
        println!("Set key rotation interval: {rotate_interval} hour(s)");
        Ok(())
    }

    async fn process_wireguard_rotation_interval_reset() -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.reset_wireguard_rotation_interval(()).await?;
        println!(
            "Set key rotation interval: default ({} hours)",
            duration_hours(&DEFAULT_ROTATION_INTERVAL)
        );
        Ok(())
    }

    async fn handle_ipv6_cmd(matches: &clap::ArgMatches) -> Result<()> {
        if matches.subcommand_matches("get").is_some() {
            Self::process_ipv6_get().await
        } else if let Some(m) = matches.subcommand_matches("set") {
            Self::process_ipv6_set(m).await
        } else {
            unreachable!("unhandled command");
        }
    }

    async fn process_openvpn_mssfix_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options().await?;
        let mssfix = tunnel_options.openvpn.unwrap().mssfix;
        println!(
            "mssfix: {}",
            if mssfix != 0 {
                mssfix.to_string()
            } else {
                "unset".to_string()
            },
        );
        Ok(())
    }

    async fn get_tunnel_options() -> Result<TunnelOptions> {
        let mut rpc = new_rpc_client().await?;
        Ok(rpc
            .get_settings(())
            .await?
            .into_inner()
            .tunnel_options
            .unwrap())
    }

    async fn process_openvpn_mssfix_unset() -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.set_openvpn_mssfix(0).await?;
        println!("mssfix parameter has been unset");
        Ok(())
    }

    async fn process_openvpn_mssfix_set(matches: &clap::ArgMatches) -> Result<()> {
        let new_value = matches.value_of_t_or_exit::<u16>("mssfix");
        let mut rpc = new_rpc_client().await?;
        rpc.set_openvpn_mssfix(new_value as u32).await?;
        println!("mssfix parameter has been updated");
        Ok(())
    }

    async fn process_ipv6_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options().await?;
        println!(
            "IPv6: {}",
            if tunnel_options.generic.unwrap().enable_ipv6 {
                "on"
            } else {
                "off"
            }
        );
        Ok(())
    }

    async fn process_ipv6_set(matches: &clap::ArgMatches) -> Result<()> {
        let enabled = matches.value_of("policy").unwrap() == "on";

        let mut rpc = new_rpc_client().await?;
        rpc.set_enable_ipv6(enabled).await?;
        if enabled {
            println!("Enabled IPv6");
        } else {
            println!("Disabled IPv6");
        }
        Ok(())
    }

    fn format_key_timestamp(timestamp: &Timestamp) -> String {
        let ndt = chrono::NaiveDateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32);
        let utc = chrono::DateTime::<chrono::Utc>::from_utc(ndt, chrono::Utc);
        utc.with_timezone(&chrono::Local).to_string()
    }
}

fn duration_hours(duration: &Duration) -> u64 {
    duration.as_secs() / 60 / 60
}

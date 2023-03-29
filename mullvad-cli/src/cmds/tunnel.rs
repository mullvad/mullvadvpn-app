use crate::{Command, Error, MullvadProxyClient, Result};
use mullvad_types::{
    settings::TunnelOptions,
    wireguard::{QuantumResistantState, RotationInterval, DEFAULT_ROTATION_INTERVAL},
};
use std::time::Duration;

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
                unreachable!("unhandled command");
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
        match tunnel_options.wireguard.mtu {
            Some(mssfix) => println!("mtu: {mssfix}"),
            None => println!("mtu: unset"),
        }
        Ok(())
    }

    async fn process_wireguard_mtu_set(matches: &clap::ArgMatches) -> Result<()> {
        let mtu = matches.value_of_t_or_exit::<u16>("mtu");
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_wireguard_mtu(Some(mtu)).await?;
        println!("Wireguard MTU has been updated");
        Ok(())
    }

    async fn process_wireguard_mtu_unset() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_wireguard_mtu(None).await?;
        println!("Wireguard MTU has been unset");
        Ok(())
    }

    async fn process_wireguard_quantum_resistant_tunnel_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options().await?;
        println!(
            "Quantum resistant state: {}",
            tunnel_options.wireguard.quantum_resistant
        );
        Ok(())
    }

    async fn process_wireguard_quantum_resistant_tunnel_set(
        matches: &clap::ArgMatches,
    ) -> Result<()> {
        let state = match matches.value_of("policy").unwrap() {
            "auto" => QuantumResistantState::Auto,
            "on" => QuantumResistantState::On,
            "off" => QuantumResistantState::Off,
            _ => unreachable!("invalid PQ state"),
        };
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_quantum_resistant_tunnel(state).await?;
        println!("Updated quantum resistant tunnel setting");
        Ok(())
    }

    #[cfg(windows)]
    async fn process_wireguard_use_wg_nt_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options().await?;
        if tunnel_options.wireguard.use_wireguard_nt {
            println!("enabled");
        } else {
            println!("disabled");
        }
        Ok(())
    }

    #[cfg(windows)]
    async fn process_wireguard_use_wg_nt_set(matches: &clap::ArgMatches) -> Result<()> {
        let new_state = matches.value_of("policy").unwrap() == "on";
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_use_wireguard_nt(new_state).await?;
        println!("Updated wireguard-nt setting");
        Ok(())
    }

    async fn process_wireguard_key_check() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let key = match rpc.get_wireguard_key().await {
            Ok(response) => Some(response),
            Err(ref error) => match error {
                mullvad_management_interface::Error::Rpc(status)
                    if status.code() == mullvad_management_interface::Code::NotFound =>
                {
                    None
                }
                _ => return Err(Error::Other("Failed to obtain key")),
            },
        };
        if let Some(key) = key {
            println!("Current key    : {}", key.key.to_base64());
            println!(
                "Key created on : {}",
                key.created.with_timezone(&chrono::Local),
            );
        } else {
            println!("No key is set");
            return Ok(());
        }
        Ok(())
    }

    async fn process_wireguard_key_generate() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.rotate_wireguard_key().await?;
        println!("Rotated WireGuard key");
        Ok(())
    }

    async fn process_wireguard_rotation_interval_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options().await?;
        match tunnel_options.wireguard.rotation_interval {
            Some(interval) => {
                let hours = duration_hours(interval.as_duration());
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
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_wireguard_rotation_interval(
            RotationInterval::new(Duration::from_secs(60 * 60 * rotate_interval))
                .expect("Failed to convert rotation interval to prost_types::Duration"),
        )
        .await?;
        println!("Set key rotation interval: {rotate_interval} hour(s)");
        Ok(())
    }

    async fn process_wireguard_rotation_interval_reset() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.reset_wireguard_rotation_interval().await?;
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
        match tunnel_options.openvpn.mssfix {
            Some(mssfix) => println!("mssfix: {mssfix}"),
            None => println!("mssfix: unset"),
        }
        Ok(())
    }

    async fn get_tunnel_options() -> Result<TunnelOptions> {
        let mut rpc = MullvadProxyClient::new().await?;
        Ok(rpc.get_settings().await?.tunnel_options)
    }

    async fn process_openvpn_mssfix_unset() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_openvpn_mssfix(None).await?;
        println!("mssfix parameter has been unset");
        Ok(())
    }

    async fn process_openvpn_mssfix_set(matches: &clap::ArgMatches) -> Result<()> {
        let new_value = matches.value_of_t_or_exit::<u16>("mssfix");
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_openvpn_mssfix(Some(new_value)).await?;
        println!("mssfix parameter has been updated");
        Ok(())
    }

    async fn process_ipv6_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options().await?;
        println!(
            "IPv6: {}",
            if tunnel_options.generic.enable_ipv6 {
                "on"
            } else {
                "off"
            }
        );
        Ok(())
    }

    async fn process_ipv6_set(matches: &clap::ArgMatches) -> Result<()> {
        let enabled = matches.value_of("policy").unwrap() == "on";

        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_enable_ipv6(enabled).await?;
        if enabled {
            println!("Enabled IPv6");
        } else {
            println!("Disabled IPv6");
        }
        Ok(())
    }
}

fn duration_hours(duration: &Duration) -> u64 {
    duration.as_secs() / 60 / 60
}

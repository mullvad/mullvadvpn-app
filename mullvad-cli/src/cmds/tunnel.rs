use crate::{new_rpc_client, Command, Result};
use clap::value_t;

use mullvad_types::settings::TunnelOptions;

pub struct Tunnel;

#[async_trait::async_trait]
impl Command for Tunnel {
    fn name(&self) -> &'static str {
        "tunnel"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Manage tunnel specific options")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(create_openvpn_subcommand())
            .subcommand(create_wireguard_subcommand())
            .subcommand(create_ipv6_subcommand())
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("openvpn", Some(openvpn_matches)) => Self::handle_openvpn_cmd(openvpn_matches),
            ("wireguard", Some(wg_matches)) => Self::handle_wireguard_cmd(wg_matches),
            ("ipv6", Some(ipv6_matches)) => Self::handle_ipv6_cmd(ipv6_matches),
            _ => {
                unreachable!("unhandled comand");
            }
        }
    }
}

fn create_wireguard_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("wireguard")
        .about("Manage options for Wireguard tunnels")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(create_wireguard_mtu_subcommand())
        .subcommand(create_wireguard_keys_subcommand())
}

fn create_wireguard_mtu_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("mtu")
        .about("Configure the MTU of the wireguard tunnel")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::SubCommand::with_name("get"))
        .subcommand(clap::SubCommand::with_name("unset"))
        .subcommand(
            clap::SubCommand::with_name("set").arg(clap::Arg::with_name("mtu").required(true)),
        )
}

fn create_wireguard_keys_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("key")
        .about("Manage your wireguard key")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::SubCommand::with_name("check"))
        .subcommand(clap::SubCommand::with_name("regenerate"))
        .subcommand(create_wireguard_keys_rotation_interval_subcommand())
}

fn create_wireguard_keys_rotation_interval_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("rotation-interval")
        .about("Manage automatic key rotation (specified in hours; 0 = disabled)")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::SubCommand::with_name("get"))
        .subcommand(clap::SubCommand::with_name("reset").about("Use the default rotation interval"))
        .subcommand(
            clap::SubCommand::with_name("set").arg(clap::Arg::with_name("interval").required(true)),
        )
}


fn create_openvpn_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("openvpn")
        .about("Manage options for OpenVPN tunnels")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(create_openvpn_mssfix_subcommand())
}

fn create_openvpn_mssfix_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("mssfix")
        .about("Configure the optional mssfix parameter")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::SubCommand::with_name("get"))
        .subcommand(clap::SubCommand::with_name("unset"))
        .subcommand(
            clap::SubCommand::with_name("set").arg(clap::Arg::with_name("mssfix").required(true)),
        )
}

fn create_ipv6_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("ipv6")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::SubCommand::with_name("get"))
        .subcommand(
            clap::SubCommand::with_name("set").arg(
                clap::Arg::with_name("enable")
                    .required(true)
                    .takes_value(true)
                    .possible_values(&["on", "off"]),
            ),
        )
}

impl Tunnel {
    fn handle_openvpn_cmd(matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("mssfix", Some(mssfix_matches)) => Self::handle_openvpn_mssfix_cmd(mssfix_matches),
            _ => unreachable!("unhandled command"),
        }
    }

    fn handle_openvpn_mssfix_cmd(matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("get", Some(_)) => Self::process_openvpn_mssfix_get(),
            ("unset", Some(_)) => Self::process_openvpn_mssfix_unset(),
            ("set", Some(set_matches)) => Self::process_openvpn_mssfix_set(set_matches),
            _ => unreachable!("unhandled command"),
        }
    }

    fn handle_wireguard_cmd(matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("mtu", Some(matches)) => match matches.subcommand() {
                ("get", _) => Self::process_wireguard_mtu_get(),
                ("set", Some(matches)) => Self::process_wireguard_mtu_set(matches),
                ("unset", _) => Self::process_wireguard_mtu_unset(),
                _ => unreachable!("unhandled command"),
            },

            ("key", Some(matches)) => match matches.subcommand() {
                ("check", _) => Self::process_wireguard_key_check(),
                ("regenerate", _) => Self::process_wireguard_key_generate(),
                ("rotation-interval", Some(matches)) => match matches.subcommand() {
                    ("get", _) => Self::process_wireguard_rotation_interval_get(),
                    ("set", Some(matches)) => {
                        Self::process_wireguard_rotation_interval_set(matches)
                    }
                    ("reset", _) => Self::process_wireguard_rotation_interval_reset(),
                    _ => unreachable!("unhandled command"),
                },
                _ => unreachable!("unhandled command"),
            },

            _ => unreachable!("unhandled command"),
        }
    }

    fn process_wireguard_mtu_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options()?;
        println!(
            "mtu: {}",
            tunnel_options
                .wireguard
                .mtu
                .map(|mtu| mtu.to_string())
                .unwrap_or_else(|| "unset".to_owned())
        );
        Ok(())
    }

    fn process_wireguard_mtu_set(matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mtu = value_t!(matches.value_of("mtu"), u16).unwrap_or_else(|e| e.exit());
        let mut rpc = new_rpc_client()?;
        rpc.set_wireguard_mtu(Some(mtu))?;
        println!("Wireguard MTU has been updated");
        Ok(())
    }

    fn process_wireguard_mtu_unset() -> Result<()> {
        let mut rpc = new_rpc_client()?;
        rpc.set_wireguard_mtu(None)?;
        println!("Wireguard MTU has been unset");
        Ok(())
    }

    fn process_wireguard_key_check() -> Result<()> {
        let mut rpc = new_rpc_client()?;
        match rpc.get_wireguard_key()? {
            Some(key) => {
                println!("Current key    : {}", &key.key);
                println!(
                    "Key created on : {}",
                    &key.created.with_timezone(&chrono::offset::Local)
                );
            }
            None => {
                println!("No key is set");
                return Ok(());
            }
        };

        let is_valid = rpc.verify_wireguard_key()?;
        println!("Key is valid for use with current account: {}", is_valid);
        Ok(())
    }

    fn process_wireguard_key_generate() -> Result<()> {
        let mut rpc = new_rpc_client()?;
        let result = rpc
            .generate_wireguard_key()
            .map_err(|e| crate::Error::RpcClientError(e))?;
        println!("{}", result);
        Ok(())
    }

    fn process_wireguard_rotation_interval_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options()?;
        println!(
            "Rotation interval: {} hour(s)",
            tunnel_options
                .wireguard
                .automatic_rotation
                .map(|interval| interval.to_string())
                .unwrap_or_else(|| "default".to_owned())
        );
        Ok(())
    }

    fn process_wireguard_rotation_interval_set(matches: &clap::ArgMatches<'_>) -> Result<()> {
        let rotate_interval =
            value_t!(matches.value_of("interval"), u32).unwrap_or_else(|e| e.exit());
        let mut rpc = new_rpc_client()?;
        rpc.set_wireguard_rotation_interval(Some(rotate_interval))?;
        println!("Set key rotation interval: {} hour(s)", rotate_interval);
        Ok(())
    }

    fn process_wireguard_rotation_interval_reset() -> Result<()> {
        let mut rpc = new_rpc_client()?;
        rpc.set_wireguard_rotation_interval(None)?;
        println!("Set key rotation interval: default");
        Ok(())
    }

    fn handle_ipv6_cmd(matches: &clap::ArgMatches<'_>) -> Result<()> {
        if matches.subcommand_matches("get").is_some() {
            Self::process_ipv6_get()
        } else if let Some(m) = matches.subcommand_matches("set") {
            Self::process_ipv6_set(m)
        } else {
            unreachable!("unhandled command");
        }
    }

    fn process_openvpn_mssfix_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options()?;
        println!(
            "mssfix: {}",
            tunnel_options
                .openvpn
                .mssfix
                .map_or_else(|| "unset".to_owned(), |v| v.to_string())
        );
        Ok(())
    }

    fn get_tunnel_options() -> Result<TunnelOptions> {
        let mut rpc = new_rpc_client()?;
        Ok(rpc.get_settings()?.tunnel_options)
    }

    fn process_openvpn_mssfix_unset() -> Result<()> {
        let mut rpc = new_rpc_client()?;
        rpc.set_openvpn_mssfix(None)?;
        println!("mssfix parameter has been unset");
        Ok(())
    }

    fn process_openvpn_mssfix_set(matches: &clap::ArgMatches<'_>) -> Result<()> {
        let new_value = value_t!(matches.value_of("mssfix"), u16).unwrap_or_else(|e| e.exit());
        let mut rpc = new_rpc_client()?;
        rpc.set_openvpn_mssfix(Some(new_value))?;
        println!("mssfix parameter has been updated");
        Ok(())
    }

    fn process_ipv6_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options()?;
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

    fn process_ipv6_set(matches: &clap::ArgMatches<'_>) -> Result<()> {
        let enabled = matches.value_of("enable").unwrap() == "on";

        let mut rpc = new_rpc_client()?;
        rpc.set_enable_ipv6(enabled)?;
        println!("IPv6 setting has been updated");
        Ok(())
    }
}

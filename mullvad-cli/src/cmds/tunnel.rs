use crate::{format::print_keygen_event, new_grpc_client, proto, Command, Error, Result};
use clap::value_t;
use proto::TunnelOptions;

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
            ("openvpn", Some(openvpn_matches)) => Self::handle_openvpn_cmd(openvpn_matches).await,
            ("wireguard", Some(wg_matches)) => Self::handle_wireguard_cmd(wg_matches).await,
            ("ipv6", Some(ipv6_matches)) => Self::handle_ipv6_cmd(ipv6_matches).await,
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
    async fn handle_openvpn_cmd(matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("mssfix", Some(mssfix_matches)) => Self::handle_openvpn_mssfix_cmd(mssfix_matches).await,
            _ => unreachable!("unhandled command"),
        }
    }

    async fn handle_openvpn_mssfix_cmd(matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("get", Some(_)) => Self::process_openvpn_mssfix_get().await,
            ("unset", Some(_)) => Self::process_openvpn_mssfix_unset().await,
            ("set", Some(set_matches)) => Self::process_openvpn_mssfix_set(set_matches).await,
            _ => unreachable!("unhandled command"),
        }
    }

    async fn handle_wireguard_cmd(matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("mtu", Some(matches)) => match matches.subcommand() {
                ("get", _) => Self::process_wireguard_mtu_get().await,
                ("set", Some(matches)) => Self::process_wireguard_mtu_set(matches).await,
                ("unset", _) => Self::process_wireguard_mtu_unset().await,
                _ => unreachable!("unhandled command"),
            },

            ("key", Some(matches)) => match matches.subcommand() {
                ("check", _) => Self::process_wireguard_key_check().await,
                ("regenerate", _) => Self::process_wireguard_key_generate().await,
                ("rotation-interval", Some(matches)) => match matches.subcommand() {
                    ("get", _) => Self::process_wireguard_rotation_interval_get().await,
                    ("set", Some(matches)) => {
                        Self::process_wireguard_rotation_interval_set(matches).await
                    }
                    ("reset", _) => Self::process_wireguard_rotation_interval_reset().await,
                    _ => unreachable!("unhandled command"),
                },
                _ => unreachable!("unhandled command"),
            },

            _ => unreachable!("unhandled command"),
        }
    }

    async fn process_wireguard_mtu_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options().await?;
        let mtu = tunnel_options
            .wireguard.unwrap()
            .mtu;
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

    async fn process_wireguard_mtu_set(matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mtu = value_t!(matches.value_of("mtu"), u16).unwrap_or_else(|e| e.exit());
        let mut rpc = new_grpc_client().await?;
        rpc.set_wireguard_mtu(mtu as u32).await.map_err(Error::GrpcClientError)?;
        println!("Wireguard MTU has been updated");
        Ok(())
    }

    async fn process_wireguard_mtu_unset() -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        rpc.set_wireguard_mtu(0).await.map_err(Error::GrpcClientError)?;
        println!("Wireguard MTU has been unset");
        Ok(())
    }

    async fn process_wireguard_key_check() -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        let key = rpc.get_wireguard_key(()).await.map_err(Error::GrpcClientError)?.into_inner();
        if !key.key.is_empty() {
            // TODO: Fix formatting
            println!("Current key    : {:?}", &key.key);
            println!("Key created on : {:?}", key.created);
        } else {
            println!("No key is set");
            return Ok(());
        }

        let is_valid = rpc.verify_wireguard_key(())
            .await
            .map_err(Error::GrpcClientError)?
            .into_inner();
        println!("Key is valid for use with current account: {}", is_valid);
        Ok(())
    }

    async fn process_wireguard_key_generate() -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        let keygen_event = rpc
            .generate_wireguard_key(())
            .await
            .map_err(Error::GrpcClientError)?;
        print_keygen_event(&keygen_event.into_inner());
        Ok(())
    }

    async fn process_wireguard_rotation_interval_get() -> Result<()> {
        // TODO: handle default interval
        let tunnel_options = Self::get_tunnel_options().await?;
        println!(
            "Rotation interval: {} hour(s)",
            tunnel_options
                .wireguard.unwrap()
                .automatic_rotation
        );
        Ok(())
    }

    async fn process_wireguard_rotation_interval_set(matches: &clap::ArgMatches<'_>) -> Result<()> {
        // TODO: handle default interval
        let rotate_interval =
            value_t!(matches.value_of("interval"), u32).unwrap_or_else(|e| e.exit());
        let mut rpc = new_grpc_client().await?;
        rpc.set_wireguard_rotation_interval(rotate_interval)
            .await
            .map_err(Error::GrpcClientError)?;
        println!("Set key rotation interval: {} hour(s)", rotate_interval);
        Ok(())
    }

    async fn process_wireguard_rotation_interval_reset() -> Result<()> {
        // TODO: handle default interval
        let mut rpc = new_grpc_client().await?;
        rpc.set_wireguard_rotation_interval(0)
            .await
            .map_err(Error::GrpcClientError)?;
        println!("Set key rotation interval: default");
        Ok(())
    }

    async fn handle_ipv6_cmd(matches: &clap::ArgMatches<'_>) -> Result<()> {
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
        let mut rpc = new_grpc_client().await?;
        Ok(rpc.get_settings(())
            .await
            .map_err(Error::GrpcClientError)?
            .into_inner()
            .tunnel_options
            .unwrap()
        )
    }

    async fn process_openvpn_mssfix_unset() -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        rpc.set_openvpn_mssfix(0).await.map_err(Error::GrpcClientError)?;
        println!("mssfix parameter has been unset");
        Ok(())
    }

    async fn process_openvpn_mssfix_set(matches: &clap::ArgMatches<'_>) -> Result<()> {
        let new_value = value_t!(matches.value_of("mssfix"), u16).unwrap_or_else(|e| e.exit());
        let mut rpc = new_grpc_client().await?;
        rpc.set_openvpn_mssfix(new_value as u32).await.map_err(Error::GrpcClientError)?;
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

    async fn process_ipv6_set(matches: &clap::ArgMatches<'_>) -> Result<()> {
        let enabled = matches.value_of("enable").unwrap() == "on";

        let mut rpc = new_grpc_client().await?;
        rpc.set_enable_ipv6(enabled).await.map_err(Error::GrpcClientError)?;
        if enabled {
            println!("Enabled IPv6");
        } else {
            println!("Disabled IPv6");
        }
        Ok(())
    }
}

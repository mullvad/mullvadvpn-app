use crate::{new_rpc_client, Command, Result};
use clap::value_t;

use mullvad_types::{relay_constraints::BridgeSettings, settings::TunnelOptions};
use talpid_types::net::openvpn::{self, SHADOWSOCKS_CIPHERS};

use std::net::{IpAddr, SocketAddr};

pub struct Bridge;

impl Command for Bridge {
    fn name(&self) -> &'static str {
        "bridge"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Manage use of bridge")
            .setting(clap::AppSettings::SubcommandRequired)
            .subcommand(create_bridge_set_subcommand())
            .subcommand(create_bridge_get_subcommand())
    }

    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            // ("set", Some(set_matches)) => Self::handle_set_cmd(set_matches),
            // ("get", _) => Self::handle_bridge_get(),
            _ => unreachable!("unhandled command"),
        }
    }
}

fn create_bridge_set_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("set")
        .about("Set bridge state and settings")
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(create_set_state_subcommand())
        .subcommand(create_set_settings_subcommand())
}

fn create_bridge_get_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("get")
        .about("Get current bridge settings and state")
        .setting(clap::AppSettings::SubcommandRequired)
}


fn create_set_settings_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("get")
        .about("")
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(create_set_custom_settings_subcommand())
        .subcommand(create_set_bridge_constraints_subcommand())
}

fn create_set_custom_settings_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("custom")
        .about("Configure a SOCKS5 proxy")
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(
            clap::SubCommand::with_name("local")
                .about("Registers a local SOCKS5 proxy")
                .arg(
                    clap::Arg::with_name("local-port")
                        .help("Specifies the port the local proxy server is listening on")
                        .required(true)
                        .index(1),
                )
                .arg(
                    clap::Arg::with_name("remote-ip")
                        .help("Specifies the IP of the proxy server peer")
                        .required(true)
                        .index(2),
                )
                .arg(
                    clap::Arg::with_name("remote-port")
                        .help("Specifies the port of the proxy server peer")
                        .required(true)
                        .index(3),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("remote")
                .about("Registers a remote SOCKS5 proxy")
                .arg(
                    clap::Arg::with_name("remote-ip")
                        .help("Specifies the IP of the remote proxy server")
                        .required(true)
                        .index(1),
                )
                .arg(
                    clap::Arg::with_name("remote-port")
                        .help("Specifies the port the remote proxy server is listening on")
                        .required(true)
                        .index(2),
                )
                .arg(
                    clap::Arg::with_name("username")
                        .help("Specifies the username for remote authentication")
                        .required(true)
                        .index(3),
                )
                .arg(
                    clap::Arg::with_name("password")
                        .help("Specifies the password for remote authentication")
                        .required(true)
                        .index(4),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("shadowsocks")
                .about("Configure bundled Shadowsocks proxy")
                .arg(
                    clap::Arg::with_name("remote-ip")
                        .help("Specifies the IP of the remote Shadowsocks server")
                        .required(true)
                        .index(1),
                )
                .arg(
                    clap::Arg::with_name("remote-port")
                        .help("Specifies the port of the remote Shadowsocks server")
                        .default_value("443")
                        .index(2),
                )
                .arg(
                    clap::Arg::with_name("password")
                        .help("Specifies the password on the remote Shadowsocks server")
                        .default_value("23#dfsbbb")
                        .index(3),
                )
                .arg(
                    clap::Arg::with_name("cipher")
                        .help("Specifies the cipher to use")
                        .default_value("chacha20")
                        .possible_values(SHADOWSOCKS_CIPHERS)
                        .index(4),
                ),
        )
}

fn create_set_bridge_constraints_subcommand() -> clap::App<'static, 'static> {
    // TODO: figure out how to reuse code from cmds/relay.rs
    clap::SubCommand::with_name("location")
        .about(
            "Set country or city to select bridges from. Use the 'list' \
             command to show available alternatives",
        )
        .arg(
            clap::Arg::with_name("country")
                .help("The two letter country code, or 'any' for no preference.")
                .required(true)
                .index(1)
                .validator(|code| {
                    if code.len() == 2 || code == "any" {
                        Ok(())
                    } else {
                        Err(String::from("Country codes must be two letters, or 'any'."))
                    }
                }),
        )
    // .arg(
    //     clap::Arg::with_name("city")
    //         .help("The three letter city code")
    //         .index(2)
    //         .validator(city_code_validator),
    // )
    // .arg(
    //     clap::Arg::with_name("hostname")
    //         .help("The relay hostname")
    //         .index(3),
    // )
}

fn create_set_state_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("get").about("Get current bridge settings and state")
}

impl Bridge {
    fn handle_set(matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("settings", Some(_)) => Self::process_openvpn_proxy_get(),
            ("", Some(_)) => Self::process_openvpn_proxy_unset(),
            ("set", Some(set_matches)) => Self::process_openvpn_proxy_set(set_matches),
            _ => unreachable!("unhandled command"),
        }
    }

    fn handle_set_bridge_state(matches: &clap::ArgMatches<'_>) -> Result<()> {
        Ok(())
    }

    fn handle_bridge_set_custom_settings(matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(args) = matches.subcommand_matches("local") {
            let local_port =
                value_t!(args.value_of("local-port"), u16).unwrap_or_else(|e| e.exit());
            let remote_ip =
                value_t!(args.value_of("remote-ip"), IpAddr).unwrap_or_else(|e| e.exit());
            let remote_port =
                value_t!(args.value_of("remote-port"), u16).unwrap_or_else(|e| e.exit());

            let proxy = openvpn::LocalProxySettings {
                port: local_port,
                peer: SocketAddr::new(remote_ip, remote_port),
            };

            let packed_proxy = openvpn::ProxySettings::Local(proxy);

            if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
                panic!(error);
            }

            let mut rpc = new_rpc_client()?;
            rpc.set_bridge_settings(BridgeSettings::Custom(packed_proxy))?;
        } else if let Some(args) = matches.subcommand_matches("remote") {
            let remote_ip =
                value_t!(args.value_of("remote-ip"), IpAddr).unwrap_or_else(|e| e.exit());
            let remote_port =
                value_t!(args.value_of("remote-port"), u16).unwrap_or_else(|e| e.exit());
            let username = args.value_of("username");
            let password = args.value_of("password");

            let auth = match (username, password) {
                (Some(username), Some(password)) => Some(openvpn::ProxyAuth {
                    username: username.to_string(),
                    password: password.to_string(),
                }),
                _ => None,
            };

            let proxy = openvpn::RemoteProxySettings {
                address: SocketAddr::new(remote_ip, remote_port),
                auth,
            };

            let packed_proxy = openvpn::ProxySettings::Remote(proxy);

            if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
                panic!(error);
            }

            let mut rpc = new_rpc_client()?;
            rpc.set_bridge_settings(BridgeSettings::Custom(packed_proxy))?;
        } else if let Some(args) = matches.subcommand_matches("shadowsocks") {
            let remote_ip =
                value_t!(args.value_of("remote-ip"), IpAddr).unwrap_or_else(|e| e.exit());
            let remote_port =
                value_t!(args.value_of("remote-port"), u16).unwrap_or_else(|e| e.exit());
            let password = args.value_of("password").unwrap().to_string();
            let cipher = args.value_of("cipher").unwrap().to_string();

            let proxy = openvpn::ShadowsocksProxySettings {
                peer: SocketAddr::new(remote_ip, remote_port),
                password,
                cipher,
            };

            let packed_proxy = openvpn::ProxySettings::Shadowsocks(proxy);

            if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
                panic!(error);
            }

            let mut rpc = new_rpc_client()?;
            rpc.set_bridge_settings(BridgeSettings::Custom(packed_proxy))?;
        } else {
            unreachable!("unhandled proxy type");
        }

        println!("proxy details have been updated");
        Ok(())
    }
}

use crate::{new_rpc_client, Command, Result};
use clap::{self, value_t};

use talpid_types::net::{
    LocalOpenVpnProxySettings, OpenVpnProxyAuth, OpenVpnProxySettings,
    OpenVpnProxySettingsValidation, RemoteOpenVpnProxySettings, TunnelOptions,
};

use std::net::{IpAddr, SocketAddr};

pub struct Tunnel;

impl Command for Tunnel {
    fn name(&self) -> &'static str {
        "tunnel"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Manage tunnel specific options")
            .setting(clap::AppSettings::SubcommandRequired)
            .subcommand(create_openvpn_subcommand())
            .subcommand(create_ipv6_subcommand())
    }

    fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(openvpn_matches) = matches.subcommand_matches("openvpn") {
            Self::handle_openvpn_cmd(openvpn_matches)
        } else if let Some(ipv6_matches) = matches.subcommand_matches("ipv6") {
            Self::handle_ipv6_cmd(ipv6_matches)
        } else {
            unreachable!("unhandled command");
        }
    }
}

fn create_openvpn_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("openvpn")
        .about("Manage options for OpenVPN tunnels")
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(create_openvpn_mssfix_subcommand())
        .subcommand(create_openvpn_proxy_subcommand())
}

fn create_openvpn_mssfix_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("mssfix")
        .about("Configure the optional mssfix parameter")
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(clap::SubCommand::with_name("get"))
        .subcommand(clap::SubCommand::with_name("unset"))
        .subcommand(
            clap::SubCommand::with_name("set").arg(clap::Arg::with_name("mssfix").required(true)),
        )
}

fn create_openvpn_proxy_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("proxy")
        .about("Configure a SOCKS5 proxy")
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(clap::SubCommand::with_name("get"))
        .subcommand(clap::SubCommand::with_name("unset"))
        .subcommand(
            clap::SubCommand::with_name("set")
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
                                .index(3),
                        )
                        .arg(
                            clap::Arg::with_name("password")
                                .help("Specifies the password for remote authentication")
                                .index(4),
                        ),
                ),
        )
}

fn create_ipv6_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("ipv6")
        .setting(clap::AppSettings::SubcommandRequired)
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
    fn handle_openvpn_cmd(matches: &clap::ArgMatches) -> Result<()> {
        if let Some(m) = matches.subcommand_matches("mssfix") {
            Self::handle_openvpn_mssfix_cmd(m)
        } else if let Some(m) = matches.subcommand_matches("proxy") {
            Self::handle_openvpn_proxy_cmd(m)
        } else {
            unreachable!("unhandled command");
        }
    }

    fn handle_openvpn_mssfix_cmd(matches: &clap::ArgMatches) -> Result<()> {
        if matches.subcommand_matches("get").is_some() {
            Self::process_openvpn_mssfix_get()
        } else if matches.subcommand_matches("unset").is_some() {
            Self::process_openvpn_mssfix_unset()
        } else if let Some(m) = matches.subcommand_matches("set") {
            Self::process_openvpn_mssfix_set(m)
        } else {
            unreachable!("unhandled command");
        }
    }

    fn handle_openvpn_proxy_cmd(matches: &clap::ArgMatches) -> Result<()> {
        if matches.subcommand_matches("get").is_some() {
            Self::process_openvpn_proxy_get()
        } else if matches.subcommand_matches("unset").is_some() {
            Self::process_openvpn_proxy_unset()
        } else if let Some(m) = matches.subcommand_matches("set") {
            Self::process_openvpn_proxy_set(m)
        } else {
            unreachable!("unhandled command");
        }
    }

    fn handle_ipv6_cmd(matches: &clap::ArgMatches) -> Result<()> {
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
                .map_or_else(|| "unset".to_string(), |v| v.to_string())
        );
        Ok(())
    }

    fn get_tunnel_options() -> Result<TunnelOptions> {
        let mut rpc = new_rpc_client()?;
        Ok(rpc.get_settings()?.get_tunnel_options().clone())
    }

    fn process_openvpn_mssfix_unset() -> Result<()> {
        let mut rpc = new_rpc_client()?;
        rpc.set_openvpn_mssfix(None)?;
        println!("mssfix parameter has been unset");
        Ok(())
    }

    fn process_openvpn_mssfix_set(matches: &clap::ArgMatches) -> Result<()> {
        let new_value = value_t!(matches.value_of("mssfix"), u16).unwrap_or_else(|e| e.exit());
        let mut rpc = new_rpc_client()?;
        rpc.set_openvpn_mssfix(Some(new_value))?;
        println!("mssfix parameter has been updated");
        Ok(())
    }

    fn process_openvpn_proxy_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options()?;
        if let Some(proxy) = tunnel_options.openvpn.proxy {
            if let OpenVpnProxySettings::Local(local_proxy) = proxy {
                Self::print_local_proxy(&local_proxy)
            } else if let OpenVpnProxySettings::Remote(remote_proxy) = proxy {
                Self::print_remote_proxy(&remote_proxy)
            } else {
                unreachable!("unhandled proxy type");
            }
        } else {
            println!("proxy: unset");
        }
        Ok(())
    }

    fn print_local_proxy(proxy: &LocalOpenVpnProxySettings) {
        println!("proxy: local");
        println!("  local port: {}", proxy.port);
        println!("  peer IP: {}", proxy.peer.ip());
        println!("  peer port: {}", proxy.peer.port());
    }

    fn print_remote_proxy(proxy: &RemoteOpenVpnProxySettings) {
        println!("proxy: remote");
        println!("  server IP: {}", proxy.address.ip());
        println!("  server port: {}", proxy.address.port());

        if let Some(ref auth) = proxy.auth {
            println!("  auth username: {}", auth.username);
            println!("  auth password: {}", auth.password);
        } else {
            println!("  auth: none");
        }
    }

    fn process_openvpn_proxy_unset() -> Result<()> {
        let mut rpc = new_rpc_client()?;
        rpc.set_openvpn_proxy(None)?;
        println!("proxy details have been unset");
        Ok(())
    }

    fn process_openvpn_proxy_set(matches: &clap::ArgMatches) -> Result<()> {
        if let Some(args) = matches.subcommand_matches("local") {
            let local_port =
                value_t!(args.value_of("local-port"), u16).unwrap_or_else(|e| e.exit());
            let remote_ip =
                value_t!(args.value_of("remote-ip"), IpAddr).unwrap_or_else(|e| e.exit());
            let remote_port =
                value_t!(args.value_of("remote-port"), u16).unwrap_or_else(|e| e.exit());

            let proxy = LocalOpenVpnProxySettings {
                port: local_port,
                peer: SocketAddr::new(remote_ip, remote_port),
            };

            let packed_proxy = OpenVpnProxySettings::Local(proxy);

            if let Err(error) = OpenVpnProxySettingsValidation::validate(&packed_proxy) {
                panic!(error);
            }

            let mut rpc = new_rpc_client()?;
            rpc.set_openvpn_proxy(Some(packed_proxy))?;
        } else if let Some(args) = matches.subcommand_matches("remote") {
            let remote_ip =
                value_t!(args.value_of("remote-ip"), IpAddr).unwrap_or_else(|e| e.exit());
            let remote_port =
                value_t!(args.value_of("remote-port"), u16).unwrap_or_else(|e| e.exit());
            let username = args.value_of("username");
            let password = args.value_of("password");

            let auth = match (username, password) {
                (Some(username), Some(password)) => Some(OpenVpnProxyAuth {
                    username: username.to_string(),
                    password: password.to_string(),
                }),
                _ => None,
            };

            let proxy = RemoteOpenVpnProxySettings {
                address: SocketAddr::new(remote_ip, remote_port),
                auth,
            };

            let packed_proxy = OpenVpnProxySettings::Remote(proxy);

            if let Err(error) = OpenVpnProxySettingsValidation::validate(&packed_proxy) {
                panic!(error);
            }

            let mut rpc = new_rpc_client()?;
            rpc.set_openvpn_proxy(Some(packed_proxy))?;
        } else {
            unreachable!("unhandled proxy type");
        }

        println!("proxy details have been updated");
        println!("note: The OpenVPN tunnel constraints have been updated to use TCP");
        Ok(())
    }

    fn process_ipv6_get() -> Result<()> {
        let tunnel_options = Self::get_tunnel_options()?;
        println!(
            "IPv6: {}",
            if tunnel_options.enable_ipv6 {
                "on"
            } else {
                "off"
            }
        );
        Ok(())
    }

    fn process_ipv6_set(matches: &clap::ArgMatches) -> Result<()> {
        let enabled = matches.value_of("enable").unwrap() == "on";

        let mut rpc = new_rpc_client()?;
        rpc.set_enable_ipv6(enabled)?;
        println!("IPv6 setting has been updated");
        Ok(())
    }
}

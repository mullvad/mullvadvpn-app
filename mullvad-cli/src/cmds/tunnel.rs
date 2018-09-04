use clap;
use {new_rpc_client, Command, Result};

use talpid_types::net::{OpenVpnTunnelOptions, TunnelOptions};

pub struct Tunnel;

impl Command for Tunnel {
    fn name(&self) -> &'static str {
        "tunnel"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Manage tunnel specific options")
            .setting(clap::AppSettings::SubcommandRequired)
            .subcommand(
                clap::SubCommand::with_name("openvpn")
                    .about("Manage options for OpenVPN tunnels")
                    .setting(clap::AppSettings::SubcommandRequired)
                    .subcommand(
                        clap::SubCommand::with_name("set")
                            .subcommand(
                                clap::SubCommand::with_name("mssfix").arg(
                                    clap::Arg::with_name("mssfix")
                                        .help(
                                            "Sets the optional  mssfix parameter. \
                                             Set an empty string to clear it.",
                                        ).required(true),
                                ),
                            ).setting(clap::AppSettings::SubcommandRequired),
                    ).subcommand(
                        clap::SubCommand::with_name("get")
                            .help("Retrieves the current setting for mssfix"),
                    ),
            ).subcommand(
                clap::SubCommand::with_name("set")
                    .subcommand(
                        clap::SubCommand::with_name("ipv6").arg(
                            clap::Arg::with_name("enable")
                                .required(true)
                                .takes_value(true)
                                .possible_values(&["on", "off"]),
                        ),
                    ).setting(clap::AppSettings::SubcommandRequired),
            ).subcommand(
                clap::SubCommand::with_name("get")
                    .help("Retrieves the current setting for common tunnel options"),
            )
    }

    fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(openvpn_matches) = matches.subcommand_matches("openvpn") {
            Self::handle_openvpn_cmd(openvpn_matches)
        } else if let Some(set_matches) = matches.subcommand_matches("set") {
            Self::set_tunnel_option(set_matches)
        } else if let Some(_) = matches.subcommand_matches("get") {
            let tunnel_options = Self::get_tunnel_options()?;
            Self::print_common_tunnel_options(&tunnel_options);
            Ok(())
        } else {
            unreachable!("No tunnel command given")
        }
    }
}

impl Tunnel {
    fn set_tunnel_option(matches: &clap::ArgMatches) -> Result<()> {
        if let Some(ipv6_args) = matches.subcommand_matches("ipv6") {
            Self::set_enable_ipv6_option(ipv6_args)
        } else {
            unreachable!("Invalid option passed to 'tunnel set'");
        }
    }

    fn set_enable_ipv6_option(args: &clap::ArgMatches) -> Result<()> {
        let enabled = args.value_of("enable").unwrap() == "on";

        let mut rpc = new_rpc_client()?;
        rpc.set_openvpn_enable_ipv6(enabled)?;
        println!("enable_ipv6 parameter updated");
        Ok(())
    }

    fn handle_openvpn_cmd(matches: &clap::ArgMatches) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            Self::set_openvpn_option(set_matches)
        } else if let Some(_) = matches.subcommand_matches("get") {
            let tunnel_options = Self::get_tunnel_options()?;
            Self::print_openvpn_tunnel_options(tunnel_options.openvpn);
            Ok(())
        } else {
            unreachable!("Unrecognized subcommand");
        }
    }

    fn set_openvpn_option(matches: &clap::ArgMatches) -> Result<()> {
        if let Some(mssfix_args) = matches.subcommand_matches("mssfix") {
            Self::set_openvpn_mssfix_option(mssfix_args)
        } else {
            unreachable!("Invalid option passed to 'openvpn set'");
        }
    }

    fn set_openvpn_mssfix_option(args: &clap::ArgMatches) -> Result<()> {
        let mssfix_str = args.value_of("mssfix").unwrap();
        let mssfix: Option<u16> = if mssfix_str == "" {
            None
        } else {
            Some(mssfix_str.parse()?)
        };

        let mut rpc = new_rpc_client()?;
        rpc.set_openvpn_mssfix(mssfix)?;
        println!("mssfix parameter updated");
        Ok(())
    }

    fn get_tunnel_options() -> Result<TunnelOptions> {
        let mut rpc = new_rpc_client()?;
        Ok(rpc.get_tunnel_options()?)
    }

    fn print_common_tunnel_options(options: &TunnelOptions) {
        println!("Common tunnel options");
        println!(
            "\tIPv6:   {}",
            if options.openvpn.enable_ipv6 {
                "on"
            } else {
                "off"
            }
        );
    }

    fn print_openvpn_tunnel_options(options: OpenVpnTunnelOptions) {
        println!("OpenVPN tunnel options");
        println!(
            "\tmssfix: {}",
            options
                .mssfix
                .map_or_else(|| "UNSET".to_string(), |v| v.to_string())
        );
    }
}

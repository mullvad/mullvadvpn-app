use {Command, Result, ResultExt};
use clap;
use std::str::FromStr;

use mullvad_types::relay_constraints::{Constraint, OpenVpnConstraintsUpdate, RelayConstraints,
                                       RelayConstraintsUpdate, TunnelConstraintsUpdate};

use rpc;
use talpid_types::net::TransportProtocol;

pub struct Relay;

impl Command for Relay {
    fn name(&self) -> &'static str {
        "relay"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Manage relay and tunnel constraints")
            .setting(clap::AppSettings::SubcommandRequired)
            .subcommand(
                clap::SubCommand::with_name("set")
                    .setting(clap::AppSettings::SubcommandRequired)
                    .subcommand(
                        clap::SubCommand::with_name("host")
                            .about("Set host")
                            .arg(clap::Arg::with_name("host").required(true)),
                    )
                    .subcommand(
                        clap::SubCommand::with_name("port")
                            .about("Set port")
                            .arg(clap::Arg::with_name("port").required(true)),
                    )
                    .subcommand(
                        clap::SubCommand::with_name("protocol")
                            .about("Set protocol")
                            .arg(
                                clap::Arg::with_name("protocol")
                                    .required(true)
                                    .possible_values(&["any", "udp", "tcp"]),
                            ),
                    ),
            )
            .subcommand(clap::SubCommand::with_name("get"))
    }

    fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            self.set(set_matches)
        } else if let Some(_) = matches.subcommand_matches("get") {
            self.get()
        } else {
            unreachable!("No relay command given");
        }
    }
}

impl Relay {
    fn update_constraints(&self, constraints_update: RelayConstraintsUpdate) -> Result<()> {
        rpc::call("update_relay_constraints", &[constraints_update])
            .map(|_: Option<()>| println!("Relay constraints updated"))
    }

    fn set(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(host_matches) = matches.subcommand_matches("host") {
            let host = value_t_or_exit!(host_matches.value_of("host"), String);

            self.update_constraints(RelayConstraintsUpdate {
                host: Some(Constraint::Only(host)),
                tunnel: TunnelConstraintsUpdate::OpenVpn(OpenVpnConstraintsUpdate {
                    port: None,
                    protocol: None,
                }),
            })
        } else if let Some(port_matches) = matches.subcommand_matches("port") {
            let port = parse_port(port_matches.value_of("port").unwrap())?;

            self.update_constraints(RelayConstraintsUpdate {
                host: None,
                tunnel: TunnelConstraintsUpdate::OpenVpn(OpenVpnConstraintsUpdate {
                    port: Some(port),
                    protocol: None,
                }),
            })
        } else if let Some(protocol_matches) = matches.subcommand_matches("protocol") {
            let protocol = parse_protocol(protocol_matches.value_of("protocol").unwrap());

            self.update_constraints(RelayConstraintsUpdate {
                host: None,
                tunnel: TunnelConstraintsUpdate::OpenVpn(OpenVpnConstraintsUpdate {
                    port: None,
                    protocol: Some(protocol),
                }),
            })
        } else {
            unreachable!("No set relay command given");
        }
    }

    fn get(&self) -> Result<()> {
        let constraints: RelayConstraints = rpc::call("get_relay_constraints", &[] as &[u8; 0])?;
        println!("Current constraints: {:?}", constraints);

        Ok(())
    }
}


fn parse_port(raw_port: &str) -> Result<Constraint<u16>> {
    match raw_port.to_lowercase().as_str() {
        "any" => Ok(Constraint::Any),
        port => Ok(Constraint::Only(
            u16::from_str(port).chain_err(|| "Invalid port")?,
        )),
    }
}

/// Parses a protocol constraint string. Can be infallible because the possible values are limited
/// with clap.
fn parse_protocol(raw_protocol: &str) -> Constraint<TransportProtocol> {
    match raw_protocol.to_lowercase().as_str() {
        "any" => Constraint::Any,
        "udp" => Constraint::Only(TransportProtocol::Udp),
        "tcp" => Constraint::Only(TransportProtocol::Tcp),
        _ => unreachable!(),
    }
}

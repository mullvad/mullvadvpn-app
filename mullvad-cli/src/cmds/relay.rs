use {Command, Result};
use clap;

use rpc;

use mullvad_types::relay_constraints::{OpenVpnConstraintsUpdate, Port, RelayConstraintsUpdate,
                                       TunnelConstraintsUpdate};
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
                            .possible_values(&["udp", "tcp"]),
                    ),
            )
    }

    fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(host_matches) = matches.subcommand_matches("host") {
            let host = value_t_or_exit!(host_matches.value_of("host"), String);

            self.update_constraints(RelayConstraintsUpdate {
                host: Some(Some(host)),
                tunnel: TunnelConstraintsUpdate::OpenVpn(OpenVpnConstraintsUpdate {
                    port: None,
                    protocol: None,
                }),
            })
        } else if let Some(port_matches) = matches.subcommand_matches("port") {
            let port = value_t_or_exit!(port_matches.value_of("port"), Port);

            self.update_constraints(RelayConstraintsUpdate {
                host: None,
                tunnel: TunnelConstraintsUpdate::OpenVpn(OpenVpnConstraintsUpdate {
                    port: Some(port),
                    protocol: None,
                }),
            })
        } else if let Some(protocol_matches) = matches.subcommand_matches("protocol") {
            let protocol =
                value_t_or_exit!(protocol_matches.value_of("protocol"), TransportProtocol);

            self.update_constraints(RelayConstraintsUpdate {
                host: None,
                tunnel: TunnelConstraintsUpdate::OpenVpn(OpenVpnConstraintsUpdate {
                    port: None,
                    protocol: Some(protocol),
                }),
            })
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
}

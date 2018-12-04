use crate::{new_rpc_client, Command, Result, ResultExt};
use clap::value_t;
use std::str::FromStr;

use mullvad_types::{
    relay_constraints::{
        Constraint, LocationConstraint, OpenVpnConstraints, RelayConstraintsUpdate,
        RelaySettingsUpdate, TunnelConstraints,
    },
    CustomTunnelEndpoint,
};
use talpid_types::net::{
    OpenVpnEndpointData, TransportProtocol, TunnelEndpointData, WireguardEndpointData,
};

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
                    .about(
                        "Set relay server selection parameters. Such as location and port/protocol",
                    )
                    .setting(clap::AppSettings::SubcommandRequired)
                    .subcommand(
                        clap::SubCommand::with_name("custom")
                            .about("Set a custom VPN relay")
                            .arg(
                                clap::Arg::with_name("tunnel")
                                    .required(true)
                                    .index(1)
                                    .possible_values(&["openvpn", "wireguard"]),
                            )
                            .arg(
                                clap::Arg::with_name("host")
                                    .help("Hostname or IP")
                                    .required(true)
                                    .index(2),
                            )
                            .arg(
                                clap::Arg::with_name("port")
                                    .help("Remote network port")
                                    .required(true)
                                    .index(3),
                            )
                            .arg(
                                clap::Arg::with_name("protocol")
                                    .help("Transport protocol. For Wireguard this is ignored.")
                                    .index(4)
                                    .default_value("udp")
                                    .possible_values(&["udp", "tcp"]),
                            ),
                    )
                    .subcommand(
                        clap::SubCommand::with_name("location")
                            .about(
                                "Set country or city to select relays from. Use the 'list' \
                                 command to show available alternatives.",
                            )
                            .arg(
                                clap::Arg::with_name("country")
                                    .help(
                                        "The two letter country code, or 'any' for no preference.",
                                    )
                                    .required(true)
                                    .index(1)
                                    .validator(country_code_validator),
                            )
                            .arg(
                                clap::Arg::with_name("city")
                                    .help("The three letter city code")
                                    .index(2)
                                    .validator(city_code_validator),
                            )
                            .arg(
                                clap::Arg::with_name("hostname")
                                    .help("The relay hostname")
                                    .index(3),
                            ),
                    )
                    .subcommand(
                        clap::SubCommand::with_name("tunnel")
                            .about("Set tunnel constraints")
                            .arg(clap::Arg::with_name("port").required(true).index(1))
                            .arg(
                                clap::Arg::with_name("protocol")
                                    .required(true)
                                    .index(2)
                                    .possible_values(&["any", "udp", "tcp"]),
                            ),
                    ),
            )
            .subcommand(clap::SubCommand::with_name("get"))
            .subcommand(
                clap::SubCommand::with_name("list").about("List available countries and cities"),
            )
            .subcommand(
                clap::SubCommand::with_name("update")
                    .about("Update the list of available countries and cities"),
            )
    }

    fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            self.set(set_matches)
        } else if matches.subcommand_matches("get").is_some() {
            self.get()
        } else if matches.subcommand_matches("list").is_some() {
            self.list()
        } else if matches.subcommand_matches("update").is_some() {
            self.update()
        } else {
            unreachable!("No relay command given");
        }
    }
}

impl Relay {
    fn update_constraints(&self, update: RelaySettingsUpdate) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        rpc.update_relay_settings(update)?;
        println!("Relay constraints updated");
        Ok(())
    }

    fn set(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(custom_matches) = matches.subcommand_matches("custom") {
            self.set_custom(custom_matches)
        } else if let Some(location_matches) = matches.subcommand_matches("location") {
            self.set_location(location_matches)
        } else if let Some(tunnel_matches) = matches.subcommand_matches("tunnel") {
            self.set_tunnel(tunnel_matches)
        } else {
            unreachable!("No set relay command given");
        }
    }

    fn set_custom(&self, matches: &clap::ArgMatches) -> Result<()> {
        let host = value_t!(matches.value_of("host"), String).unwrap_or_else(|e| e.exit());
        let port = value_t!(matches.value_of("port"), u16).unwrap_or_else(|e| e.exit());
        let tunnel = match matches.value_of("tunnel").unwrap() {
            "openvpn" => TunnelEndpointData::OpenVpn(OpenVpnEndpointData {
                port,
                protocol: value_t!(matches.value_of("protocol"), TransportProtocol).unwrap(),
            }),
            // TODO: FIX
            // "wireguard" => TunnelEndpointData::Wireguard(WireguardEndpointData { port }),
            _ => unreachable!("Invalid tunnel protocol"),
        };
        self.update_constraints(RelaySettingsUpdate::CustomTunnelEndpoint(
            CustomTunnelEndpoint { host, tunnel },
        ))
    }

    fn set_location(&self, matches: &clap::ArgMatches) -> Result<()> {
        let country = matches.value_of("country").unwrap();
        let city = matches.value_of("city");
        let hostname = matches.value_of("hostname");

        let location_constraint = match (country, city, hostname) {
            ("any", None, None) => Constraint::Any,
            ("any", ..) => clap::Error::with_description(
                "City can't be given when selecting 'any' country",
                clap::ErrorKind::InvalidValue,
            )
            .exit(),
            (country, None, None) => {
                Constraint::Only(LocationConstraint::Country(country.to_owned()))
            }
            (country, Some(city), None) => Constraint::Only(LocationConstraint::City(
                country.to_owned(),
                city.to_owned(),
            )),
            (country, Some(city), Some(hostname)) => {
                Constraint::Only(LocationConstraint::Hostname(
                    country.to_owned(),
                    city.to_owned(),
                    hostname.to_owned(),
                ))
            }
            (..) => clap::Error::with_description(
                "Invalid country, city and hostname combination given",
                clap::ErrorKind::InvalidValue,
            )
            .exit(),
        };

        self.update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            location: Some(location_constraint),
            tunnel: None,
        }))
    }

    fn set_tunnel(&self, matches: &clap::ArgMatches) -> Result<()> {
        let port = parse_port_constraint(matches.value_of("port").unwrap())?;
        let protocol = parse_protocol_constraint(matches.value_of("protocol").unwrap());

        self.update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            location: None,
            tunnel: Some(Constraint::Only(TunnelConstraints::OpenVpn(
                OpenVpnConstraints { port, protocol },
            ))),
        }))
    }

    fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        let constraints = rpc.get_settings()?.get_relay_settings();
        println!("Current constraints: {}", constraints);

        Ok(())
    }

    fn list(&self) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        let mut locations = rpc.get_relay_locations()?;
        locations.countries.sort_by(|c1, c2| c1.name.cmp(&c2.name));
        for mut country in locations.countries {
            country.cities.sort_by(|c1, c2| c1.name.cmp(&c2.name));
            println!("{} ({})", country.name, country.code);
            for city in &country.cities {
                println!(
                    "\t{} ({}) @ {:.5}°N, {:.5}°W",
                    city.name, city.code, city.latitude, city.longitude
                );
            }
            println!();
        }
        Ok(())
    }

    fn update(&self) -> Result<()> {
        new_rpc_client()?.update_relay_locations()?;
        println!("Updating relay list in the background...");
        Ok(())
    }
}


fn parse_port_constraint(raw_port: &str) -> Result<Constraint<u16>> {
    match raw_port.to_lowercase().as_str() {
        "any" => Ok(Constraint::Any),
        port => Ok(Constraint::Only(
            u16::from_str(port).chain_err(|| "Invalid port")?,
        )),
    }
}

/// Parses a protocol constraint string. Can be infallible because the possible values are limited
/// with clap.
fn parse_protocol_constraint(raw_protocol: &str) -> Constraint<TransportProtocol> {
    match raw_protocol.to_lowercase().as_str() {
        "any" => Constraint::Any,
        "udp" => Constraint::Only(TransportProtocol::Udp),
        "tcp" => Constraint::Only(TransportProtocol::Tcp),
        _ => unreachable!(),
    }
}

fn country_code_validator(code: String) -> ::std::result::Result<(), String> {
    if code.len() == 2 || code == "any" {
        Ok(())
    } else {
        Err(String::from("Country codes must be two letters, or 'any'."))
    }
}

fn city_code_validator(code: String) -> ::std::result::Result<(), String> {
    if code.len() == 3 {
        Ok(())
    } else {
        Err(String::from("City codes must be three letters"))
    }
}

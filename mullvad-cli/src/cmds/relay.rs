use crate::{location, new_rpc_client, Command, Error, Result};
use clap::{value_t, values_t};
use itertools::Itertools;
use std::{
    convert::TryFrom,
    io::{self, BufRead},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

use mullvad_management_interface::{types, ManagementServiceClient};
use mullvad_types::relay_constraints::{Constraint, RelaySettings};
use talpid_types::net::all_of_the_internet;

pub struct Relay;

#[mullvad_management_interface::async_trait]
impl Command for Relay {
    fn name(&self) -> &'static str {
        "relay"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Manage relay and tunnel constraints")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("set")
                    .about(
                        "Set relay server selection parameters. Such as location and port/protocol",
                    )
                    .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(
                        clap::SubCommand::with_name("custom")
                            .about("Set a custom VPN relay")
                            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                            .subcommand(clap::SubCommand::with_name("wireguard")
                                .arg(
                                    clap::Arg::with_name("host")
                                        .help("Hostname or IP")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::with_name("port")
                                        .help("Remote network port")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::with_name("peer-pubkey")
                                        .help("Base64 encoded peer public key")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::with_name("v4-gateway")
                                        .help("IPv4 gateway address")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::with_name("addr")
                                        .help("Local address of wireguard tunnel")
                                        .required(true)
                                        .multiple(true),
                                )
                                .arg(
                                    clap::Arg::with_name("protocol")
                                        .help("Transport protocol. If TCP is selected, traffic is \
                                               sent over TCP using a udp-over-tcp proxy")
                                        .long("protocol")
                                        .default_value("udp")
                                        .possible_values(&["udp", "tcp"]),
                                )
                                .arg(
                                    clap::Arg::with_name("v6-gateway")
                                        .help("IPv6 gateway address")
                                        .long("v6-gateway")
                                        .takes_value(true),
                                )
                            )
                            .subcommand(clap::SubCommand::with_name("openvpn")
                                .arg(
                                    clap::Arg::with_name("host")
                                        .help("Hostname or IP")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::with_name("port")
                                        .help("Remote network port")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::with_name("username")
                                        .help("Username to be used with the OpenVpn relay")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::with_name("password")
                                        .help("Password to be used with the OpenVpn relay")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::with_name("protocol")
                                        .help("Transport protocol")
                                        .long("protocol")
                                        .default_value("udp")
                                        .possible_values(&["udp", "tcp"]),
                                )
                            )
                    )
                    .subcommand(
                        location::get_subcommand()
                            .about("Set country or city to select relays from. Use the 'list' \
                                   command to show available alternatives.")
                    )
                    .subcommand(
                        clap::SubCommand::with_name("hostname")
                            .about("Set the exact relay to use via its hostname. Shortcut for \
                                'location <country> <city> <hostname>'.")
                            .arg(
                                clap::Arg::with_name("hostname")
                                    .help("The hostname")
                                    .required(true),
                            ),
                    )
                    .subcommand(
                        clap::SubCommand::with_name("provider")
                            .about("Set hosting provider(s) to select relays from. The 'list' \
                                   command shows the available relays and their providers.")
                            .arg(
                                clap::Arg::with_name("provider")
                                .help("The hosting provider(s) to use, or 'any' for no preference.")
                                .multiple(true)
                                .required(true)
                            )
                    )
                    .subcommand(
                        clap::SubCommand::with_name("tunnel")
                            .about("Set tunnel protocol-specific constraints.")
                            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                            .subcommand(
                                clap::SubCommand::with_name("openvpn")
                                    .about("Set OpenVPN-specific constraints")
                                    .setting(clap::AppSettings::ArgRequiredElseHelp)
                                    .arg(
                                        clap::Arg::with_name("port")
                                            .help("Port to use. Either 'any' or a specific port")
                                            .long("port")
                                            .takes_value(true),
                                    )
                                    .arg(
                                        clap::Arg::with_name("transport protocol")
                                            .help("Transport protocol")
                                            .long("protocol")
                                            .possible_values(&["any", "udp", "tcp"])
                                            .takes_value(true),
                                    )
                            )
                            .subcommand(
                                clap::SubCommand::with_name("wireguard")
                                    .about("Set WireGuard-specific constraints")
                                    .setting(clap::AppSettings::ArgRequiredElseHelp)
                                    .arg(
                                        clap::Arg::with_name("port")
                                            .help("Port to use. Either 'any' or a specific port")
                                            .long("port")
                                            .takes_value(true),
                                    )
                                    .arg(
                                        clap::Arg::with_name("transport protocol")
                                            .help("Transport protocol. If TCP is selected, traffic is \
                                                   sent over TCP using a udp-over-tcp proxy")
                                            .long("protocol")
                                            .possible_values(&["any", "udp", "tcp"])
                                            .takes_value(true),
                                    )
                                    .arg(
                                        clap::Arg::with_name("ip version")
                                            .long("ipv")
                                            .possible_values(&["any", "4", "6"])
                                            .takes_value(true),
                                    )
                                    .arg(
                                        clap::Arg::with_name("entry location")
                                            .help("Entry endpoint to use. This can be 'any', 'none', or \
                                                   any location that is valid with 'set location', \
                                                   such as 'se got'.")
                                            .long("entry-location")
                                            .min_values(1)
                                            .max_values(3),
                                    )
                            )
                    )
                    .subcommand(clap::SubCommand::with_name("tunnel-protocol")
                                .about("Set tunnel protocol")
                                .arg(
                                    clap::Arg::with_name("tunnel protocol")
                                    .required(true)
                                    .index(1)
                                    .possible_values(&["any", "wireguard", "openvpn", ]),
                                    )
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

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            self.set(set_matches).await
        } else if matches.subcommand_matches("get").is_some() {
            self.get().await
        } else if matches.subcommand_matches("list").is_some() {
            self.list().await
        } else if matches.subcommand_matches("update").is_some() {
            self.update().await
        } else {
            unreachable!("No relay command given");
        }
    }
}

impl Relay {
    async fn update_constraints(&self, update: types::RelaySettingsUpdate) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.update_relay_settings(update)
            .await
            .map_err(|error| Error::RpcFailedExt("Failed to update relay settings", error))?;
        println!("Relay constraints updated");
        Ok(())
    }

    async fn set(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(custom_matches) = matches.subcommand_matches("custom") {
            self.set_custom(custom_matches).await
        } else if let Some(location_matches) = matches.subcommand_matches("location") {
            self.set_location(location_matches).await
        } else if let Some(relay_matches) = matches.subcommand_matches("hostname") {
            self.set_hostname(relay_matches).await
        } else if let Some(providers_matches) = matches.subcommand_matches("provider") {
            self.set_providers(providers_matches).await
        } else if let Some(matches) = matches.subcommand_matches("tunnel") {
            if let Some(tunnel_matches) = matches.subcommand_matches("openvpn") {
                self.set_openvpn_constraints(tunnel_matches).await
            } else if let Some(tunnel_matches) = matches.subcommand_matches("wireguard") {
                self.set_wireguard_constraints(tunnel_matches).await
            } else {
                unreachable!("Invalid tunnel protocol");
            }
        } else if let Some(tunnel_matches) = matches.subcommand_matches("tunnel-protocol") {
            self.set_tunnel_protocol(tunnel_matches).await
        } else {
            unreachable!("No set relay command given");
        }
    }

    async fn set_custom(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let custom_endpoint = match matches.subcommand() {
            ("openvpn", Some(openvpn_matches)) => Self::read_custom_openvpn_relay(openvpn_matches),
            ("wireguard", Some(wg_matches)) => Self::read_custom_wireguard_relay(wg_matches),
            (_unknown_tunnel, _) => unreachable!("No set relay command given"),
        };

        self.update_constraints(types::RelaySettingsUpdate {
            r#type: Some(types::relay_settings_update::Type::Custom(custom_endpoint)),
        })
        .await
    }

    fn read_custom_openvpn_relay(matches: &clap::ArgMatches<'_>) -> types::CustomRelaySettings {
        let host = value_t!(matches.value_of("host"), String).unwrap_or_else(|e| e.exit());
        let port = value_t!(matches.value_of("port"), u16).unwrap_or_else(|e| e.exit());
        let username = value_t!(matches.value_of("username"), String).unwrap_or_else(|e| e.exit());
        let password = value_t!(matches.value_of("password"), String).unwrap_or_else(|e| e.exit());
        let protocol = value_t!(matches.value_of("protocol"), String).unwrap_or_else(|e| e.exit());

        let protocol = Self::validate_transport_protocol(&protocol);

        types::CustomRelaySettings {
            host,
            config: Some(types::ConnectionConfig {
                config: Some(types::connection_config::Config::Openvpn(
                    types::connection_config::OpenvpnConfig {
                        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)
                            .to_string(),
                        protocol: protocol as i32,
                        username,
                        password,
                    },
                )),
            }),
        }
    }

    fn read_custom_wireguard_relay(matches: &clap::ArgMatches<'_>) -> types::CustomRelaySettings {
        use types::connection_config::wireguard_config;

        let host = value_t!(matches.value_of("host"), String).unwrap_or_else(|e| e.exit());
        let port = value_t!(matches.value_of("port"), u16).unwrap_or_else(|e| e.exit());
        let addresses = values_t!(matches.values_of("addr"), IpAddr).unwrap_or_else(|e| e.exit());
        let peer_key_str =
            value_t!(matches.value_of("peer-pubkey"), String).unwrap_or_else(|e| e.exit());
        let ipv4_gateway =
            value_t!(matches.value_of("v4-gateway"), Ipv4Addr).unwrap_or_else(|e| e.exit());
        let ipv6_gateway = match value_t!(matches.value_of("v6-gateway"), Ipv6Addr) {
            Ok(gateway) => Some(gateway),
            Err(e) => match e.kind {
                clap::ErrorKind::ArgumentNotFound => None,
                _ => e.exit(),
            },
        };
        let protocol = value_t!(matches.value_of("protocol"), String).unwrap_or_else(|e| e.exit());
        let protocol = Self::validate_transport_protocol(&protocol);
        let mut private_key_str = String::new();
        println!("Reading private key from standard input");
        let _ = io::stdin().lock().read_line(&mut private_key_str);
        if private_key_str.trim().is_empty() {
            eprintln!("Expected to read private key from standard input");
        }
        let private_key = Self::validate_wireguard_key(&private_key_str);
        let peer_public_key = Self::validate_wireguard_key(&peer_key_str);

        types::CustomRelaySettings {
            host,
            config: Some(types::ConnectionConfig {
                config: Some(types::connection_config::Config::Wireguard(
                    types::connection_config::WireguardConfig {
                        tunnel: Some(wireguard_config::TunnelConfig {
                            private_key: private_key.to_vec(),
                            addresses: addresses
                                .iter()
                                .map(|address| address.to_string())
                                .collect(),
                        }),
                        peer: Some(wireguard_config::PeerConfig {
                            public_key: peer_public_key.to_vec(),
                            allowed_ips: all_of_the_internet()
                                .iter()
                                .map(|address| address.to_string())
                                .collect(),
                            endpoint: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)
                                .to_string(),
                            protocol: protocol as i32,
                        }),
                        ipv4_gateway: ipv4_gateway.to_string(),
                        ipv6_gateway: ipv6_gateway
                            .as_ref()
                            .map(|addr| addr.to_string())
                            .unwrap_or_default(),
                    },
                )),
            }),
        }
    }

    fn validate_wireguard_key(key_str: &str) -> [u8; 32] {
        let key_bytes = base64::decode(key_str.trim()).unwrap_or_else(|e| {
            eprintln!("Failed to decode wireguard key: {}", e);
            std::process::exit(1);
        });

        let mut key = [0u8; 32];
        if key_bytes.len() != 32 {
            eprintln!(
                "Expected key length to be 32 bytes, got {}",
                key_bytes.len()
            );
            std::process::exit(1);
        }

        key.copy_from_slice(&key_bytes);
        key
    }

    fn validate_transport_protocol(protocol: &str) -> types::TransportProtocol {
        match protocol {
            "udp" => types::TransportProtocol::Udp,
            "tcp" => types::TransportProtocol::Tcp,
            _ => clap::Error::with_description(
                "invalid transport protocol",
                clap::ErrorKind::ValueValidation,
            )
            .exit(),
        }
    }

    async fn set_hostname(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let hostname = matches.value_of("hostname").unwrap();
        let countries = Self::get_filtered_relays().await?;

        let find_relay = || {
            for country in &countries {
                for city in &country.cities {
                    for relay in &city.relays {
                        if relay.hostname == hostname {
                            return Some((country, city, relay));
                        }
                    }
                }
            }
            None
        };

        if let Some(location) = find_relay() {
            println!(
                "Setting location constraint to {} in {}, {}",
                location.2.hostname, location.1.name, location.0.name
            );

            let location_constraint = types::RelayLocation {
                country: location.0.code.clone(),
                city: location.1.code.clone(),
                hostname: location.2.hostname.clone(),
            };

            self.update_constraints(types::RelaySettingsUpdate {
                r#type: Some(types::relay_settings_update::Type::Normal(
                    types::NormalRelaySettingsUpdate {
                        location: Some(location_constraint),
                        ..Default::default()
                    },
                )),
            })
            .await
        } else {
            clap::Error::with_description(
                "No matching server found",
                clap::ErrorKind::ValueValidation,
            )
            .exit()
        }
    }

    async fn set_location(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let location_constraint = location::get_constraint_from_args(matches);
        let mut found = false;

        if !location_constraint.country.is_empty() {
            // TODO: `mullvad_types::relay_constraints::LocationConstraint::matches(&relay)`
            //       could be used to guarantee consistency with the daemon.
            let countries = Self::get_filtered_relays().await?;
            for country in &countries {
                if country.code != location_constraint.country {
                    continue;
                }

                if location_constraint.city.is_empty() {
                    found = true;
                    break;
                }

                for city in &country.cities {
                    if city.code != location_constraint.city {
                        continue;
                    }

                    if location_constraint.hostname.is_empty() {
                        found = true;
                        break;
                    }

                    for relay in &city.relays {
                        if relay.hostname != location_constraint.hostname {
                            continue;
                        }
                        found = true;
                        break;
                    }

                    break;
                }
                break;
            }

            if !found {
                eprintln!("Warning: No matching relay was found.");
            }
        }

        self.update_constraints(types::RelaySettingsUpdate {
            r#type: Some(types::relay_settings_update::Type::Normal(
                types::NormalRelaySettingsUpdate {
                    location: Some(location_constraint),
                    ..Default::default()
                },
            )),
        })
        .await
    }

    async fn set_providers(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let providers =
            values_t!(matches.values_of("provider"), String).unwrap_or_else(|e| e.exit());

        let providers = if providers.iter().next().map(String::as_str) == Some("any") {
            vec![]
        } else {
            providers
        };

        self.update_constraints(types::RelaySettingsUpdate {
            r#type: Some(types::relay_settings_update::Type::Normal(
                types::NormalRelaySettingsUpdate {
                    providers: Some(types::ProviderUpdate { providers }),
                    ..Default::default()
                },
            )),
        })
        .await
    }

    async fn set_openvpn_constraints(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut openvpn_constraints = {
            let mut rpc = new_rpc_client().await?;
            self.get_openvpn_constraints(&mut rpc).await?
        };
        openvpn_constraints.port = parse_transport_port(matches, &mut openvpn_constraints.port)?;

        self.update_constraints(types::RelaySettingsUpdate {
            r#type: Some(types::relay_settings_update::Type::Normal(
                types::NormalRelaySettingsUpdate {
                    openvpn_constraints: Some(openvpn_constraints),
                    ..Default::default()
                },
            )),
        })
        .await
    }

    async fn get_openvpn_constraints(
        &self,
        rpc: &mut ManagementServiceClient,
    ) -> Result<types::OpenvpnConstraints> {
        match rpc
            .get_settings(())
            .await?
            .into_inner()
            .relay_settings
            .unwrap()
            .endpoint
            .unwrap()
        {
            types::relay_settings::Endpoint::Normal(settings) => {
                Ok(settings.openvpn_constraints.unwrap())
            }
            types::relay_settings::Endpoint::Custom(_settings) => {
                println!("Clearing custom tunnel constraints");
                Ok(types::OpenvpnConstraints::default())
            }
        }
    }

    async fn set_wireguard_constraints(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let mut wireguard_constraints = self.get_wireguard_constraints(&mut rpc).await?;

        wireguard_constraints.port =
            parse_transport_port(matches, &mut wireguard_constraints.port)?;

        if let Some(ipv) = matches.value_of("ip version") {
            wireguard_constraints.ip_version =
                parse_ip_version_constraint(ipv).option().map(|protocol| {
                    types::IpVersionConstraint {
                        protocol: protocol as i32,
                    }
                });
        }
        if let Some(entry) = matches.values_of("entry location") {
            wireguard_constraints.entry_location = parse_entry_location_constraint(entry);
        }

        self.update_constraints(types::RelaySettingsUpdate {
            r#type: Some(types::relay_settings_update::Type::Normal(
                types::NormalRelaySettingsUpdate {
                    wireguard_constraints: Some(wireguard_constraints),
                    ..Default::default()
                },
            )),
        })
        .await
    }

    async fn get_wireguard_constraints(
        &self,
        rpc: &mut ManagementServiceClient,
    ) -> Result<types::WireguardConstraints> {
        match rpc
            .get_settings(())
            .await?
            .into_inner()
            .relay_settings
            .unwrap()
            .endpoint
            .unwrap()
        {
            types::relay_settings::Endpoint::Normal(settings) => {
                Ok(settings.wireguard_constraints.unwrap())
            }
            types::relay_settings::Endpoint::Custom(_settings) => {
                println!("Clearing custom tunnel constraints");
                Ok(types::WireguardConstraints::default())
            }
        }
    }

    async fn set_tunnel_protocol(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let tunnel_type = match matches.value_of("tunnel protocol").unwrap() {
            "wireguard" => Some(types::TunnelType::Wireguard),
            "openvpn" => Some(types::TunnelType::Openvpn),
            "any" => None,
            _ => unreachable!(),
        };
        self.update_constraints(types::RelaySettingsUpdate {
            r#type: Some(types::relay_settings_update::Type::Normal(
                types::NormalRelaySettingsUpdate {
                    tunnel_type: Some(types::TunnelTypeUpdate {
                        tunnel_type: tunnel_type.map(|tunnel_type| types::TunnelTypeConstraint {
                            tunnel_type: tunnel_type as i32,
                        }),
                    }),
                    ..Default::default()
                },
            )),
        })
        .await
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let relay_settings = rpc
            .get_settings(())
            .await?
            .into_inner()
            .relay_settings
            .unwrap();

        println!(
            "Current constraints: {}",
            RelaySettings::try_from(relay_settings).unwrap()
        );

        Ok(())
    }

    async fn list(&self) -> Result<()> {
        let mut countries = Self::get_filtered_relays().await?;
        countries.sort_by(|c1, c2| natord::compare_ignore_case(&c1.name, &c2.name));
        for mut country in countries {
            country
                .cities
                .sort_by(|c1, c2| natord::compare_ignore_case(&c1.name, &c2.name));
            println!("{} ({})", country.name, country.code);
            for mut city in country.cities {
                city.relays
                    .sort_by(|r1, r2| natord::compare_ignore_case(&r1.hostname, &r2.hostname));
                println!(
                    "\t{} ({}) @ {:.5}°N, {:.5}°W",
                    city.name, city.code, city.latitude, city.longitude
                );
                for relay in &city.relays {
                    let tunnels = relay.tunnels.as_ref().unwrap();
                    let supports_openvpn = !tunnels.openvpn.is_empty();
                    let supports_wireguard = !tunnels.wireguard.is_empty();
                    let support_msg = match (supports_openvpn, supports_wireguard) {
                        (true, true) => "OpenVPN and WireGuard",
                        (true, false) => "OpenVPN",
                        (false, true) => "WireGuard",
                        _ => unreachable!("Bug in relay filtering earlier on"),
                    };
                    let mut addresses = vec![&relay.ipv4_addr_in];
                    if !relay.ipv6_addr_in.is_empty() {
                        addresses.push(&relay.ipv6_addr_in);
                    }
                    println!(
                        "\t\t{} ({}) - {}, hosted by {}",
                        relay.hostname,
                        addresses.iter().join(", "),
                        support_msg,
                        relay.provider
                    );
                }
            }
            println!();
        }
        Ok(())
    }

    async fn update(&self) -> Result<()> {
        new_rpc_client().await?.update_relay_locations(()).await?;
        println!("Updating relay list in the background...");
        Ok(())
    }

    async fn get_filtered_relays() -> Result<Vec<types::RelayListCountry>> {
        let mut rpc = new_rpc_client().await?;
        let mut locations = rpc
            .get_relay_locations(())
            .await
            .map_err(|error| Error::RpcFailedExt("Failed to obtain relay locations", error))?
            .into_inner();

        let mut countries = Vec::new();

        while let Some(mut country) = locations.message().await? {
            country.cities = country
                .cities
                .into_iter()
                .filter_map(|mut city| {
                    city.relays.retain(|relay| {
                        relay.active
                            && relay.tunnels.is_some()
                            && !(relay.tunnels.as_ref().unwrap().openvpn.is_empty()
                                && relay.tunnels.as_ref().unwrap().wireguard.is_empty())
                    });
                    if !city.relays.is_empty() {
                        Some(city)
                    } else {
                        None
                    }
                })
                .collect();
            if !country.cities.is_empty() {
                countries.push(country);
            }
        }

        Ok(countries)
    }
}


fn parse_port_constraint(raw_port: &str) -> Result<Constraint<u16>> {
    match raw_port.to_lowercase().as_str() {
        "any" => Ok(Constraint::Any),
        port => Ok(Constraint::Only(u16::from_str(port).map_err(|_| {
            Error::InvalidCommand("Invalid port. Must be \"any\" or [0-65535].")
        })?)),
    }
}

fn parse_protocol(raw_protocol: &str) -> Constraint<types::TransportProtocol> {
    match raw_protocol {
        "any" => Constraint::Any,
        "udp" => Constraint::Only(types::TransportProtocol::Udp),
        "tcp" => Constraint::Only(types::TransportProtocol::Tcp),
        _ => unreachable!(),
    }
}

fn parse_ip_version_constraint(raw_protocol: &str) -> Constraint<types::IpVersion> {
    match raw_protocol {
        "any" => Constraint::Any,
        "4" => Constraint::Only(types::IpVersion::V4),
        "6" => Constraint::Only(types::IpVersion::V6),
        _ => unreachable!(),
    }
}

fn parse_entry_location_constraint<'a, T: Iterator<Item = &'a str>>(
    mut location: T,
) -> Option<types::RelayLocation> {
    let country = location.next().unwrap();

    if country == "none" {
        return None;
    }

    Some(location::get_constraint(
        country,
        location.next(),
        location.next(),
    ))
}

fn parse_transport_port(
    matches: &clap::ArgMatches<'_>,
    current_constraint: &mut Option<types::TransportPort>,
) -> Result<Option<types::TransportPort>> {
    let protocol = match matches.value_of("transport protocol") {
        Some(protocol) => parse_protocol(protocol),
        None => {
            if let Some(ref transport_port) = current_constraint {
                Constraint::Only(
                    types::TransportProtocol::from_i32(transport_port.protocol).unwrap(),
                )
            } else {
                Constraint::Any
            }
        }
    };
    let mut port = match matches.value_of("port") {
        Some(port) => parse_port_constraint(port)?,
        None => {
            if let Some(ref transport_port) = current_constraint {
                if transport_port.port != 0 {
                    Constraint::Only(transport_port.port as u16)
                } else {
                    Constraint::Any
                }
            } else {
                Constraint::Any
            }
        }
    };
    if port.is_only() && protocol.is_any() && !matches.is_present("port") {
        // Reset the port if the transport protocol is set to any.
        println!("The port constraint was set to 'any'");
        port = Constraint::Any;
    }
    match (port, protocol) {
        (Constraint::Any, Constraint::Any) => Ok(None),
        (Constraint::Any, Constraint::Only(protocol)) => Ok(Some(types::TransportPort {
            protocol: protocol as i32,
            // If no port was specified, set it to "any"
            ..types::TransportPort::default()
        })),
        (Constraint::Only(port), Constraint::Only(protocol)) => Ok(Some(types::TransportPort {
            protocol: protocol as i32,
            port: u32::from(port),
        })),
        (Constraint::Only(_), Constraint::Any) => Err(Error::InvalidCommand(
            "a transport protocol must be given to select a specific port",
        )),
    }
}

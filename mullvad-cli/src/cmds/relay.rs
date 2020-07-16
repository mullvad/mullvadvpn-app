use crate::{location, new_grpc_client, proto, Command, Error, Result};
use clap::{value_t, values_t};
use std::{
    io::{self, BufRead},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

use mullvad_types::relay_constraints::Constraint;
use proto::{
    connection_config::{self, OpenvpnConfig, WireguardConfig},
    relay_settings_update,
    normal_relay_settings::{
        OpenvpnConstraints,
        WireguardConstraints,
    },
    ConnectionConfig,
    CustomRelaySettings,
    NormalRelaySettings,
    RelayLocation,
    RelaySettingsUpdate,
    TransportProtocol,
    TunnelType,
};
use talpid_types::net::all_of_the_internet;

pub struct Relay;

#[async_trait::async_trait]
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
                                        .required(true)
                                        .index(1),
                                )
                                .arg(
                                    clap::Arg::with_name("port")
                                        .help("Remote network port")
                                        .required(true)
                                        .index(2),
                                )
                                .arg(
                                    clap::Arg::with_name("peer-key")
                                        .help("Base64 encoded peer public key")
                                        .index(3)
                                        .required(false),
                                )
                                .arg(
                                    clap::Arg::with_name("v4-gateway")
                                        .help("IPv4 gateway address")
                                        .long("v4-gateway")
                                        .index(4)
                                        .required(false),
                                ).arg(
                                    clap::Arg::with_name("v6-gateway")
                                        .help("IPv6 gateway address")
                                        .long("v6-gateway")
                                        .takes_value(true)
                                        .required(false),
                                )
                                .arg(
                                    clap::Arg::with_name("addr")
                                        .help("Local address of wireguard tunnel")
                                        .long("addr")
                                        .takes_value(true)
                                        .multiple(true)
                                        .required(false),
                                ),
                            )
                            .subcommand(clap::SubCommand::with_name("openvpn")
                                .arg(
                                    clap::Arg::with_name("host")
                                        .help("Hostname or IP")
                                        .required(true)
                                        .index(1),
                                )
                                .arg(
                                    clap::Arg::with_name("port")
                                        .help("Remote network port")
                                        .required(true)
                                        .index(2),
                                )
                                .arg(
                                    clap::Arg::with_name("protocol")
                                        .help("Transport protocol. For Wireguard this is ignored.")
                                        .index(3)
                                        .default_value("udp")
                                        .possible_values(&["udp", "tcp"]),
                                )
                                .arg(
                                    clap::Arg::with_name("username")
                                        .help("Username to be used with the OpenVpn relay")
                                        .index(4),
                                )
                                .arg(
                                    clap::Arg::with_name("password")
                                        .help("Password to be used with the OpenVpn relay")
                                        .index(5),
                                )
                            )
                    )
                    .subcommand(
                        location::get_subcommand()
                            .about("Set country or city to select relays from. Use the 'list' \
                                   command to show available alternatives.")
                    )
                    .subcommand(
                        clap::SubCommand::with_name("tunnel")
                            .about("Set individual tunnel constraints")
                            .arg(
                                clap::Arg::with_name("vpn protocol")
                                    .required(true)
                                    .index(1)
                                    .possible_values(&["wireguard", "openvpn"]),
                            )
                            .arg(clap::Arg::with_name("port").required(true).index(2))
                            .arg(
                                clap::Arg::with_name("transport protocol")
                                    .long("protocol")
                                    .required(false)
                                    .default_value("any")
                                    .possible_values(&["any", "udp", "tcp"]),
                            ),

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
    async fn update_constraints(&self, update: RelaySettingsUpdate) -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        rpc.update_relay_settings(update).await.map_err(Error::GrpcClientError)?;
        println!("Relay constraints updated");
        Ok(())
    }

    async fn set(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(custom_matches) = matches.subcommand_matches("custom") {
            self.set_custom(custom_matches).await
        } else if let Some(location_matches) = matches.subcommand_matches("location") {
            self.set_location(location_matches).await
        } else if let Some(tunnel_matches) = matches.subcommand_matches("tunnel") {
            self.set_tunnel(tunnel_matches).await
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
        
        self.update_constraints(RelaySettingsUpdate {
            r#type: Some(relay_settings_update::Type::Custom(custom_endpoint))
        }).await
    }

    fn read_custom_openvpn_relay(matches: &clap::ArgMatches<'_>) -> CustomRelaySettings {
        let host = value_t!(matches.value_of("host"), String).unwrap_or_else(|e| e.exit());
        let port = value_t!(matches.value_of("port"), u16).unwrap_or_else(|e| e.exit());
        let username = value_t!(matches.value_of("username"), String).unwrap_or_else(|e| e.exit());
        let password = value_t!(matches.value_of("password"), String).unwrap_or_else(|e| e.exit());
        let protocol =
            value_t!(matches.value_of("protocol"), String).unwrap_or_else(|e| e.exit());

        let protocol = match protocol.as_str() {
            "udp" => TransportProtocol::Udp,
            "tcp" => TransportProtocol::Tcp,
            _ => clap::Error::with_description(
                "unknown transport protocol",
                clap::ErrorKind::ValueValidation,
            ).exit(),
        };

        CustomRelaySettings {
            host,
            config: Some(ConnectionConfig {
                config: Some(connection_config::Config::Openvpn(OpenvpnConfig {
                    address: SocketAddr::new(
                        IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                        port,
                    ).to_string(),
                    protocol: protocol as i32,
                    username,
                    password,
                }))
            }),
        }
    }

    fn read_custom_wireguard_relay(matches: &clap::ArgMatches<'_>) -> CustomRelaySettings {
        use connection_config::wireguard_config;

        let host = value_t!(matches.value_of("host"), String).unwrap_or_else(|e| e.exit());
        let port = value_t!(matches.value_of("port"), u16).unwrap_or_else(|e| e.exit());
        let addresses = values_t!(matches.values_of("addr"), IpAddr).unwrap_or_else(|e| e.exit());
        let peer_key_str =
            value_t!(matches.value_of("peer-key"), String).unwrap_or_else(|e| e.exit());
        let ipv4_gateway =
            value_t!(matches.value_of("v4-gateway"), Ipv4Addr).unwrap_or_else(|e| e.exit());
        let ipv6_gateway = match value_t!(matches.value_of("v6-gateway"), Ipv6Addr) {
            Ok(gateway) => Some(gateway),
            Err(e) => match e.kind {
                clap::ErrorKind::ArgumentNotFound => None,
                _ => e.exit(),
            },
        };
        let mut private_key_str = String::new();
        println!("Reading private key from standard input");
        let _ = io::stdin().lock().read_line(&mut private_key_str);
        if private_key_str.trim().is_empty() {
            eprintln!("Expected to read private key from standard input");
        }
        let private_key = Self::validate_wireguard_key(&private_key_str);
        let peer_public_key = Self::validate_wireguard_key(&peer_key_str);

        CustomRelaySettings {
            host,
            config: Some(ConnectionConfig {
                config: Some(connection_config::Config::Wireguard(WireguardConfig {
                    tunnel: Some(wireguard_config::TunnelConfig {
                        private_key: private_key.to_vec(),
                        addresses: addresses.iter().map(|address| address.to_string()).collect(),
                    }),
                    peer: Some(wireguard_config::PeerConfig {
                        public_key: peer_public_key.to_vec(),
                        allowed_ips: all_of_the_internet().iter().map(|address| address.to_string()).collect(),
                        endpoint: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port).to_string(),
                    }),
                    ipv4_gateway: ipv4_gateway.to_string(),
                    ipv6_gateway: ipv6_gateway.as_ref().map(|addr| addr.to_string()).unwrap_or_default(),
                }))
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

    async fn set_location(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let location_constraint = location::get_constraint(matches);

        self.update_constraints(RelaySettingsUpdate {
            r#type: Some(relay_settings_update::Type::Normal(NormalRelaySettings {
                location: Some(RelayLocation {
                    hostname: location_constraint
                }),
                ..Default::default()
            }))
        }).await
    }

    async fn set_tunnel(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let vpn_protocol = matches.value_of("vpn protocol").unwrap();
        let port = parse_port_constraint(matches.value_of("port").unwrap())?;
        let protocol = parse_protocol_constraint(matches.value_of("transport protocol").unwrap());

        match vpn_protocol {
            "wireguard" => {
                if let Constraint::Only(TransportProtocol::Tcp) = protocol {
                    return Err(Error::InvalidCommand("WireGuard does not support TCP"));
                }
                self.update_constraints(RelaySettingsUpdate {
                    r#type: Some(relay_settings_update::Type::Normal(NormalRelaySettings {
                        wireguard_constraints: Some(WireguardConstraints { port: port.unwrap_or(0) as u32 }),
                        ..Default::default()
                    }))
                }).await
            }
            "openvpn" => {
                self.update_constraints(RelaySettingsUpdate {
                    r#type: Some(relay_settings_update::Type::Normal(NormalRelaySettings {
                        openvpn_constraints: Some(OpenvpnConstraints {
                            port: port.unwrap_or(0) as u32,
                            protocol: protocol.unwrap_or(TransportProtocol::AnyProtocol) as i32,
                        }),
                        ..Default::default()
                    }))
                }).await
            }
            _ => unreachable!(),
        }
    }

    async fn set_tunnel_protocol(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let tunnel_type = match matches.value_of("tunnel protocol").unwrap() {
            "wireguard" => TunnelType::Wireguard,
            "openvpn" => TunnelType::Openvpn,
            "any" => TunnelType::AnyTunnel,
            _ => unreachable!(),
        };
        self.update_constraints(RelaySettingsUpdate {
            r#type: Some(relay_settings_update::Type::Normal(NormalRelaySettings {
                tunnel_type: tunnel_type as i32,
                ..Default::default()
            }))
        }).await
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        let constraints = rpc.get_settings(())
            .await
            .map_err(Error::GrpcClientError)?
            .into_inner()
            .relay_settings;
        println!("Current constraints: {:?}", constraints);

        Ok(())
    }

    async fn list(&self) -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        let mut locations = rpc.get_relay_locations(())
            .await
            .map_err(Error::GrpcClientError)?
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
                    println!(
                        "\t\t{} ({}) - {}",
                        relay.hostname, relay.ipv4_addr_in, support_msg
                    );
                }
            }
            println!();
        }
        Ok(())
    }

    async fn update(&self) -> Result<()> {
        new_grpc_client().await?
            .update_relay_locations(())
            .await
            .map_err(Error::GrpcClientError)?;
        println!("Updating relay list in the background...");
        Ok(())
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

/// Parses a protocol constraint string. Can be infallible because the possible values are limited
/// with clap.
fn parse_protocol_constraint(raw_protocol: &str) -> Constraint<TransportProtocol> {
    match raw_protocol {
        "any" => Constraint::Any,
        "udp" => Constraint::Only(TransportProtocol::Udp),
        "tcp" => Constraint::Only(TransportProtocol::Tcp),
        _ => unreachable!(),
    }
}

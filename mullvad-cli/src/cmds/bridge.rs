use crate::{location, new_grpc_client, Command, Error, Result};
use clap::value_t;

use crate::proto::{
    bridge_settings::{Type as BridgeSettingsType, *},
    bridge_state::State as BridgeStateType,
    BridgeSettings, BridgeState,
};
use talpid_types::net::openvpn::SHADOWSOCKS_CIPHERS;

use std::net::{IpAddr, SocketAddr};

pub struct Bridge;

#[async_trait::async_trait]
impl Command for Bridge {
    fn name(&self) -> &'static str {
        "bridge"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Manage use of bridges")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(create_bridge_set_subcommand())
            .subcommand(
                clap::SubCommand::with_name("get").about("Get current bridge settings and state"),
            )
            .subcommand(clap::SubCommand::with_name("list").about("List bridge relays"))
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("set", Some(set_matches)) => Self::handle_set(set_matches).await,
            ("get", _) => Self::handle_get().await,
            ("list", _) => Self::list_bridge_relays().await,
            _ => unreachable!("unhandled command"),
        }
    }
}

fn create_bridge_set_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("set")
        .about("Set bridge state and settings")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(create_set_state_subcommand())
        .subcommand(create_set_custom_settings_subcommand())
        .subcommand(location::get_subcommand().about(
            "Set country or city to select bridge relays from. Use the 'list' \
             command to show available alternatives.",
        ))
}


fn create_set_custom_settings_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("custom")
        .about("Configure a SOCKS5 proxy")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
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

fn create_set_state_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("state")
        .about("Set bridge state")
        .arg(
            clap::Arg::with_name("state")
                .help("Specifies whether a bridge should be used")
                .required(true)
                .index(1)
                .possible_values(&["auto", "on", "off"]),
        )
}

impl Bridge {
    async fn handle_set(matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("location", Some(location_matches)) => {
                Self::handle_set_bridge_location(location_matches).await
            }
            ("custom", Some(custom_matches)) => {
                Self::handle_bridge_set_custom_settings(custom_matches).await
            }
            ("state", Some(set_matches)) => Self::handle_set_bridge_state(set_matches).await,
            _ => unreachable!("unhandled command"),
        }
    }

    async fn handle_get() -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        let settings = rpc
            .get_settings(())
            .await
            .map_err(Error::GrpcClientError)?
            .into_inner();
        Self::print_state(settings.bridge_state.unwrap());
        match settings.bridge_settings.unwrap().r#type.unwrap() {
            BridgeSettingsType::Local(local_proxy) => Self::print_local_proxy(&local_proxy),
            BridgeSettingsType::Remote(remote_proxy) => Self::print_remote_proxy(&remote_proxy),
            BridgeSettingsType::Shadowsocks(shadowsocks_proxy) => {
                Self::print_shadowsocks_proxy(&shadowsocks_proxy)
            }
            BridgeSettingsType::Normal(constraints) => {
                println!("Bridge constraints: {:?}", constraints);
            }
        };
        Ok(())
    }

    async fn handle_set_bridge_location(matches: &clap::ArgMatches<'_>) -> Result<()> {
        let hostname = location::get_constraint(matches);
        let mut rpc = new_grpc_client().await?;
        rpc.set_bridge_settings(BridgeSettings {
            r#type: Some(BridgeSettingsType::Normal(BridgeConstraints {
                location: Some(crate::proto::RelayLocation { hostname }),
            })),
        })
        .await
        .map_err(Error::GrpcClientError)?;
        Ok(())
    }

    async fn handle_set_bridge_state(matches: &clap::ArgMatches<'_>) -> Result<()> {
        let state = match matches.value_of("state").unwrap() {
            "auto" => BridgeStateType::Auto as i32,
            "on" => BridgeStateType::On as i32,
            "off" => BridgeStateType::Off as i32,
            _ => unreachable!(),
        };
        let mut rpc = new_grpc_client().await?;
        rpc.set_bridge_state(BridgeState { state })
            .await
            .map_err(Error::GrpcClientError)?;
        Ok(())
    }

    async fn handle_bridge_set_custom_settings(matches: &clap::ArgMatches<'_>) -> Result<()> {
        use talpid_types::net::openvpn;

        if let Some(args) = matches.subcommand_matches("local") {
            let local_port =
                value_t!(args.value_of("local-port"), u16).unwrap_or_else(|e| e.exit());
            let remote_ip =
                value_t!(args.value_of("remote-ip"), IpAddr).unwrap_or_else(|e| e.exit());
            let remote_port =
                value_t!(args.value_of("remote-port"), u16).unwrap_or_else(|e| e.exit());

            let local_proxy = openvpn::LocalProxySettings {
                port: local_port,
                peer: SocketAddr::new(remote_ip, remote_port),
            };
            let prost_proxy = LocalProxySettings {
                port: local_proxy.port as u32,
                peer: local_proxy.peer.to_string(),
            };
            let packed_proxy = openvpn::ProxySettings::Local(local_proxy);
            if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
                panic!(error);
            }

            let mut rpc = new_grpc_client().await?;
            rpc.set_bridge_settings(BridgeSettings {
                r#type: Some(BridgeSettingsType::Local(prost_proxy)),
            })
            .await
            .map_err(Error::GrpcClientError)?;
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
            let prost_auth = auth.clone().map(|auth| RemoteProxyAuth {
                username: auth.username.clone(),
                password: auth.password.clone(),
            });

            let proxy = openvpn::RemoteProxySettings {
                address: SocketAddr::new(remote_ip, remote_port),
                auth,
            };
            let prost_proxy = RemoteProxySettings {
                address: proxy.address.to_string(),
                auth: prost_auth,
            };

            let packed_proxy = openvpn::ProxySettings::Remote(proxy);
            if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
                panic!(error);
            }

            let mut rpc = new_grpc_client().await?;
            rpc.set_bridge_settings(BridgeSettings {
                r#type: Some(BridgeSettingsType::Remote(prost_proxy)),
            })
            .await
            .map_err(Error::GrpcClientError)?;
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
            let prost_proxy = ShadowsocksProxySettings {
                peer: proxy.peer.to_string(),
                password: proxy.password.clone(),
                cipher: proxy.cipher.clone(),
            };

            let packed_proxy = openvpn::ProxySettings::Shadowsocks(proxy);
            if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
                panic!(error);
            }

            let mut rpc = new_grpc_client().await?;
            rpc.set_bridge_settings(BridgeSettings {
                r#type: Some(BridgeSettingsType::Shadowsocks(prost_proxy)),
            })
            .await
            .map_err(Error::GrpcClientError)?;
        } else {
            unreachable!("unhandled proxy type");
        }

        println!("proxy details have been updated");
        Ok(())
    }

    fn print_state(state: BridgeState) {
        let state = match BridgeStateType::from_i32(state.state).expect("unknown bridge state") {
            BridgeStateType::Auto => "auto",
            BridgeStateType::On => "on",
            BridgeStateType::Off => "off",
        };
        println!("Bridge state - {}", state);
    }

    fn print_local_proxy(proxy: &LocalProxySettings) {
        println!("proxy: local");
        println!("  local port: {}", proxy.port);
        println!("  peer address: {}", proxy.peer);
    }

    fn print_remote_proxy(proxy: &RemoteProxySettings) {
        println!("proxy: remote");
        println!("  server address: {}", proxy.address);

        if let Some(ref auth) = proxy.auth {
            println!("  auth username: {}", auth.username);
            println!("  auth password: {}", auth.password);
        } else {
            println!("  auth: none");
        }
    }

    fn print_shadowsocks_proxy(proxy: &ShadowsocksProxySettings) {
        println!("proxy: Shadowsocks");
        println!("  peer address: {}", proxy.peer);
        println!("  password: {}", proxy.password);
        println!("  cipher: {}", proxy.cipher);
    }

    async fn list_bridge_relays() -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        let mut locations = rpc
            .get_relay_locations(())
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
                            && relay.bridges.is_some()
                            && !relay.bridges.as_ref().unwrap().shadowsocks.is_empty()
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
                    println!("\t\t{} ({})", relay.hostname, relay.ipv4_addr_in);
                }
            }
            println!();
        }
        Ok(())
    }
}

use crate::{new_rpc_client, Command, Result};
use clap::value_t_or_exit;
use mullvad_management_interface::types;

pub struct CustomDns;

#[mullvad_management_interface::async_trait]
impl Command for CustomDns {
    fn name(&self) -> &'static str {
        "custom-dns"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Configure custom DNS servers to use when connected")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("servers")
                    .about("Set custom DNS servers to use")
                    .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(
                        clap::SubCommand::with_name("set")
                            .about("Set custom DNS servers to use")
                            .arg(
                                clap::Arg::with_name("servers")
                                    .multiple(true)
                                    .help("One or more IP addresses pointing to DNS resolvers.")
                                    .required(true),
                            ),
                    )
                    .subcommand(
                        clap::SubCommand::with_name("clear").about("Remove all custom DNS servers"),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("get").about("Display the current custom DNS settings"),
            )
            .subcommand(
                clap::SubCommand::with_name("set")
                    .about("Enable or disable custom DNS")
                    .arg(
                        clap::Arg::with_name("enabled")
                            .required(true)
                            .possible_values(&["on", "off"]),
                    ),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("servers", Some(matches)) => match matches.subcommand() {
                ("set", Some(matches)) => {
                    self.set_servers(matches.values_of_lossy("servers")).await
                }
                ("clear", _) => self.clear_servers().await,
                _ => unreachable!("No custom-dns server command given"),
            },
            ("set", Some(matches)) => {
                let enabled = value_t_or_exit!(matches.value_of("enabled"), String);
                self.set_state(enabled == "on").await
            }
            ("get", _) => self.get().await,
            _ => unreachable!("No custom-dns command given"),
        }
    }
}

impl CustomDns {
    async fn set_state(&self, enabled: bool) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let options = rpc
            .get_settings(())
            .await?
            .into_inner()
            .tunnel_options
            .unwrap()
            .dns_options
            .unwrap();
        rpc.set_dns_options(types::DnsOptions {
            custom: enabled,
            addresses: options.addresses,
        })
        .await?;
        println!("Updated custom DNS settings");
        Ok(())
    }

    async fn set_servers(&self, servers: Option<Vec<String>>) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let options = rpc
            .get_settings(())
            .await?
            .into_inner()
            .tunnel_options
            .unwrap()
            .dns_options
            .unwrap();
        rpc.set_dns_options(types::DnsOptions {
            custom: options.custom,
            addresses: servers.unwrap_or_default(),
        })
        .await?;
        println!("Updated custom DNS settings");
        Ok(())
    }

    async fn clear_servers(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let options = rpc
            .get_settings(())
            .await?
            .into_inner()
            .tunnel_options
            .unwrap()
            .dns_options
            .unwrap();
        rpc.set_dns_options(types::DnsOptions {
            custom: options.custom,
            addresses: vec![],
        })
        .await?;
        println!("Cleared list of custom DNS servers");
        Ok(())
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let options = rpc
            .get_settings(())
            .await?
            .into_inner()
            .tunnel_options
            .unwrap()
            .dns_options
            .unwrap();

        let state = if options.custom {
            "enabled"
        } else {
            "disabled"
        };
        println!("Custom DNS: {}", state);

        match options.addresses.len() {
            0 => println!("No DNS servers are configured"),
            _ => {
                println!("Servers:");
                for server in &options.addresses {
                    println!("\t{}", server);
                }
            }
        }

        Ok(())
    }
}

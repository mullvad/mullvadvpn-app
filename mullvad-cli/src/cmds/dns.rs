use crate::{new_rpc_client, Command, Result};
use mullvad_management_interface::types;

pub struct Dns;

#[mullvad_management_interface::async_trait]
impl Command for Dns {
    fn name(&self) -> &'static str {
        "dns"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Configure DNS servers to use when connected")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("get").about("Display the current DNS settings"),
            )
            .subcommand(
                clap::SubCommand::with_name("set")
                    .about("Set DNS servers to use")
                    .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(
                        clap::SubCommand::with_name("custom")
                            .about("Set a list of custom DNS servers")
                            .arg(
                                clap::Arg::with_name("servers")
                                    .multiple(true)
                                    .help("One or more IP addresses pointing to DNS resolvers.")
                                    .required(true),
                            ),
                    )
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("set", Some(matches)) => {
                match matches.subcommand() {
                    ("custom", Some(matches)) => {
                        self.set_custom(matches.values_of_lossy("servers")).await
                    }
                    _ => unreachable!("No custom-dns server command given"),
                }
            }
            ("get", _) => self.get().await,
            _ => unreachable!("No custom-dns command given"),
        }
    }
}

impl Dns {
    async fn set_default(&self, servers: Option<Vec<String>>) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.set_dns_options(types::DnsOptions {
            custom: true,
            addresses: servers.unwrap_or_default(),
        })
        .await?;
        println!("Updated DNS settings");
        Ok(())
    }

    async fn set_custom(&self, servers: Option<Vec<String>>) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.set_dns_options(types::DnsOptions {
            custom: true,
            addresses: servers.unwrap_or_default(),
        })
        .await?;
        println!("Updated DNS settings");
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

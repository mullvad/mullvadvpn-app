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
                        clap::SubCommand::with_name("default")
                            .about("Use default DNS servers")
                            .arg(
                                clap::Arg::with_name("block ads")
                                    .long("block-ads")
                                    .takes_value(false)
                                    .help("Block domain names used for ads"),
                            )
                            .arg(
                                clap::Arg::with_name("block trackers")
                                    .long("block-trackers")
                                    .takes_value(false)
                                    .help("Block domain names used for tracking"),
                            ),
                    )
                    .subcommand(
                        clap::SubCommand::with_name("custom")
                            .about("Set a list of custom DNS servers")
                            .arg(
                                clap::Arg::with_name("servers")
                                    .multiple(true)
                                    .help("One or more IP addresses pointing to DNS resolvers.")
                                    .required(true),
                            ),
                    ),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("set", Some(matches)) => match matches.subcommand() {
                ("default", Some(matches)) => {
                    self.set_default(
                        matches.is_present("block ads"),
                        matches.is_present("block trackers"),
                    )
                    .await
                }
                ("custom", Some(matches)) => {
                    self.set_custom(matches.values_of_lossy("servers")).await
                }
                _ => unreachable!("No custom-dns server command given"),
            },
            ("get", _) => self.get().await,
            _ => unreachable!("No custom-dns command given"),
        }
    }
}

impl Dns {
    async fn set_default(&self, block_ads: bool, block_trackers: bool) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.set_dns_options(types::DnsOptions {
            r#type: Some(types::dns_options::Type::Default(
                types::DefaultDnsOptions {
                    block_ads,
                    block_trackers,
                },
            )),
        })
        .await?;
        println!("Updated DNS settings");
        Ok(())
    }

    async fn set_custom(&self, servers: Option<Vec<String>>) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.set_dns_options(types::DnsOptions {
            r#type: Some(types::dns_options::Type::Custom(types::CustomDnsOptions {
                addresses: servers.unwrap_or_default(),
            })),
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

        println!("DNS: {:?}", options);

        Ok(())
    }
}

use crate::{new_rpc_client, Command, Result};
use mullvad_management_interface::types;
use mullvad_types::settings::{DnsOptions, DnsState};
use std::convert::TryInto;

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
                            )
                            .arg(
                                clap::Arg::with_name("block malware")
                                    .long("block-malware")
                                    .takes_value(false)
                                    .help("Block domains known to be used by malware"),
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
                        matches.is_present("block malware"),
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
    async fn set_default(
        &self,
        block_ads: bool,
        block_trackers: bool,
        block_malware: bool,
    ) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let settings = rpc.get_settings(()).await?.into_inner();
        rpc.set_dns_options(types::DnsOptions {
            state: types::dns_options::DnsState::Default as i32,
            default_options: Some(types::DefaultDnsOptions {
                block_ads,
                block_trackers,
                block_malware,
            }),
            ..settings.tunnel_options.unwrap().dns_options.unwrap()
        })
        .await?;
        println!("Updated DNS settings");
        Ok(())
    }

    async fn set_custom(&self, servers: Option<Vec<String>>) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let settings = rpc.get_settings(()).await?.into_inner();
        rpc.set_dns_options(types::DnsOptions {
            state: types::dns_options::DnsState::Custom as i32,
            custom_options: Some(types::CustomDnsOptions {
                addresses: servers.unwrap_or_default(),
            }),
            ..settings.tunnel_options.unwrap().dns_options.unwrap()
        })
        .await?;
        println!("Updated DNS settings");
        Ok(())
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let options: DnsOptions = rpc
            .get_settings(())
            .await?
            .into_inner()
            .tunnel_options
            .unwrap()
            .dns_options
            .unwrap()
            .try_into()
            .unwrap();

        match options.state {
            DnsState::Default => {
                println!("Custom DNS: no");
                println!("Block ads: {}", options.default_options.block_ads);
                println!("Block trackers: {}", options.default_options.block_trackers);
                println!("Block malware: {}", options.default_options.block_malware);
            }
            DnsState::Custom => {
                println!("Custom DNS: yes\nServers:");
                for server in &options.custom_options.addresses {
                    println!("{}", server);
                }
            }
        }

        Ok(())
    }
}

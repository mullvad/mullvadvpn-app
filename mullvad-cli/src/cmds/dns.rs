use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::settings::{CustomDnsOptions, DefaultDnsOptions, DnsOptions, DnsState};
use std::net::IpAddr;

#[derive(Subcommand, Debug)]
pub enum Dns {
    /// Display the current DNS settings
    Get,

    /// Set DNS servers to use
    Set {
        #[clap(subcommand)]
        cmd: DnsSet,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum DnsSet {
    /// Use a default DNS server, with or without content
    /// blocking.
    Default {
        /// Block domains known to be used for ads
        #[arg(long)]
        block_ads: bool,

        /// Block domains known to be used for tracking
        #[arg(long)]
        block_trackers: bool,

        /// Block domains known to be used by malware
        #[arg(long)]
        block_malware: bool,

        /// Block domains known to be used for adult content
        #[arg(long)]
        block_adult_content: bool,

        /// Block domains known to be used for gambling
        #[arg(long)]
        block_gambling: bool,
    },

    /// Set a list of custom DNS servers
    Custom {
        /// One or more IP addresses pointing to DNS resolvers
        #[arg(required(true), num_args = 1..)]
        servers: Vec<IpAddr>,
    },
}

impl Dns {
    pub async fn handle(self) -> Result<()> {
        match self {
            Dns::Get => Self::get().await,
            Dns::Set {
                cmd:
                    DnsSet::Default {
                        block_ads,
                        block_trackers,
                        block_malware,
                        block_adult_content,
                        block_gambling,
                    },
            } => {
                Self::set_default(
                    block_ads,
                    block_trackers,
                    block_malware,
                    block_adult_content,
                    block_gambling,
                )
                .await
            }
            Dns::Set {
                cmd: DnsSet::Custom { servers },
            } => Self::set_custom(servers).await,
        }
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let options = rpc.get_settings().await?.tunnel_options.dns_options;

        match options.state {
            DnsState::Default => {
                println!("Custom DNS: no");
                println!("Block ads: {}", options.default_options.block_ads);
                println!("Block trackers: {}", options.default_options.block_trackers);
                println!("Block malware: {}", options.default_options.block_malware);
                println!(
                    "Block adult content: {}",
                    options.default_options.block_adult_content
                );
                println!("Block gambling: {}", options.default_options.block_gambling);
            }
            DnsState::Custom => {
                println!("Custom DNS: yes\nServers:");
                for server in &options.custom_options.addresses {
                    println!("{server}");
                }
            }
        }

        Ok(())
    }

    async fn set_default(
        block_ads: bool,
        block_trackers: bool,
        block_malware: bool,
        block_adult_content: bool,
        block_gambling: bool,
    ) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;
        rpc.set_dns_options(DnsOptions {
            state: DnsState::Default,
            default_options: DefaultDnsOptions {
                block_ads,
                block_trackers,
                block_malware,
                block_adult_content,
                block_gambling,
            },
            ..settings.tunnel_options.dns_options
        })
        .await?;
        println!("Updated DNS settings");
        Ok(())
    }

    async fn set_custom(servers: Vec<IpAddr>) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;
        rpc.set_dns_options(DnsOptions {
            state: DnsState::Custom,
            custom_options: CustomDnsOptions { addresses: servers },
            ..settings.tunnel_options.dns_options
        })
        .await?;
        println!("Updated DNS settings");
        Ok(())
    }
}

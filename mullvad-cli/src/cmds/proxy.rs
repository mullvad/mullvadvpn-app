use anyhow::Result;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::api_access_method::AccessMethod;
use std::net::IpAddr;

use clap::Subcommand;
use talpid_types::net::openvpn::SHADOWSOCKS_CIPHERS;

#[derive(Subcommand, Debug)]
pub enum Proxy {
    /// Get current api settings
    #[clap(subcommand)]
    Api(ApiCommands),
}

impl Proxy {
    pub async fn handle(self) -> Result<()> {
        match self {
            Proxy::Api(cmd) => match cmd {
                ApiCommands::List => {
                    //println!("Listing the API access methods: ..");
                    Self::list().await?;
                }
                ApiCommands::Add(cmd) => {
                    //println!("Adding custom proxy");
                    Self::add(cmd).await?;
                }
            },
        };
        Ok(())
    }

    /// Show all API access methods.
    async fn list() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        println!("Calling [rpc::get_api_access_methods] ..");
        for api_access_method in rpc.get_api_access_methods().await? {
            println!("{:?}", api_access_method);
        }
        Ok(())
    }

    /// Add a custom API access method.
    async fn add(cmd: AddCustomCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        // TODO: Do NOT `unwrap`.
        let proxy = AccessMethod::try_from(cmd.clone()).unwrap();
        rpc.add_access_method(proxy).await?;
        Ok(())
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum ApiCommands {
    /// List the configured API proxies
    List,
    /// Add a custom API proxy
    #[clap(subcommand)]
    Add(AddCustomCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum AddCustomCommands {
    /// Configure SOCKS5 proxy
    #[clap(subcommand)]
    Socks5(Socks5AddCommands),

    /// Configure bundled Shadowsocks proxy
    Shadowsocks {
        /// The IP of the remote Shadowsocks server
        remote_ip: IpAddr,
        /// The port of the remote Shadowsocks server
        #[arg(default_value = "443")]
        remote_port: u16,
        /// Password for authentication
        #[arg(default_value = "mullvad")]
        password: String,
        /// Cipher to use
        #[arg(value_parser = SHADOWSOCKS_CIPHERS, default_value = "aes-256-gcm")]
        cipher: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum Socks5AddCommands {
    /// Configure a local SOCKS5 proxy
    Local {
        /// The port that the server on localhost is listening on
        local_port: u16,
        /// The IP of the remote peer
        remote_ip: IpAddr,
        /// The port of the remote peer
        remote_port: u16,
    },
    /// Configure a remote SOCKS5 proxy
    Remote {
        /// The IP of the remote proxy server
        remote_ip: IpAddr,
        /// The port of the remote proxy server
        remote_port: u16,

        /// Username for authentication
        #[arg(requires = "password")]
        username: Option<String>,
        /// Password for authentication
        #[arg(requires = "username")]
        password: Option<String>,
    },
}

/// Implement conversios from CLI types to Daemon types.
///
/// Since these are not supposed to be used outside of the CLI,
/// we define them in a hidden-away module.
mod conversions {
    use mullvad_types::api_access_method::AccessMethod;

    use super::{AddCustomCommands, Socks5AddCommands};

    impl TryFrom<AddCustomCommands> for AccessMethod {
        // TODO: Use some other Error type than String!
        type Error = String;

        fn try_from(value: AddCustomCommands) -> Result<Self, Self::Error> {
            Ok(match value {
                AddCustomCommands::Socks5(variant) => match variant {
                    Socks5AddCommands::Local {
                        local_port,
                        remote_ip,
                        remote_port,
                    } => {
                        println!("Adding LOCAL SOCKS5-proxy: localhost:{local_port} => {remote_ip}:{remote_port}");
                        Self {
                            name: "SOCKS5 [local]({remote_ip}:{remote_port})".to_string(),
                        }
                    }
                    Socks5AddCommands::Remote {
                        remote_ip,
                        remote_port,
                        username,
                        password,
                    } => {
                        println!("Adding REMOTE SOCKS5-proxy: {username:?}+{password:?} @ {remote_ip}:{remote_port}");
                        Self {
                            name: "SOCKS5 [remote]({remote_ip}:{remote_port})".to_string(),
                        }
                    }
                },
                AddCustomCommands::Shadowsocks {
                    remote_ip,
                    remote_port,
                    password,
                    cipher,
                } => {
                    println!(
                "Adding Shadowsocks-proxy: {password} @ {remote_ip}:{remote_port} using {cipher}"
                    );
                    Self {
                        name: "Shadowsocks ({remote_ip}:{remote_port})".to_string(),
                    }
                }
            })
        }
    }
}

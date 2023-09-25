use anyhow::{anyhow, Result};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::api_access_method::{
    daemon::{ApiAccessMethodReplace, ApiAccessMethodToggle},
    AccessMethod, ObfuscationProtocol,
};
use std::net::IpAddr;

use clap::{Args, Subcommand};
use talpid_types::net::openvpn::SHADOWSOCKS_CIPHERS;

#[derive(Subcommand, Debug, Clone)]
pub enum ApiAccess {
    /// List the configured API proxies
    List,
    /// Add a custom API proxy
    #[clap(subcommand)]
    Add(AddCustomCommands),
    /// Edit an API proxy
    Edit(EditCustomCommands),
    /// Remove an API proxy
    Remove(SelectItem),
    /// Enable an API proxy
    Enable(SelectItem),
    /// Disable an API proxy
    Disable(SelectItem),
    /// Test an API proxy
    Test(SelectItem),
    /// Force the use of a specific API proxy.
    ///
    /// Selecting "Mullvad Bridges" respects your current bridge settings.
    Use(SelectItem),
}

impl ApiAccess {
    pub async fn handle(self) -> Result<()> {
        match self {
            ApiAccess::List => {
                Self::list().await?;
            }
            ApiAccess::Add(cmd) => {
                Self::add(cmd).await?;
            }
            ApiAccess::Edit(cmd) => Self::edit(cmd).await?,
            ApiAccess::Remove(cmd) => Self::remove(cmd).await?,
            ApiAccess::Enable(cmd) => {
                let enabled = true;
                Self::toggle(cmd, enabled).await?;
            }
            ApiAccess::Disable(cmd) => {
                let enabled = false;
                Self::toggle(cmd, enabled).await?;
            }
            ApiAccess::Test(cmd) => {
                Self::test(cmd).await?;
            }
            ApiAccess::Use(cmd) => {
                Self::set(cmd).await?;
            }
        };
        Ok(())
    }

    /// Show all API access methods.
    async fn list() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        for (index, api_access_method) in rpc.get_api_access_methods().await?.iter().enumerate() {
            println!(
                "{}. {}",
                index + 1,
                pp::AccessMethodFormatter::new(api_access_method)
            );
        }
        Ok(())
    }

    /// Add a custom API access method.
    async fn add(cmd: AddCustomCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let proxy = AccessMethod::try_from(cmd.clone())?;
        rpc.add_access_method(proxy).await?;
        Ok(())
    }

    /// Remove an API access method.
    async fn remove(cmd: SelectItem) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let access_method = Self::get_access_method(&mut rpc, &cmd).await?;
        rpc.remove_access_method(access_method)
            .await
            .map_err(Into::<anyhow::Error>::into)
    }

    /// Edit the data of an API access method.
    async fn edit(cmd: EditCustomCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let access_method = Self::get_access_method(&mut rpc, &cmd.item).await?;
        let access_method = access_method
            .as_custom()
            .cloned()
            .ok_or(anyhow!("Can not edit built-in access method"))?;

        // Create a new access method combining the new params with the previous values
        let edited_access_method: AccessMethod = match access_method.access_method {
            ObfuscationProtocol::Shadowsocks(shadowsocks) => {
                let ip = cmd.params.ip.unwrap_or(shadowsocks.peer.ip()).to_string();
                let port = cmd.params.port.unwrap_or(shadowsocks.peer.port());
                let password = cmd.params.password.unwrap_or(shadowsocks.password);
                let cipher = cmd.params.cipher.unwrap_or(shadowsocks.cipher);
                let name = cmd.params.name.unwrap_or(shadowsocks.name);
                let enabled = shadowsocks.enabled;
                mullvad_types::api_access_method::Shadowsocks::from_args(
                    ip, port, cipher, password, enabled, name,
                )
                .map(|x| x.into())
            }
            ObfuscationProtocol::Socks5(socks) => match socks {
                mullvad_types::api_access_method::Socks5::Local(local) => {
                    let ip = cmd.params.ip.unwrap_or(local.peer.ip()).to_string();
                    let port = cmd.params.port.unwrap_or(local.peer.port());
                    let local_port = cmd.params.local_port.unwrap_or(local.port);
                    let name = cmd.params.name.unwrap_or(local.name);
                    let enabled = local.enabled;
                    mullvad_types::api_access_method::Socks5Local::from_args(
                        ip, port, local_port, enabled, name,
                    )
                    .map(|x| x.into())
                }
                mullvad_types::api_access_method::Socks5::Remote(remote) => {
                    let ip = cmd.params.ip.unwrap_or(remote.peer.ip()).to_string();
                    let port = cmd.params.port.unwrap_or(remote.peer.port());
                    let name = cmd.params.name.unwrap_or(remote.name);
                    let enabled = remote.enabled;
                    mullvad_types::api_access_method::Socks5Remote::from_args(
                        ip, port, enabled, name,
                    )
                    .map(|x| x.into())
                }
            },
        }
        .ok_or(anyhow!(
            "Could not edit access method {}, reverting changes.",
            cmd.item
        ))?;

        rpc.replace_access_method(ApiAccessMethodReplace {
            index: cmd.item.as_array_index()?,
            access_method: edited_access_method,
        })
        .await?;

        Ok(())
    }

    /// Toggle a custom API access method to be enabled or disabled.
    async fn toggle(item: SelectItem, enabled: bool) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let access_method = Self::get_access_method(&mut rpc, &item).await?;
        rpc.toggle_access_method(ApiAccessMethodToggle {
            access_method,
            enable: enabled,
        })
        .await?;
        Ok(())
    }

    /// Test an access method to see if it successfully reaches the Mullvad API.
    async fn test(item: SelectItem) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let access_method = Self::get_access_method(&mut rpc, &item).await?;
        rpc.set_access_method(access_method).await?;
        match rpc.test_api().await {
            Ok(_) => println!("API call succeeded!"),
            Err(_) => println!("API call failed :-("),
        }

        Ok(())
    }

    /// Force the use of a specific access method when trying to reach the
    /// Mullvad API. If this method fails, the daemon will resume the automatic
    /// roll-over behavior (which is the default).
    async fn set(item: SelectItem) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let access_method = Self::get_access_method(&mut rpc, &item).await?;
        rpc.set_access_method(access_method).await?;
        Ok(())
    }

    async fn get_access_method(
        rpc: &mut MullvadProxyClient,
        item: &SelectItem,
    ) -> Result<AccessMethod> {
        rpc.get_api_access_methods()
            .await?
            .get(item.as_array_index()?)
            .cloned()
            .ok_or(anyhow!(format!("Access method {} does not exist", item)))
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum AddCustomCommands {
    /// Configure SOCKS5 proxy
    #[clap(subcommand)]
    Socks5(Socks5AddCommands),

    /// Configure Shadowsocks proxy
    Shadowsocks {
        /// An easy to remember name for this custom proxy
        name: String,
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
        /// An easy to remember name for this custom proxy
        name: String,
        /// The port that the server on localhost is listening on
        local_port: u16,
        /// The IP of the remote peer
        remote_ip: IpAddr,
        /// The port of the remote peer
        remote_port: u16,
    },
    /// Configure a remote SOCKS5 proxy
    Remote {
        /// An easy to remember name for this custom proxy
        name: String,
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

/// A minimal wrapper type allowing the user to supply a list index to some
/// Access Method.
#[derive(Args, Debug, Clone)]
pub struct SelectItem {
    /// Which access method to pick
    index: usize,
}

impl SelectItem {
    /// Transform human-readable (1-based) index to 0-based indexing.
    pub fn as_array_index(&self) -> Result<usize> {
        self.index
            .checked_sub(1)
            .ok_or(anyhow!("Access method 0 does not exist"))
    }
}

impl std::fmt::Display for SelectItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

#[derive(Args, Debug, Clone)]
pub struct EditCustomCommands {
    /// Which API proxy to edit
    #[clap(flatten)]
    item: SelectItem,
    /// Editing parameters
    #[clap(flatten)]
    params: EditParams,
}

#[derive(Args, Debug, Clone)]
pub struct EditParams {
    /// Name of the API proxy in the Mullvad client [All]
    #[arg(long)]
    name: Option<String>,
    /// Username for authentication [Shadowsocks]
    #[arg(long)]
    username: Option<String>,
    /// Password for authentication [Shadowsocks]
    #[arg(long)]
    password: Option<String>,
    /// Cipher to use [Shadowsocks]
    #[arg(value_parser = SHADOWSOCKS_CIPHERS, long)]
    cipher: Option<String>,
    /// The IP of the remote proxy server [Socks5 (Local & Remote proxy), Shadowsocks]
    #[arg(long)]
    ip: Option<IpAddr>,
    /// The port of the remote proxy server [Socks5 (Local & Remote proxy), Shadowsocks]
    #[arg(long)]
    port: Option<u16>,
    /// The port that the server on localhost is listening on [Socks5 (Local proxy)]
    #[arg(long)]
    local_port: Option<u16>,
}

/// Implement conversions from CLI types to Daemon types.
///
/// Since these are not supposed to be used outside of the CLI,
/// we define them in a hidden-away module.
mod conversions {
    use anyhow::{anyhow, Error};
    use mullvad_types::api_access_method as daemon_types;

    use super::{AddCustomCommands, Socks5AddCommands};

    impl TryFrom<AddCustomCommands> for daemon_types::AccessMethod {
        type Error = Error;

        fn try_from(value: AddCustomCommands) -> Result<Self, Self::Error> {
            // All api proxies are automatically enabled from the start.
            let enabled = true;
            Ok(match value {
                AddCustomCommands::Socks5(variant) => match variant {
                    Socks5AddCommands::Local {
                        local_port,
                        remote_ip,
                        remote_port,
                        name,
                    } => {
                        println!("Adding LOCAL SOCKS5-proxy: localhost:{local_port} => {remote_ip}:{remote_port}");
                        let socks_proxy = daemon_types::Socks5::Local(
                            daemon_types::Socks5Local::from_args(
                                remote_ip.to_string(),
                                remote_port,
                                local_port,
                                enabled,
                                name,
                            )
                            .ok_or(anyhow!("Could not create a local Socks5 api proxy"))?,
                        );
                        socks_proxy.into()
                    }
                    Socks5AddCommands::Remote {
                        remote_ip,
                        remote_port,
                        username,
                        password,
                        name,
                    } => {
                        println!("Adding REMOTE SOCKS5-proxy: {username:?}+{password:?} @ {remote_ip}:{remote_port}");
                        let socks_proxy = daemon_types::Socks5::Remote(
                            daemon_types::Socks5Remote::from_args(
                                remote_ip.to_string(),
                                remote_port,
                                enabled,
                                name,
                            )
                            .ok_or(anyhow!("Could not create a remote Socks5 api proxy"))?,
                        );
                        socks_proxy.into()
                    }
                },
                AddCustomCommands::Shadowsocks {
                    remote_ip,
                    remote_port,
                    password,
                    cipher,
                    name,
                } => {
                    println!(
                "Adding Shadowsocks-proxy: {password} @ {remote_ip}:{remote_port} using {cipher}"
                    );
                    let shadowsocks_proxy = daemon_types::Shadowsocks::from_args(
                        remote_ip.to_string(),
                        remote_port,
                        cipher,
                        password,
                        enabled,
                        name,
                    )
                    .ok_or(anyhow!("Could not create a Shadowsocks api proxy"))?;
                    shadowsocks_proxy.into()
                }
            })
        }
    }
}

/// Pretty printing of [`AccessMethod`]s
mod pp {
    use mullvad_types::api_access_method::{
        AccessMethod, BuiltInAccessMethod, ObfuscationProtocol, Socks5,
    };

    pub struct AccessMethodFormatter<'a> {
        access_method: &'a AccessMethod,
    }

    impl<'a> AccessMethodFormatter<'a> {
        pub fn new(access_method: &'a AccessMethod) -> AccessMethodFormatter<'a> {
            AccessMethodFormatter { access_method }
        }
    }

    impl<'a> std::fmt::Display for AccessMethodFormatter<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use crate::print_option;

            let write_status = |f: &mut std::fmt::Formatter<'_>, enabled: bool| {
                if enabled {
                    write!(f, " *")
                } else {
                    write!(f, "")
                }
            };

            match self.access_method {
                AccessMethod::BuiltIn(method) => match method {
                    BuiltInAccessMethod::Direct(enabled) => {
                        write!(f, "Direct")?;
                        write_status(f, *enabled)
                    }
                    BuiltInAccessMethod::Bridge(enabled) => {
                        write!(f, "Mullvad Bridges")?;
                        write_status(f, *enabled)
                    }
                },
                AccessMethod::Custom(method) => match &method.access_method {
                    ObfuscationProtocol::Shadowsocks(shadowsocks) => {
                        write!(f, "{}", shadowsocks.name)?;
                        write_status(f, shadowsocks.enabled)?;
                        writeln!(f)?;
                        print_option!("Protocol", format!("Shadowsocks [{}]", shadowsocks.cipher));
                        print_option!("Peer", shadowsocks.peer);
                        print_option!("Password", shadowsocks.password);
                        Ok(())
                    }
                    ObfuscationProtocol::Socks5(socks) => match socks {
                        Socks5::Remote(remote) => {
                            write!(f, "{}", remote.name)?;
                            write_status(f, remote.enabled)?;
                            writeln!(f)?;
                            print_option!("Protocol", "Socks5");
                            print_option!("Peer", remote.peer);
                            Ok(())
                        }
                        Socks5::Local(local) => {
                            write!(f, "{}", local.name)?;
                            write_status(f, local.enabled)?;
                            writeln!(f)?;
                            print_option!("Protocol", "Socks5 (local)");
                            print_option!("Peer", local.peer);
                            print_option!("Local port", local.port);
                            Ok(())
                        }
                    },
                },
            }
        }
    }
}

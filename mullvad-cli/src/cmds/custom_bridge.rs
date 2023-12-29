use anyhow::{bail, Result};
use clap::Subcommand;

use super::proxies::{ProxyEditParams, ShadowsocksAdd, Socks5LocalAdd, Socks5RemoteAdd};
use mullvad_management_interface::MullvadProxyClient;
use talpid_types::net::proxy::{
    CustomBridgeSettings, CustomProxy, Shadowsocks, Socks5, Socks5Local, Socks5Remote,
};

#[derive(Subcommand, Debug, Clone)]
pub enum CustomCommands {
    /// Remove the saved custom bridge configuration, does not disconnect if the bridge is
    /// currently in use.
    Remove,
    /// Connects to the currently saved custom bridge configuration.
    Set,
    /// Add a new custom bridge configuration.
    #[clap(subcommand)]
    Add(AddCustomCommands),
    /// Edit an already existing custom bridge configuration.
    Edit(ProxyEditParams),
}

#[derive(Subcommand, Debug, Clone)]
pub enum AddCustomCommands {
    #[clap(subcommand)]
    Socks5(AddSocks5Commands),
    /// Configure bundled Shadowsocks proxy
    Shadowsocks {
        #[clap(flatten)]
        add: ShadowsocksAdd,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum AddSocks5Commands {
    /// Configure a local SOCKS5 proxy
    #[clap(
        about = "Registers a local SOCKS5 proxy. Will allow all local programs to leak traffic *only* to the remote endpoint."
    )]
    Local {
        #[clap(flatten)]
        add: Socks5LocalAdd,
    },

    /// Configure a remote SOCKS5 proxy
    Remote {
        #[clap(flatten)]
        add: Socks5RemoteAdd,
    },
}

impl CustomCommands {
    pub async fn handle(self) -> Result<()> {
        match self {
            CustomCommands::Set => Self::custom_bridge_set().await,
            CustomCommands::Remove => Self::custom_bridge_remove().await,
            CustomCommands::Edit(edit) => Self::custom_bridge_edit(edit).await,
            CustomCommands::Add(add_custom_commands) => {
                Self::custom_bridge_add(add_custom_commands).await
            }
        }
    }

    async fn custom_bridge_edit(edit: ProxyEditParams) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;
        let mut custom_bridge = settings.custom_bridge;
        let Some(old_custom_bridge) = custom_bridge.custom_bridge else {
            bail!("Can not edit as there is no currently saved custom bridge");
        };

        custom_bridge.custom_bridge = Some(match old_custom_bridge {
            CustomProxy::Shadowsocks(ss) => CustomProxy::Shadowsocks(edit.merge_shadowsocks(ss)),
            CustomProxy::Socks5(socks) => match socks {
                Socks5::Local(local) => {
                    CustomProxy::Socks5(Socks5::Local(edit.merge_socks_local(local)))
                }
                Socks5::Remote(remote) => {
                    CustomProxy::Socks5(Socks5::Remote(edit.merge_socks_remote(remote)))
                }
            },
        });

        rpc.update_custom_bridge(custom_bridge)
            .await
            .map_err(anyhow::Error::from)
    }

    async fn custom_bridge_remove() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let custom_bridge = CustomBridgeSettings {
            custom_bridge: None,
        };
        rpc.update_custom_bridge(custom_bridge)
            .await
            .map_err(anyhow::Error::from)
    }

    async fn custom_bridge_set() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_custom_bridge().await.map_err(anyhow::Error::from)
    }

    async fn custom_bridge_add(add_commands: AddCustomCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let custom_bridge = CustomBridgeSettings {
            custom_bridge: Some(match add_commands {
                AddCustomCommands::Socks5(AddSocks5Commands::Local { add }) => {
                    CustomProxy::Socks5(Socks5::Local(Socks5Local::from(add)))
                }
                AddCustomCommands::Socks5(AddSocks5Commands::Remote { add }) => {
                    CustomProxy::Socks5(Socks5::Remote(Socks5Remote::from(add)))
                }
                AddCustomCommands::Shadowsocks { add } => {
                    CustomProxy::Shadowsocks(Shadowsocks::from(add))
                }
            }),
        };
        rpc.update_custom_bridge(custom_bridge)
            .await
            .map_err(anyhow::Error::from)
    }
}

use anyhow::Result;
use clap::Subcommand;

use super::proxies::{EditParams, ShadowsocksAdd, Socks5LocalAdd, Socks5RemoteAdd};
use mullvad_management_interface::MullvadProxyClient;
use talpid_types::net::proxy::{
    CustomProxy, CustomProxySettings, Shadowsocks, Socks5, Socks5Local, Socks5Remote,
};

#[derive(Subcommand, Debug, Clone)]
pub enum CustomCommands {
    /// TODO
    Get,
    /// TODO
    Remove,
    /// TODO
    Set,
    /// TODO
    #[clap(subcommand)]
    Add(AddCustomCommands),
    /// TODO
    Edit(EditParams),
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
    // TODO: Update with new security requirements
    /// Configure a local SOCKS5 proxy
    // TODO: Fix comment
    #[cfg_attr(
        target_os = "linux",
        clap(
            about = "Registers a local SOCKS5 proxy. The server must be excluded using \
        'mullvad-exclude', or `SO_MARK` must be set to '0x6d6f6c65', in order \
        to bypass firewall restrictions"
        )
    )]
    // TODO: Fix comment
    #[cfg_attr(
        target_os = "windows",
        clap(
            about = "Registers a local SOCKS5 proxy. The server must be excluded using \
        split tunneling in order to bypass firewall restrictions"
        )
    )]
    // TODO: Fix comment
    #[cfg_attr(
        target_os = "macos",
        clap(
            about = "Registers a local SOCKS5 proxy. The server must run as root to bypass \
        firewall restrictions"
        )
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
            CustomCommands::Get => Self::custom_proxy_get().await,
            CustomCommands::Set => Self::custom_proxy_set().await,
            CustomCommands::Remove => Self::custom_bridge_remove().await,
            CustomCommands::Edit(edit) => Self::custom_proxy_edit(edit).await,
            CustomCommands::Add(add_custom_commands) => {
                Self::custom_proxy_add(add_custom_commands).await
            }
        }
    }

    async fn custom_proxy_edit(edit: EditParams) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let mut custom_bridge = rpc.get_custom_bridge().await?;
        custom_bridge.custom_proxy =
            custom_bridge
                .custom_proxy
                .map(|custom_bridge| match custom_bridge {
                    CustomProxy::Shadowsocks(ss) => {
                        CustomProxy::Shadowsocks(edit.merge_shadowsocks(ss))
                    }
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
        rpc.remove_custom_bridge()
            .await
            .map_err(anyhow::Error::from)
    }

    async fn custom_proxy_set() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_custom_bridge().await.map_err(anyhow::Error::from)
    }

    async fn custom_proxy_get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let current_proxy_settings = rpc.get_custom_bridge().await?;
        println!("{:?}", current_proxy_settings);
        Ok(())
    }

    async fn custom_proxy_add(add_commands: AddCustomCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let custom_bridge = CustomProxySettings {
            active: true,
            custom_proxy: Some(match add_commands {
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

//async fn custom_bridge_add(subcmd: AddCustomCommands) -> Result<()> {
//    match subcmd {
//        AddCustomCommands::Socks5(AddSocks5Commands::Local {
//            add,
//        }) => {
//            let local_proxy = openvpn::LocalProxySettings {
//                port: add.local_port,
//                peer: SocketAddr::new(add.remote_ip, add.remote_port),
//            };
//            let packed_proxy = openvpn::ProxySettings::Local(local_proxy);
//            if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
//                panic!("{}", error);
//            }
//
//            let mut rpc = MullvadProxyClient::new().await?;
//            rpc.set_bridge_settings(BridgeSettings::Custom(packed_proxy))
//                .await?;
//        }
//        AddCustomCommands::Socks5(AddSocks5Commands::Remote {
//            add,
//        }) => {
//            let auth = match add.authentication {
//                Some(auth) => Some(openvpn::ProxyAuth { username: auth.username, password: auth.password }),
//                _ => None,
//            };
//            let proxy = openvpn::RemoteProxySettings {
//                address: SocketAddr::new(add.remote_ip, add.remote_port),
//                auth,
//            };
//            let packed_proxy = openvpn::ProxySettings::Remote(proxy);
//            if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
//                panic!("{}", error);
//            }
//
//            let mut rpc = MullvadProxyClient::new().await?;
//            rpc.set_bridge_settings(BridgeSettings::Custom(packed_proxy))
//                .await?;
//        }
//        AddCustomCommands::Shadowsocks {
//            add
//        } => {
//            let proxy = openvpn::ShadowsocksProxySettings {
//                peer: SocketAddr::new(add.remote_ip, add.remote_port),
//                password: add.password,
//                cipher: add.cipher,
//                #[cfg(target_os = "linux")]
//                fwmark: None,
//            };
//            let packed_proxy = openvpn::ProxySettings::Shadowsocks(proxy);
//            if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
//                panic!("{}", error);
//            }
//
//            let mut rpc = MullvadProxyClient::new().await?;
//            rpc.set_bridge_settings(BridgeSettings::Custom(packed_proxy))
//                .await?;
//        }
//    }
//
//    println!("Updated bridge settings");
//    Ok(())
//}

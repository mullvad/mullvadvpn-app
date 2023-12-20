use anyhow::Result;
use clap::{Subcommand, Args};
use std::net::SocketAddr;

use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::relay_constraints::BridgeSettings;
use talpid_types::net::openvpn;

use super::proxies::{ShadowsocksAdd, Socks5LocalAdd, Socks5RemoteAdd, SelectItem, EditParams};

#[derive(Subcommand, Debug, Clone)]
pub enum CustomCommands {
    /// TODO
    Get,
    /// TODO
    Remove,
    /// TODO
    Select,
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

pub async fn custom_bridge(subcmd: CustomCommands) -> Result<()> {
    match subcmd {
        CustomCommands::Get => custom_proxy_get(),
        CustomCommands::Select => custom_proxy_select(),
        CustomCommands::Remove => custom_bridge_remove(),
        CustomCommands::Edit(edit) => custom_proxy_edit(edit),
        CustomCommands::Add(add_custom_commands) => custom_proxy_add(add_custom_commands).await,
    }
}

async fn custom_proxy_edit(edit: EditCustomCommands) -> Result<()> {
    todo!()
}

async fn custom_bridge_remove() -> Result<()> {
    todo!()
}

async fn custom_proxy_select() -> Result<()> {
    todo!()
}

async fn custom_proxy_get() -> Result<()> {
    todo!()
}

async fn custom_proxy_add(add_commands: AddCustomCommands) -> Result<()> {
    todo!()
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

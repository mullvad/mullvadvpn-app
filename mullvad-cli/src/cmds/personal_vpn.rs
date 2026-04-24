#![cfg(feature = "personal-vpn")]

use anyhow::{Context, Result};
use clap::Subcommand;
use ipnetwork::IpNetwork;
use mullvad_management_interface::MullvadProxyClient;
use std::{
    fs,
    io::{self, Read},
    net::{IpAddr, SocketAddr},
    str::FromStr,
};
use talpid_types::net::wireguard;
use talpid_types::net::wireguard::{
    PersonalVpnConfig, PersonalVpnPeerConfig, PersonalVpnTunnelConfig,
};

use super::BooleanOption;

#[derive(Subcommand, Debug)]
pub enum PersonalVpn {
    /// Show current personal VPN configuration and enabled state
    Get,

    /// Set the personal WireGuard VPN configuration (requires all fields)
    Set {
        /// Base64-encoded WireGuard private key for the tunnel interface
        #[arg(long, value_parser = wireguard::PrivateKey::from_base64)]
        private_key: wireguard::PrivateKey,

        /// IP address for the tunnel interface
        #[arg(long)]
        tunnel_ip: IpAddr,

        /// Base64-encoded WireGuard public key of the VPN peer
        #[arg(long, value_parser = wireguard::PublicKey::from_base64)]
        peer_pubkey: wireguard::PublicKey,

        /// IP network to route through the VPN peer, in CIDR notation (e.g. 0.0.0.0/0)
        #[arg(long)]
        allowed_ip: Vec<IpNetwork>,

        /// Endpoint of the VPN peer (e.g. 1.2.3.4:51820)
        #[arg(long)]
        endpoint: SocketAddr,
    },

    /// Enable or disable the personal VPN
    Enable { state: BooleanOption },

    /// Remove the personal VPN configuration
    Unset,

    /// Import a WireGuard config file.
    /// If PATH is "-", read from standard input.
    Import {
        /// Path to the config file, or "-" for stdin
        path: String,
    },
}

impl PersonalVpn {
    pub async fn handle(self) -> Result<()> {
        match self {
            PersonalVpn::Get => Self::get().await,
            PersonalVpn::Set {
                private_key,
                tunnel_ip,
                peer_pubkey,
                allowed_ip,
                endpoint,
            } => Self::set(private_key, tunnel_ip, peer_pubkey, allowed_ip, endpoint).await,
            PersonalVpn::Enable { state } => Self::enable(state).await,
            PersonalVpn::Unset => Self::unset().await,
            PersonalVpn::Import { path } => Self::import(path).await,
        }
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;
        let enabled = BooleanOption::from(settings.personal_vpn_enabled);
        println!("Personal VPN: {enabled}");
        match settings.personal_vpn_config {
            None => println!("Configuration: none"),
            Some(config) => {
                // TODO: Do not print?
                println!(
                    "Tunnel private key: {}",
                    config.tunnel.private_key.to_base64()
                );
                println!("Tunnel IP:          {}", config.tunnel.ip);
                println!("Peer public key:    {}", config.peer.public_key.to_base64());
                for allowed_ip in &config.peer.allowed_ip {
                    println!("Allowed IP:         {allowed_ip}");
                }
                println!("Endpoint:           {}", config.peer.endpoint);
            }
        }
        Ok(())
    }

    async fn set(
        private_key: wireguard::PrivateKey,
        tunnel_ip: IpAddr,
        peer_pubkey: wireguard::PublicKey,
        allowed_ip: Vec<IpNetwork>,
        endpoint: SocketAddr,
    ) -> Result<()> {
        let config = PersonalVpnConfig {
            tunnel: PersonalVpnTunnelConfig {
                private_key,
                ip: tunnel_ip,
            },
            peer: PersonalVpnPeerConfig {
                public_key: peer_pubkey,
                allowed_ip,
                endpoint,
            },
        };
        let mut rpc = MullvadProxyClient::new().await?;
        let error = rpc.set_personal_vpn_config(Some(config)).await?;
        if !error.is_empty() {
            anyhow::bail!("Daemon returned error: {error}");
        }
        println!("Personal VPN configuration updated");
        Ok(())
    }

    async fn enable(state: BooleanOption) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_personal_vpn_config_status(*state).await?;
        println!("Personal VPN: {state}");
        Ok(())
    }

    async fn unset() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let error = rpc.set_personal_vpn_config(None).await?;
        if !error.is_empty() {
            anyhow::bail!("Daemon returned error: {error}");
        }
        println!("Personal VPN configuration removed");
        Ok(())
    }

    /// Blocking: reads the entire file or stdin before returning.
    async fn import(path: String) -> Result<()> {
        let config_str = tokio::task::spawn_blocking(move || -> Result<String> {
            if path == "-" {
                let mut buf = String::new();
                io::stdin()
                    .read_to_string(&mut buf)
                    .context("Failed to read from stdin")?;
                Ok(buf)
            } else {
                fs::read_to_string(&path).with_context(|| format!("Failed to read {path}"))
            }
        })
        .await??;

        let config =
            PersonalVpnConfig::from_str(&config_str).context("Failed to parse WireGuard config")?;

        let mut rpc = MullvadProxyClient::new().await?;
        let error = rpc.set_personal_vpn_config(Some(config)).await?;
        if !error.is_empty() {
            anyhow::bail!("Daemon returned error: {error}");
        }
        println!("Personal VPN configuration imported");
        Ok(())
    }
}

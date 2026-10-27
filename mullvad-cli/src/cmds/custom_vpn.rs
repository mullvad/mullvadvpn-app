#![cfg(feature = "custom-vpn")]

use anyhow::{Context, Result};
use clap::Subcommand;
use ipnetwork::IpNetwork;
use mullvad_management_interface::MullvadProxyClient;
use std::{
    fs,
    io::{self, Read},
    net::IpAddr,
};
use talpid_types::net::wireguard;
use talpid_types::net::wireguard::{CustomVpnConfig, CustomVpnPeerConfig, CustomVpnTunnelConfig};

use super::BooleanOption;

#[derive(Subcommand, Debug)]
pub enum CustomVpn {
    /// Show current custom VPN configuration and enabled state
    Get,

    /// Set the custom WireGuard VPN configuration (requires all fields)
    Set {
        /// Base64-encoded WireGuard private key for the tunnel interface
        #[arg(long, value_parser = wireguard::PrivateKey::from_base64)]
        private_key: wireguard::PrivateKey,

        /// IP address for the tunnel interface. Repeat to configure multiple
        /// (typically one IPv4 and/or one IPv6), e.g.
        /// `--tunnel-ip 10.0.0.2 --tunnel-ip fd00::2`.
        #[arg(long = "tunnel-ip", required = true, num_args = 1..)]
        tunnel_ip: Vec<IpAddr>,

        /// Base64-encoded WireGuard public key of the VPN peer
        #[arg(long, value_parser = wireguard::PublicKey::from_base64)]
        peer_pubkey: wireguard::PublicKey,

        /// IP network to route through the VPN peer, in CIDR notation (e.g. 0.0.0.0/0)
        #[arg(long)]
        allowed_ip: Vec<IpNetwork>,

        /// Endpoint of the VPN peer as `<host>:<port>` (host may be an IP or
        /// DNS name, e.g. `1.2.3.4:51820` or `vpn.example.com:51820`)
        #[arg(long)]
        endpoint: String,
    },

    /// Enable or disable the custom VPN
    Enable { state: BooleanOption },

    /// Remove the custom VPN configuration
    Unset,

    /// Import a WireGuard config file.
    /// If PATH is "-", read from standard input.
    Import {
        /// Path to the config file, or "-" for stdin
        path: String,
    },
}

impl CustomVpn {
    pub async fn handle(self) -> Result<()> {
        match self {
            CustomVpn::Get => Self::get().await,
            CustomVpn::Set {
                private_key,
                tunnel_ip,
                peer_pubkey,
                allowed_ip,
                endpoint,
            } => Self::set(private_key, tunnel_ip, peer_pubkey, allowed_ip, endpoint).await,
            CustomVpn::Enable { state } => Self::enable(state).await,
            CustomVpn::Unset => Self::unset().await,
            CustomVpn::Import { path } => Self::import(path).await,
        }
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;
        let config = settings.custom_vpn.configs.into_iter().next();
        let enabled = BooleanOption::from(config.as_ref().is_some_and(|c| c.enabled));
        println!("Personal VPN: {enabled}");
        match config {
            None => println!("Configuration: none"),
            Some(config) => {
                if let Some(name) = &config.name {
                    println!("Name:               {name}");
                }
                // TODO: Do not print?
                println!(
                    "Tunnel private key: {}",
                    config.tunnel.private_key.to_base64()
                );
                let ips = config
                    .tunnel
                    .ips
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("Tunnel IPs:         {ips}");
                for peer in &config.peers {
                    println!("Peer public key:    {}", peer.public_key.to_base64());
                    for allowed_ip in &peer.allowed_ip {
                        println!("Allowed IP:         {allowed_ip}");
                    }
                    println!("Endpoint:           {}", peer.endpoint);
                }
            }
        }
        Ok(())
    }

    async fn set(
        private_key: wireguard::PrivateKey,
        tunnel_ip: Vec<IpAddr>,
        peer_pubkey: wireguard::PublicKey,
        allowed_ip: Vec<IpNetwork>,
        endpoint: String,
    ) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;
        // Keep the current enabled state
        let enabled = settings.custom_vpn.active().is_some();
        let config = CustomVpnConfig {
            enabled,
            name: None,
            tunnel: CustomVpnTunnelConfig {
                private_key,
                ips: tunnel_ip,
            },
            peers: vec1::vec1![CustomVpnPeerConfig {
                public_key: peer_pubkey,
                allowed_ip,
                endpoint: endpoint.parse()?,
            }],
        };
        let error = rpc.set_custom_vpn_config(config).await?;
        if !error.is_empty() {
            anyhow::bail!("Daemon returned error: {error}");
        }
        println!("Personal VPN configuration updated");
        Ok(())
    }

    async fn enable(state: BooleanOption) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;
        let Some(mut config) = settings.custom_vpn.configs.into_iter().next() else {
            anyhow::bail!("No personal VPN configuration");
        };
        config.enabled = *state;
        let error = rpc.set_custom_vpn_config(config).await?;
        if !error.is_empty() {
            anyhow::bail!("Daemon returned error: {error}");
        }
        println!("Personal VPN: {state}");
        Ok(())
    }

    async fn unset() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let error = rpc.clear_custom_vpn_config().await?;
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

        let mut rpc = MullvadProxyClient::new().await?;
        let error = rpc.import_custom_vpn_config(config_str).await?;
        if !error.is_empty() {
            anyhow::bail!("Daemon returned error: {error}");
        }
        println!("Personal VPN configuration imported");
        Ok(())
    }
}

use anyhow::Result;
use clap::Subcommand;
use ipnetwork::IpNetwork;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::settings::{CustomVpnConfig, CustomVpnPeerConfig, CustomVpnTunnelConfig};
use std::net::{IpAddr, SocketAddr};
use talpid_types::net::wireguard;

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
        allowed_ip: IpNetwork,

        /// Endpoint of the VPN peer (e.g. 1.2.3.4:51820)
        #[arg(long)]
        endpoint: SocketAddr,
    },

    /// Enable or disable the personal VPN
    Enable { state: BooleanOption },

    /// Remove the personal VPN configuration
    Unset,
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
        }
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;
        let enabled = BooleanOption::from(settings.custom_vpn_enabled);
        println!("Personal VPN: {enabled}");
        match settings.custom_vpn_config {
            None => println!("Configuration: none"),
            Some(config) => {
                // TODO: Do not print?
                println!(
                    "Tunnel private key: {}",
                    config.tunnel.private_key.to_base64()
                );
                println!("Tunnel IP:          {}", config.tunnel.ip);
                println!("Peer public key:    {}", config.peer.public_key.to_base64());
                println!("Allowed IP:         {}", config.peer.allowed_ip);
                println!("Endpoint:           {}", config.peer.endpoint);
            }
        }
        Ok(())
    }

    async fn set(
        private_key: wireguard::PrivateKey,
        tunnel_ip: IpAddr,
        peer_pubkey: wireguard::PublicKey,
        allowed_ip: IpNetwork,
        endpoint: SocketAddr,
    ) -> Result<()> {
        let config = CustomVpnConfig {
            tunnel: CustomVpnTunnelConfig {
                private_key,
                ip: tunnel_ip,
            },
            peer: CustomVpnPeerConfig {
                public_key: peer_pubkey,
                allowed_ip,
                endpoint,
            },
        };
        let mut rpc = MullvadProxyClient::new().await?;
        let error = rpc.set_custom_vpn_config(Some(config)).await?;
        if !error.is_empty() {
            anyhow::bail!("Daemon returned error: {error}");
        }
        println!("Personal VPN configuration updated");
        Ok(())
    }

    async fn enable(state: BooleanOption) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_custom_vpn_config_status(*state).await?;
        println!("Personal VPN: {state}");
        Ok(())
    }

    async fn unset() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let error = rpc.set_custom_vpn_config(None).await?;
        if !error.is_empty() {
            anyhow::bail!("Daemon returned error: {error}");
        }
        println!("Personal VPN configuration removed");
        Ok(())
    }
}

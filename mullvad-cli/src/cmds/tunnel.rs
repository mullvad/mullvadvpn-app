use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    constraints::Constraint,
    relay_constraints::{AllowedIps, RelaySettings, WireguardConstraints},
    wireguard::{QuantumResistantState, RotationInterval},
};

use super::BooleanOption;
use crate::print_option;

#[derive(Subcommand, Debug)]
pub enum Tunnel {
    /// Show current tunnel options
    Get,

    /// Set tunnel options
    #[clap(subcommand)]
    Set(TunnelOptions),
}

#[derive(Subcommand, Debug, Clone)]
pub enum TunnelOptions {
    /// Configure the tunnel MTU, or 'any'
    Mtu { mtu: Constraint<u16> },

    /// Configure quantum-resistant key exchange
    QuantumResistant { state: QuantumResistantState },

    /// Configure whether to enable DAITA
    Daita { state: BooleanOption },

    /// Configure whether to enable DAITA direct only
    DaitaDirectOnly { state: BooleanOption },

    /// Specify custom allowed IPs for WireGuard tunnels. Use comma-separated values of IPs and IP ranges in CIDR notation.
    /// A empty string resets to the default value, where all traffic is allowed, i.e. (0.0.0.0/0,::/0).
    /// For CIDR ranges, host bits must be zero (e.g., "10.0.0.0/24" is valid, "10.0.0.1/24" is not).
    ///
    /// Example: "10.0.0.0/24,192.168.1.1,fd00::/8"
    ///
    /// WARNING: Setting this value incorrectly may cause internet access to be blocked or the app to not work properly.
    AllowedIps { allowed_ips: String },

    /// The key rotation interval. Number of hours, or 'any'
    RotationInterval {
        interval: Constraint<RotationInterval>,
    },

    /// Replace the WireGuard key with a new one
    RotateKey,

    /// Enable or disable IPv6 in the tunnel
    #[clap(arg_required_else_help = true)]
    Ipv6 { state: BooleanOption },
}

impl Tunnel {
    pub async fn handle(self) -> Result<()> {
        match self {
            Tunnel::Get => Self::get().await,
            Tunnel::Set(options) => Self::set(options).await,
        }
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let tunnel_options = rpc.get_settings().await?.tunnel_options;

        println!("WireGuard options");

        print_option!(
            "MTU",
            tunnel_options
                .wireguard
                .mtu
                .map(|val| val.to_string())
                .unwrap_or("unset".to_string()),
        );
        print_option!(
            "Quantum resistance",
            tunnel_options.wireguard.quantum_resistant,
        );

        print_option!("DAITA", tunnel_options.wireguard.daita.enabled);

        let key = rpc.get_wireguard_key().await?;
        print_option!("Public key", key.key,);
        print_option!(format_args!(
            "Created {}",
            key.created.with_timezone(&chrono::Local)
        ),);
        print_option!(
            "Rotation interval",
            match tunnel_options.wireguard.rotation_interval {
                Some(interval) => interval.to_string(),
                None => "unset".to_string(),
            },
        );

        // Get the WireGuard allowed IPs
        let wireguard_constraints = match rpc.get_settings().await?.relay_settings {
            RelaySettings::Normal(settings) => settings.wireguard_constraints,
            RelaySettings::CustomTunnelEndpoint(_) => WireguardConstraints::default(),
        };

        print_option!(
            "Allowed IPs",
            match wireguard_constraints.allowed_ips {
                mullvad_types::constraints::Constraint::Any => "all traffic (default)".to_string(),
                mullvad_types::constraints::Constraint::Only(ips) => {
                    ips.to_string()
                }
            },
        );

        println!("Generic options");

        print_option!(
            "IPv6",
            if tunnel_options.generic.enable_ipv6 {
                "on"
            } else {
                "off"
            }
        );

        Ok(())
    }

    async fn set(options: TunnelOptions) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;

        match options {
            TunnelOptions::Mtu { mtu } => {
                rpc.set_wireguard_mtu(mtu.option()).await?;
                println!("MTU parameter has been updated");
            }
            TunnelOptions::QuantumResistant { state } => {
                rpc.set_quantum_resistant_tunnel(state).await?;
                println!("Quantum resistant setting has been updated");
            }
            TunnelOptions::Daita { state } => {
                rpc.set_enable_daita(*state).await?;
                println!("DAITA setting has been updated");
            }
            TunnelOptions::DaitaDirectOnly { state } => {
                rpc.set_daita_direct_only(*state).await?;
                println!("Direct only setting has been updated");
            }
            TunnelOptions::AllowedIps { allowed_ips } => {
                let ips = AllowedIps::parse(allowed_ips.split(','))?;
                rpc.set_wireguard_allowed_ips(ips).await?;
                println!("WireGuard allowed IPs have been updated");
            }
            TunnelOptions::RotationInterval { interval } => match interval {
                Constraint::Only(interval) => {
                    rpc.set_wireguard_rotation_interval(interval).await?;
                    println!("Set key rotation interval to {interval}");
                }
                Constraint::Any => {
                    rpc.reset_wireguard_rotation_interval().await?;
                    println!(
                        "Reset key rotation interval to {}",
                        RotationInterval::default()
                    );
                }
            },
            TunnelOptions::RotateKey => {
                rpc.rotate_wireguard_key().await?;
                println!("Rotated WireGuard key");
            }
            TunnelOptions::Ipv6 { state } => {
                rpc.set_enable_ipv6(*state).await?;
                println!("IPv6: {state}");
            }
        }

        Ok(())
    }
}

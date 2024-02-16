use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    constraints::Constraint,
    wireguard::{QuantumResistantState, RotationInterval, DEFAULT_ROTATION_INTERVAL},
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
    /// Manage options for OpenVPN tunnels
    #[clap(arg_required_else_help = true)]
    Openvpn {
        /// Configure the mssfix parameter, or 'any'
        #[arg(long, short = 'm')]
        mssfix: Option<Constraint<u16>>,
    },

    /// Manage options for WireGuard tunnels
    #[clap(arg_required_else_help = true)]
    Wireguard {
        /// Configure the tunnel MTU, or 'any'
        #[arg(long, short = 'm')]
        mtu: Option<Constraint<u16>>,
        /// Configure quantum-resistant key exchange
        #[arg(long)]
        quantum_resistant: Option<QuantumResistantState>,
        /// The key rotation interval. Number of hours, or 'any'
        #[arg(long)]
        rotation_interval: Option<Constraint<RotationInterval>>,
        /// Rotate WireGuard key
        #[clap(subcommand)]
        rotate_key: Option<RotateKey>,
    },

    /// Enable or disable IPv6 in the tunnel
    #[clap(arg_required_else_help = true)]
    Ipv6 { state: BooleanOption },
}

#[derive(Subcommand, Debug, Clone)]
pub enum RotateKey {
    /// Replace the WireGuard key with a new one
    RotateKey,
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

        println!("OpenVPN options");

        print_option!(
            "mssfix",
            tunnel_options
                .openvpn
                .mssfix
                .map(|val| val.to_string())
                .unwrap_or("unset".to_string()),
        );

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
        match options {
            TunnelOptions::Openvpn { mssfix } => Self::handle_openvpn(mssfix).await,
            TunnelOptions::Wireguard {
                mtu,
                quantum_resistant,
                rotation_interval,
                rotate_key,
            } => {
                Self::handle_wireguard(mtu, quantum_resistant, rotation_interval, rotate_key).await
            }
            TunnelOptions::Ipv6 { state } => Self::handle_ipv6(state).await,
        }
    }

    async fn handle_ipv6(state: BooleanOption) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_enable_ipv6(*state).await?;
        println!("IPv6: {state}");
        Ok(())
    }

    async fn handle_openvpn(mssfix: Option<Constraint<u16>>) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;

        if let Some(mssfix) = mssfix {
            rpc.set_openvpn_mssfix(mssfix.option()).await?;
            println!("mssfix parameter has been updated");
        }

        Ok(())
    }

    async fn handle_wireguard(
        mtu: Option<Constraint<u16>>,
        quantum_resistant: Option<QuantumResistantState>,
        rotation_interval: Option<Constraint<RotationInterval>>,
        rotate_key: Option<RotateKey>,
    ) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;

        if let Some(mtu) = mtu {
            rpc.set_wireguard_mtu(mtu.option()).await?;
            println!("MTU parameter has been updated");
        }

        if let Some(quantum_resistant) = quantum_resistant {
            rpc.set_quantum_resistant_tunnel(quantum_resistant).await?;
            println!("Quantum resistant setting has been updated");
        }

        if let Some(interval) = rotation_interval {
            match interval {
                Constraint::Only(interval) => {
                    rpc.set_wireguard_rotation_interval(interval).await?;
                    println!("Set key rotation interval to {}", interval);
                }
                Constraint::Any => {
                    rpc.reset_wireguard_rotation_interval().await?;
                    println!(
                        "Reset key rotation interval to {}",
                        RotationInterval::new(DEFAULT_ROTATION_INTERVAL).unwrap()
                    );
                }
            }
        }

        if matches!(rotate_key, Some(RotateKey::RotateKey)) {
            rpc.rotate_wireguard_key().await?;
            println!("Rotated WireGuard key");
        }

        Ok(())
    }
}

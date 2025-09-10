//! Boringtun + Masque = <3

use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use boringtun::device::{DeviceConfig, DeviceHandle};
use boringtun::tun::tun_async_device::AsyncDevice;
use tunnel_obfuscation::quic::Quic;
use tunnel_obfuscation::{PacketChannelSimple, SimpleChannelRx, SimpleChannelTx};

pub const TUN_NAME: &str = "mullvadtun";

/// Communication between Boringtun <-> Masque client.
/// This
type DeviceTransports = (PacketChannelSimple, Arc<AsyncDevice>, Arc<AsyncDevice>);

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    // god, forgive me

    let (obfs_tx, obfs_rx, obfuscator_channels) = {
        let capacity = 100;
        let source_v4 = SocketAddr::from((Ipv4Addr::new(10, 0, 0, 1), 0)); // TODO: Is this correct? No.

        tunnel_obfuscation::new_packet_channels(capacity, source_v4)
    };

    log::info!("Boringtun + Masque = <3");
    let quic = create_quic_obfuscator((obfs_tx, obfs_rx)).await;
    log::info!("Masque proxy client started successfully");
    let boringtun: DeviceHandle<DeviceTransports> = create_boringtun(obfuscator_channels).await;
    log::info!("BoringTun started successfully");

    Ok(())
}

pub async fn create_quic_obfuscator(channels: (SimpleChannelTx, SimpleChannelRx)) -> Quic {
    use tunnel_obfuscation::quic;
    let settings =
        tunnel_obfuscation::Settings::Quic(quic::Settings::new(todo!(), todo!(), todo!(), todo!()));

    let obfuscator = tunnel_obfuscation::create_quic_obfuscator(&settings, channels)
        .await
        .unwrap();
    obfuscator.unwrap()
}

pub async fn create_boringtun(
    obfuscator_socket_isch: PacketChannelSimple,
) -> DeviceHandle<DeviceTransports> {
    let api = boringtun::device::api::ApiServer::default_unix_socket(TUN_NAME).unwrap();
    let config = DeviceConfig { api: Some(api) };
    let boringtun: DeviceHandle<_> =
        DeviceHandle::from_tun_name(obfuscator_socket_isch, TUN_NAME, config)
            .await
            .unwrap();
    boringtun
}

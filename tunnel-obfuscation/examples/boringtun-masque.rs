//! Boringtun + Masque = <3

use boringtun::device::{Device, DeviceConfig, DeviceHandle};
use boringtun::tun::tun_async_device;
use boringtun::udp::UdpTransportFactory;
use std::sync::Arc;
use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
};
use tun::AsyncDevice;
use tunnel_obfuscation::PacketChannelSimple;
use tunnel_obfuscation::quic::Quic;

pub const TUN_NAME: &'static str = "mullvadtun";

type X = (PacketChannelSimple, Arc<AsyncDevice>, Arc<AsyncDevice>);

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
    let quic = create_quic_obfuscator().await?;
    let boringtun = boringtun_main(obfuscator_channels).await.unwrap();
    Ok(())
}

pub async fn boringtun_main(udp_factory: PacketChannelSimple) -> io::Result<DeviceHandle<X>> {
    let (tun, tun_tx, tun_rx) = {
        let mut tun_config = tun::Configuration::default();
        tun_config.tun_name(TUN_NAME);
        #[cfg(target_os = "macos")]
        tun_config.platform_config(|p| {
            p.enable_routing(false);
        });
        let tun = tun::create_as_async(&tun_config)?;
        let tun_tx = Arc::new(tun);
        let tun_rx = Arc::clone(&tun_tx);
        (tun, tun_tx, tun_rx)
        //Ok(DeviceHandle::new(udp_factory, tun_tx, tun_rx, config).await)
    };

    let api = boringtun::device::api::ApiServer::default_unix_socket(TUN_NAME).unwrap();

    let config = DeviceConfig { api: Some(api) };

    let boringtun = DeviceHandle::new(udp_factory, tun_tx, tun_rx, config).await;

    log::info!("BoringTun started successfully"); // TODO: abort somehow
    Ok(boringtun)
}

pub async fn create_quic_obfuscator() -> io::Result<Quic> {
    todo!()
}

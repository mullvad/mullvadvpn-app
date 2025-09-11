//! Boringtun + Masque = <3

use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use boringtun::device::{DeviceConfig, DeviceHandle};
use boringtun::packet::{Packet, PacketBufPool};
use boringtun::tun::tun_async_device::AsyncDevice;
use boringtun::udp::{UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams};
use clap::Parser;
use tunnel_obfuscation::quic::Quic;
use tunnel_obfuscation::{Obfuscator, PacketChannelSimple, SimpleChannelRx, SimpleChannelTx};

pub const TUN_NAME: &str = "mullvadtun";

/// Tun device <-Arc<AsyncDevice>-> Boringtun <-PacketChannelSimple-> Masque client.
type DeviceTransports = (PacketChannelSimple, Arc<AsyncDevice>, Arc<AsyncDevice>);
type BoringtunIO = PacketChannelSimple;
type MasqueClientIO = (SimpleChannelTx, SimpleChannelRx);
// Boringtun -> writes to BoringtunIO -> ends up in MasqueClientIO.rx -> Masque client
//
// Boringtun <- ends up in BoringtunIO <- writes to MasqueClientIO.tx <- Masque client

type BoringtunDevice = DeviceHandle<DeviceTransports>;

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = ClientArgs::parse();
    let boringtun_settings = args.boringtun_settings();
    let obfuscator_settings = args.quic_settings();

    let (mut boringtun_io, obfuscator_io) = create_in_process_communication_channels();
    log::info!("Boringtun + Masque = <3");
    let quic = create_quic_obfuscator(obfuscator_io, obfuscator_settings).await;
    let obfuscator = Box::new(quic);
    let obfs = tokio::spawn(obfuscator.run());
    log::info!("Masque proxy client started successfully");

    log::info!("Starting BoringTun ..");
    // TODO: Something with boringtun is not working .. Masque is working :+1:
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let boringtun: BoringtunDevice = create_boringtun(boringtun_io, boringtun_settings).await;
    log::info!("BoringTun started successfully");

    // HACK: v
    /*
        let mut rw = {
            let source_v4 = Ipv4Addr::new(10, 0, 0, 1); // TODO: Is this correct? No.
            let params = UdpTransportFactoryParams {
                addr_v4: source_v4,
                addr_v6: Ipv6Addr::UNSPECIFIED,
                port: 1337,
                fwmark: None,
            };
            let bound = boringtun_io.bind(&params).await;
            bound.unwrap().0
        };

        // Send a fake packet to check if the masque server works.
        let packet = Packet::default();
        rw.0.send_to(packet, SocketAddr::from((Ipv4Addr::new(10, 0, 0, 1), 0)))
            .await
            .unwrap();

        let mut pool = PacketBufPool::new(1000);
        let (packet, _x) = rw.1.recv_from(&mut pool).await.unwrap();
        println!("Received packet {packet:#?}");
    */

    // TODO: run
    tokio::select! {
        _x = obfs => {
            log::info!("Exiting obfuscator");
        }
    }

    std::future::pending::<()>().await;

    Ok(())
}

/// Create bi-directional channels where Boringtun and Masque proxy can talk to each other.
fn create_in_process_communication_channels() -> (BoringtunIO, MasqueClientIO) {
    // god, forgive me
    let (obfs_tx, obfs_rx, boringtun_io) = {
        let capacity = 100;
        let source_v4 = SocketAddr::from((Ipv4Addr::new(10, 0, 0, 1), 0)); // TODO: Is this correct? No.
        tunnel_obfuscation::new_packet_channels(capacity, source_v4)
    };
    (boringtun_io, (obfs_tx, obfs_rx))
}

async fn create_quic_obfuscator(
    channels: MasqueClientIO,
    settings: tunnel_obfuscation::Settings,
) -> Quic {
    let obfuscator = tunnel_obfuscation::create_quic_obfuscator(&settings, channels)
        .await
        .unwrap();
    obfuscator.unwrap()
}

async fn create_boringtun(
    obfuscator_socket_isch: BoringtunIO,
    settings: BoringtunSettings,
) -> DeviceHandle<DeviceTransports> {
    let tun = settings.tun;
    let api = boringtun::device::api::ApiServer::default_unix_socket(&tun).unwrap();
    let config = DeviceConfig { api: Some(api) };
    let boringtun: DeviceHandle<_> =
        DeviceHandle::from_tun_name(obfuscator_socket_isch, &tun, config)
            .await
            .unwrap();
    boringtun
}

#[derive(Parser, Debug)]
pub struct ClientArgs {
    /// Tun device for Boringtun
    #[arg(long)]
    tun: String,
    /// Destination to forward to
    #[arg(long, short = 't')]
    target_addr: SocketAddr,

    /// Path to cert
    #[arg(long, short = 'c', required = false)]
    root_cert_path: Option<PathBuf>,

    /// Server address
    #[arg(long, short = 's')]
    server_addr: SocketAddr,

    /// Server hostname/authority
    #[arg(long, short = 'H')]
    server_hostname: Option<String>,

    /// Bind address
    #[arg(long, short = 'b', default_value = "127.0.0.1:0")]
    bind_addr: SocketAddr,

    /// Maximum packet size
    #[arg(long, short = 'S', default_value = "1280")]
    mtu: u16,

    /// Authorization header value to set
    #[arg(long, default_value = "test")]
    auth: Option<String>,
}

struct BoringtunSettings {
    tun: String,
}

impl ClientArgs {
    fn boringtun_settings(&self) -> BoringtunSettings {
        BoringtunSettings {
            tun: self.tun.clone(),
        }
    }

    /// Destination to forward to
    fn quic_settings(&self) -> tunnel_obfuscation::Settings {
        use tunnel_obfuscation::{Settings, quic};
        let ClientArgs {
            target_addr,
            server_addr,
            server_hostname,
            auth,
            ..
        } = self;
        let server_addr = *server_addr;
        let server_hostname = server_hostname.clone().unwrap_or("x.com".to_string());
        let token = auth.clone().unwrap_or("Test".to_string());
        let target_addr = *target_addr;

        Settings::Quic(quic::Settings::new(
            server_addr,
            server_hostname,
            quic::AuthToken::new(token).unwrap(),
            target_addr,
        ))
    }
}

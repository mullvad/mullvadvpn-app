#![allow(clippy::unused_async)]

use gotatun::{
    device::{DeviceBuilder, Peer},
    packet::{Packet, PacketBufPool},
    tun::{tun::Configuration, tun_async_device::TunDevice},
    udp::{UdpRecv, UdpSend, UdpTransportFactory, socket::UdpSocketFactory},
    x25519::{PublicKey, StaticSecret},
};
use ipnetwork::IpNetwork;
use mullvad_ios::tunnel_adapter::Multiplex;
use std::{
    net::{Ipv4Addr, SocketAddr},
    num::NonZero,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{Mutex, Notify, mpsc},
    time::{sleep, timeout},
};

type Device<U> = gotatun::device::Device<(U, TunDevice, TunDevice)>;

async fn create_device<U: UdpTransportFactory>(
    ip: Ipv4Addr,
    private_key: StaticSecret,
    peer_config: Peer,
    destination: Ipv4Addr,
    udp: U,
    listen_port: u16,
) -> anyhow::Result<Device<U>> {
    let device = gotatun::tun::tun::create_as_async(
        Configuration::default()
            .address(ip)
            .mtu(1500)
            .netmask((255, 255, 255, 0))
            .destination(destination)
            .up(),
    )?;
    let device = TunDevice::from_tun_device(device)?;
    let device = DeviceBuilder::new()
        .with_udp(udp)
        .with_ip(device)
        .with_private_key(private_key)
        .with_peer(peer_config)
        .with_listen_port(listen_port)
        .build()
        .await?;
    Ok(device)
}

async fn reciever(
    addr: Ipv4Addr,
    ready: Arc<Notify>,
    ports: &[u16],
    cancel: Arc<Notify>,
    mut pck_recvd: mpsc::UnboundedReceiver<()>,
    pcks_count: mpsc::UnboundedSender<usize>,
) -> anyhow::Result<()> {
    let mut plexer = Multiplex::listen_to(addr, ports)?;
    let mut pool = PacketBufPool::new(4096);
    ready.notify_one();
    let mut count = 0;
    loop {
        tokio::select! { biased;
            res = plexer.recv_from(&mut pool) => { res?; count += 1; continue },
            _ = sleep(Duration::from_secs_f32(0.5)) => {},
            _ = cancel.notified() => break,
        };
        if pck_recvd.try_recv().is_ok() {
            pcks_count.send(count)?;
            count = 0;
        }
    }
    Ok(())
}

async fn sender<const PCK_SIZE: usize>(
    other: Ipv4Addr,
    ports: &[u16],
    sockets: NonZero<u8>,
    mut packets: usize,
    pck_recvd: mpsc::UnboundedSender<()>,
    count_rx: Arc<Mutex<mpsc::UnboundedReceiver<usize>>>,
) -> anyhow::Result<usize> {
    let socket = Multiplex::new(
        sockets,
        ports
            .iter()
            .map(|p| SocketAddr::new(other.into(), *p))
            .collect(),
    )
    .await?;

    let tip = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
    let kb = [0u8; PCK_SIZE];
    let mut total_retries = 0;
    while packets != 0 {
        let mut i = 0;
        while i < packets {
            socket.send_to(Packet::copy_from(&kb), tip).await?;
            i += 1;
        }
        pck_recvd.send(())?;
        let count = count_rx.lock().await.recv().await.unwrap();
        let new_count = packets.saturating_sub(count);
        total_retries += new_count;
        packets = new_count;
    }

    Ok(total_retries)
}

const PCK_COUNT: usize = 2000000;
const PCK_SIZE: usize = 500;
const _: () = [()][(PCK_COUNT * PCK_SIZE) - 1_000_000_000];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let priv1 = StaticSecret::random();
    let ip1 = Ipv4Addr::new(10, 1, 72, 1);
    let priv2 = StaticSecret::random();
    let ip2 = Ipv4Addr::new(10, 1, 72, 2);

    let peer1 = Peer::new(PublicKey::from(&priv1))
        .with_endpoint("127.0.0.1:4001".parse().unwrap())
        .with_allowed_ip(IpNetwork::new(ip2.into(), 24)?);
    let peer2 = Peer::new(PublicKey::from(&priv2))
        .with_endpoint("127.0.0.1:4002".parse().unwrap())
        .with_allowed_ip(IpNetwork::new(ip1.into(), 24)?);

    eprintln!("Transfering 1gB of data ({PCK_SIZE}b packet sent {PCK_COUNT} times)");
    eprintln!("Ports\tSockets\tTime(s)\tFailure");

    let dev1 = create_device(ip1, priv1, peer2, ip2, UdpSocketFactory::default(), 4001).await?;
    let dev2 = create_device(ip2, priv2, peer1, ip1, UdpSocketFactory::default(), 4002).await?;

    for port_num in 0..12 {
        let ready = Arc::new(Notify::new());
        let cancel = Arc::new(Notify::new());

        let start = 5000;
        let mut ports = vec![start];
        ports.extend((start + 1)..(start + 1 + port_num));

        let (count_tx, count_rx) = mpsc::unbounded_channel();
        let count_rx = Arc::new(Mutex::new(count_rx));

        let (pck_tx, pck_rx) = mpsc::unbounded_channel();
        let t1 = tokio::task::spawn({
            let (ready, cancel, ports) = (ready.clone(), cancel.clone(), ports.clone());
            async move {
                reciever(ip1, ready, &ports, cancel, pck_rx, count_tx)
                    .await
                    .unwrap()
            }
        });

        ready.notified().await;

        for socket_count in 1..=12 {
            let ports = ports.clone();

            eprint!("{}\t{}", ports.len(), socket_count);
            let start = Instant::now();

            let send = timeout(
                Duration::from_secs(200),
                sender::<PCK_SIZE>(
                    ip1,
                    &ports,
                    NonZero::new(socket_count).unwrap(),
                    PCK_COUNT,
                    pck_tx.clone(),
                    count_rx.clone(),
                ),
            )
            .await;

            match send {
                Ok(o) => match o {
                    Ok(retries) => eprintln!(
                        "\t{:.02}\t{:.02}%",
                        start.elapsed().as_secs_f64(),
                        (retries as f64 / PCK_COUNT as f64) * 100.0
                    ),
                    Err(e) => eprintln!("\terr: {e}"),
                },
                Err(_) => eprintln!("\ttimeout"),
            };

            sleep(Duration::from_secs_f32(0.5)).await;
        }

        cancel.notify_waiters();
        t1.await?;
    }

    drop((dev1, dev2));
    Ok(())
}

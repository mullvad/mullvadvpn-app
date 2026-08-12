//! Benchmark: WireGuard packet to the remote, with and without the multiplexer.
//!
//! Both scenarios send one WireGuard data packet and wait for it to arrive at the relay:
//!
//! 1. **single_lwo** - one LWO obfuscator, reached over a loopback socket
//! 2. **multiplexer_lwo** - the same obfuscator behind the multiplexer, in connected mode
//!
//! The difference is what the multiplexer costs on top of the obfuscator itself.

use core::hint::black_box;
use criterion::{Criterion, criterion_group, criterion_main};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use talpid_net::bypass::NoopBypass;
use talpid_types::net::{obfuscation::LwoVersion, wireguard::PublicKey};
use tokio::{net::UdpSocket, sync::oneshot};
use tunnel_obfuscation::{LocalSocketObfuscator, lwo, multiplexer};

/// Control: plain UDP send and receive, with nothing in between.
fn bench_plain_udp(c: &mut Criterion) {
    let rt = runtime();

    let (wg, relay) = rt.block_on(async {
        let relay = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let wg = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        wg.connect(relay.local_addr().unwrap()).await.unwrap();
        (wg, relay)
    });

    bench_send_recv(c, "plain_udp/send_recv", &rt, &wg, &relay);
}

/// One LWO obfuscator via local socket.
fn bench_single_lwo(c: &mut Criterion) {
    let rt = runtime();

    let (wg, relay) = rt.block_on(async {
        let relay = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let settings = lwo_settings(relay.local_addr().unwrap());
        let obfuscator = tunnel_obfuscation::create_local_socket_obfuscator(&settings)
            .await
            .unwrap();
        let endpoint = obfuscator.endpoint();
        tokio::spawn(obfuscator.run());

        let (wg, _) = warm_up(endpoint, &relay).await;
        (wg, relay)
    });

    bench_send_recv(c, "single_lwo/send_recv", &rt, &wg, &relay);
}

/// LWO obfuscator behind the multiplexer.
fn bench_multiplexer_lwo(c: &mut Criterion) {
    let rt = runtime();

    let (wg, relay) = rt.block_on(async {
        let relay = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (selected_tx, _selected_rx) = oneshot::channel();
        let settings = multiplexer::Settings {
            transports: vec![multiplexer::Transport::Obfuscated(lwo_settings(
                relay.local_addr().unwrap(),
            ))],
            selected_transport: selected_tx,
        };
        let multiplexer = multiplexer::Multiplexer::new(Arc::new(NoopBypass), settings)
            .await
            .unwrap();
        let endpoint = multiplexer.endpoint();
        tokio::spawn(Box::new(multiplexer).run());

        let (wg, transport_addr) = warm_up(endpoint, &relay).await;

        // Reply, so that the multiplexer commits to this transport and enters connected mode.
        let (client_key, _) = keys();
        let mut response = wg_data_packet();
        lwo::obfuscate_thread_local(&mut response, client_key.as_bytes());
        let mut buf = vec![0u8; 65535];
        loop {
            relay.send_to(&response, transport_addr).await.unwrap();
            let received = tokio::time::timeout(Duration::from_millis(100), wg.recv(&mut buf));
            if received.await.is_ok() {
                break;
            }
        }

        (wg, relay)
    });

    bench_send_recv(c, "multiplexer_lwo/send_recv", &rt, &wg, &relay);
}

/// Send one packet from WireGuard's socket and wait for it to reach the relay.
fn bench_send_recv(
    c: &mut Criterion,
    name: &str,
    rt: &tokio::runtime::Runtime,
    wg: &UdpSocket,
    relay: &UdpSocket,
) {
    let packet = wg_data_packet();
    let mut recv_buf = vec![0u8; 65535];

    c.bench_function(name, |b| {
        b.iter(|| {
            rt.block_on(async {
                wg.send(black_box(&packet)).await.unwrap();
                relay.recv_from(&mut recv_buf).await.unwrap();
            });
        });
    });
}

/// Bind a socket in WireGuard's place, connect it to `endpoint`, and push packets through until
/// the relay sees one. Returns the socket and the address the packet arrived from.
///
/// The obfuscator is only reachable once it has seen a packet, and the multiplexer spawns its
/// first transport a moment after it starts running, so this retries.
async fn warm_up(endpoint: SocketAddr, relay: &UdpSocket) -> (UdpSocket, SocketAddr) {
    let wg = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    wg.connect(endpoint).await.unwrap();

    let packet = wg_data_packet();
    let mut buf = vec![0u8; 65535];
    loop {
        wg.send(&packet).await.unwrap();
        let received = tokio::time::timeout(Duration::from_millis(100), relay.recv_from(&mut buf));
        if let Ok(Ok((_, from))) = received.await {
            return (wg, from);
        }
    }
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn lwo_settings(relay_addr: SocketAddr) -> tunnel_obfuscation::Settings {
    let (client_public_key, server_public_key) = keys();
    tunnel_obfuscation::Settings::Lwo(lwo::Settings {
        server_addr: relay_addr,
        client_public_key,
        server_public_key,
        version: LwoVersion::V1,
    })
}

fn wg_data_packet() -> Vec<u8> {
    // Minimal valid WireGuard data packet header (type=4) + 1200-byte payload
    const DATA: u8 = 4;
    const DATA_OVERHEAD: usize = 32;
    const PAYLOAD: usize = 1200;
    let mut p = vec![0u8; DATA_OVERHEAD + PAYLOAD];
    p[0] = DATA;
    p
}

fn keys() -> (PublicKey, PublicKey) {
    let client = PublicKey::from_base64("8Ka2l4T0tVrSR5pkcsvRG++mBlxfuf8XOxpqBkOCikU=").unwrap();
    let server = PublicKey::from_base64("4EkA4c160oQgN/YaNR9GN3gLMevXEfx5hnlc9jYmw14=").unwrap();
    (client, server)
}

criterion_group!(
    benches,
    bench_plain_udp,
    bench_single_lwo,
    bench_multiplexer_lwo
);
criterion_main!(benches);

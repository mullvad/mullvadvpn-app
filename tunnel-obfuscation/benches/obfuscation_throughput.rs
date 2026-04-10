//! Benchmark: LWO proxy round-trip vs inline obfuscation
//!
//! Compares three scenarios for sending a single WireGuard UDP packet to a relay:
//!
//! 1. **baseline** - plain UDP send+recv, no obfuscation
//! 2. **inline_lwo** - obfuscate in-process (`obfuscate_thread_local`) then send
//! 3. **proxy_lwo** - send through the `Lwo` localhost proxy (old approach)
//!
//! The difference `proxy_lwo - inline_lwo` is the cost of the extra kernel
//! round-trip.

use criterion::{Criterion, criterion_group, criterion_main};
use talpid_types::net::wireguard::PublicKey;
use tokio::net::UdpSocket;
use tunnel_obfuscation::{
    Obfuscator,
    lwo::{self, Lwo, Settings, obfuscate_thread_local},
};

/// Baseline: plain UDP send -> recv, no obfuscation.
fn bench_baseline(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let (sender, relay) = rt.block_on(async {
        let relay = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let sender = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        sender.connect(relay.local_addr().unwrap()).await.unwrap();
        (sender, relay)
    });

    let packet = make_wg_data_packet();
    let mut recv_buf = vec![0u8; 65535];

    c.bench_function("baseline/send_recv", |b| {
        b.iter(|| {
            rt.block_on(async {
                sender.send(&packet).await.unwrap();
                relay.recv(&mut recv_buf).await.unwrap();
            });
        });
    });
}

/// Inline LWO: obfuscate in-process with `obfuscate_thread_local`, then send.
/// This is what `ObfuscatingTransportFactory::Lwo` does on every `send_to`.
fn bench_inline_lwo(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let (client_key, server_key) = keys();
    let tx_key = *server_key.as_bytes();
    let rx_key = *client_key.as_bytes();

    let (sender, relay) = rt.block_on(async {
        let relay = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let sender = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        sender.connect(relay.local_addr().unwrap()).await.unwrap();
        (sender, relay)
    });

    let packet = make_wg_data_packet();
    let mut recv_buf = vec![0u8; 65535];

    c.bench_function("inline_lwo/send_recv", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut pkt = packet.clone();
                obfuscate_thread_local(&mut pkt, &tx_key);
                sender.send(&pkt).await.unwrap();
                let n = relay.recv(&mut recv_buf).await.unwrap();
                // Deobfuscate to confirm correctness (mirrors what LwoRecv does)
                lwo::deobfuscate(&mut recv_buf[..n], &rx_key);
            });
        });
    });
}

/// Proxy LWO: the old approach - WG sends to localhost proxy which re-sends to relay.
/// Setup: WG socket -> Lwo proxy task -> relay socket.
fn bench_proxy_lwo(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let (client_key, server_key) = keys();

    let (wg_socket, relay, proxy_addr) = rt.block_on(async {
        let relay = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let settings = Settings {
            server_addr: relay.local_addr().unwrap(),
            client_public_key: client_key.clone(),
            server_public_key: server_key.clone(),
            #[cfg(target_os = "linux")]
            fwmark: None,
        };
        let lwo = Lwo::new(&settings).await.unwrap();
        let proxy_addr = lwo.endpoint();
        tokio::spawn((Box::new(lwo) as Box<dyn Obfuscator>).run());

        // Warm up: send one packet so the proxy connects its client socket
        let wg = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let mut buf = make_wg_data_packet();
        wg.send_to(&buf, proxy_addr).await.unwrap();
        relay.recv(&mut buf).await.unwrap();

        (wg, relay, proxy_addr)
    });

    let packet = make_wg_data_packet();
    let mut recv_buf = vec![0u8; 65535];

    c.bench_function("proxy_lwo/send_recv", |b| {
        b.iter(|| {
            rt.block_on(async {
                wg_socket.send_to(&packet, proxy_addr).await.unwrap();
                relay.recv(&mut recv_buf).await.unwrap();
            });
        });
    });
}

fn make_wg_data_packet() -> Vec<u8> {
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

criterion_group!(benches, bench_baseline, bench_inline_lwo, bench_proxy_lwo);
criterion_main!(benches);

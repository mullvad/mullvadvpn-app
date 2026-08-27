use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::RngCore;
use talpid_types::net::wireguard::PublicKey;

fn obfuscate(c: &mut Criterion) {
    let pubkey = PublicKey::from_base64("8Ka2l4T0tVrSR5pkcsvRG++mBlxfuf8XOxpqBkOCikU=").unwrap();
    let rng = &mut rand::rng();

    let mut group = c.benchmark_group("lwo");
    group.throughput(criterion::Throughput::Bytes(fake_packet().len() as u64));
    group.bench_function(BenchmarkId::new("obfuscate", "v1"), |b| {
        b.iter_batched(
            fake_packet,
            |mut packet| {
                tunnel_obfuscation::lwo::obfuscate(rng, &mut packet, pubkey.as_bytes());
                packet
            },
            criterion::BatchSize::LargeInput,
        );
    });
    group.bench_function(BenchmarkId::new("obfuscate", "v2"), |b| {
        b.iter_batched(
            fake_packet,
            |mut packet| {
                tunnel_obfuscation::lwo::v2::obfuscate(&mut packet, pubkey.as_bytes());
                packet
            },
            criterion::BatchSize::LargeInput,
        );
    });
    group.finish();
}

fn deobfuscate(c: &mut Criterion) {
    let pubkey = PublicKey::from_base64("8Ka2l4T0tVrSR5pkcsvRG++mBlxfuf8XOxpqBkOCikU=").unwrap();
    let rng = &mut rand::rng();

    let mut group = c.benchmark_group("lwo");
    group.throughput(criterion::Throughput::Bytes(
        obfuscated_fake_packet(rng, pubkey.as_bytes()).len() as u64,
    ));
    group.bench_function(BenchmarkId::new("deobfuscate", "v1"), |b| {
        b.iter_batched(
            || obfuscated_fake_packet(rng, pubkey.as_bytes()),
            |mut packet| {
                tunnel_obfuscation::lwo::deobfuscate(&mut packet, pubkey.as_bytes());
                packet
            },
            criterion::BatchSize::LargeInput,
        );
    });
    group.bench_function(BenchmarkId::new("deobfuscate", "v2"), |b| {
        b.iter_batched(
            || obfuscated_fake_packet_v2(pubkey.as_bytes()),
            |mut packet| {
                tunnel_obfuscation::lwo::v2::deobfuscate(&mut packet, pubkey.as_bytes());
                packet
            },
            criterion::BatchSize::LargeInput,
        );
    });
    group.finish();
}

/// Compare the full v2 send path, [`pad`] followed by [`obfuscate`], to `obfuscate` alone.
///
/// [`pad`]: tunnel_obfuscation::lwo::v2::pad
/// [`obfuscate`]: tunnel_obfuscation::lwo::v2::obfuscate
fn pad_and_obfuscate(c: &mut Criterion) {
    let pubkey = PublicKey::from_base64("8Ka2l4T0tVrSR5pkcsvRG++mBlxfuf8XOxpqBkOCikU=").unwrap();
    let key = pubkey.as_bytes();

    let mut group = c.benchmark_group("lwo_v2_send");

    // Data packets are never padded, so `pad` only costs a message type check.
    group.throughput(criterion::Throughput::Bytes(fake_packet().len() as u64));
    group.bench_function(BenchmarkId::new("data", "obfuscate"), |b| {
        b.iter_batched(
            fake_packet,
            |mut packet| {
                tunnel_obfuscation::lwo::v2::obfuscate(&mut packet, key);
                packet
            },
            criterion::BatchSize::LargeInput,
        );
    });
    group.bench_function(BenchmarkId::new("data", "pad_and_obfuscate"), |b| {
        b.iter_batched(
            fake_packet,
            |packet| pad_and_obfuscate_packet(packet, key),
            criterion::BatchSize::LargeInput,
        );
    });

    // Handshakes are padded, which means allocating a new buffer and filling the padding with
    // random bytes.
    group.throughput(criterion::Throughput::Bytes(HANDSHAKE_INIT_SZ as u64));
    group.bench_function(BenchmarkId::new("handshake_init", "obfuscate"), |b| {
        b.iter_batched(
            fake_handshake_init,
            |mut packet| {
                tunnel_obfuscation::lwo::v2::obfuscate(&mut packet, key);
                packet
            },
            criterion::BatchSize::LargeInput,
        );
    });
    group.bench_function(
        BenchmarkId::new("handshake_init", "pad_and_obfuscate"),
        |b| {
            b.iter_batched(
                fake_handshake_init,
                |packet| pad_and_obfuscate_packet(packet, key),
                criterion::BatchSize::LargeInput,
            );
        },
    );

    group.finish();
}

type MessageType = u8;

const HANDSHAKE_INIT: MessageType = 1;
const HANDSHAKE_INIT_SZ: usize = 148;
const DATA: MessageType = 4;
const DATA_OVERHEAD_SZ: usize = 32;

fn fake_packet() -> Vec<u8> {
    let mut packet = vec![0u8; DATA_OVERHEAD_SZ + 1200];
    packet[0] = DATA;
    rand::rng().fill_bytes(&mut packet[DATA_OVERHEAD_SZ..]);
    packet
}

fn fake_handshake_init() -> Vec<u8> {
    let mut packet = vec![0u8; HANDSHAKE_INIT_SZ];
    packet[0] = HANDSHAKE_INIT;
    rand::rng().fill_bytes(&mut packet[4..]);
    packet
}

fn pad_and_obfuscate_packet(packet: Vec<u8>, key: &[u8; 32]) -> Vec<u8> {
    let mut packet = tunnel_obfuscation::lwo::v2::pad(&packet).unwrap_or(packet);
    tunnel_obfuscation::lwo::v2::obfuscate(&mut packet, key);
    packet
}

fn obfuscated_fake_packet(rng: &mut impl RngCore, key: &[u8; 32]) -> Vec<u8> {
    let mut packet = fake_packet();
    tunnel_obfuscation::lwo::obfuscate(rng, &mut packet, key);
    packet
}

fn obfuscated_fake_packet_v2(key: &[u8; 32]) -> Vec<u8> {
    // Data packets are never padded, so no `padding_len` step is needed here.
    let mut packet = fake_packet();
    tunnel_obfuscation::lwo::v2::obfuscate(&mut packet, key);
    packet
}

criterion_group!(benches, obfuscate, deobfuscate, pad_and_obfuscate);
criterion_main!(benches);

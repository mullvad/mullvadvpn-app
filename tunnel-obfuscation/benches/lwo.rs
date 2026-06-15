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

type MessageType = u8;

const DATA: MessageType = 4;
const DATA_OVERHEAD_SZ: usize = 32;

fn fake_packet() -> Vec<u8> {
    let mut packet = vec![0u8; DATA_OVERHEAD_SZ + 1200];
    packet[0] = DATA;
    rand::rng().fill_bytes(&mut packet[DATA_OVERHEAD_SZ..]);
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

criterion_group!(benches, obfuscate, deobfuscate);
criterion_main!(benches);

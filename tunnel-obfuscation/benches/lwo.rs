use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::RngCore;
use talpid_types::net::wireguard::PublicKey;
use tunnel_obfuscation::lwo::new_rng;

fn obfuscate(c: &mut Criterion) {
    let pubkey = PublicKey::from_base64("8Ka2l4T0tVrSR5pkcsvRG++mBlxfuf8XOxpqBkOCikU=").unwrap();
    let mut rng = new_rng();

    let mut group = c.benchmark_group("lwo");
    group.throughput(criterion::Throughput::Bytes(fake_packet().len() as u64));
    group.bench_function(BenchmarkId::new("obfuscate", fake_packet().len()), |b| {
        b.iter_batched(
            fake_packet,
            |mut packet| {
                tunnel_obfuscation::lwo::obfuscate(&mut rng, &mut packet, pubkey.as_bytes())
            },
            criterion::BatchSize::LargeInput,
        );
    });
    group.finish();
}

fn deobfuscate(c: &mut Criterion) {
    let pubkey = PublicKey::from_base64("8Ka2l4T0tVrSR5pkcsvRG++mBlxfuf8XOxpqBkOCikU=").unwrap();
    let mut rng = new_rng();

    let mut group = c.benchmark_group("lwo");
    group.throughput(criterion::Throughput::Bytes(
        obfuscated_fake_packet(&mut rng, pubkey.as_bytes()).len() as u64,
    ));
    group.bench_function(BenchmarkId::new("deobfuscate", fake_packet().len()), |b| {
        b.iter_batched(
            || obfuscated_fake_packet(&mut rng, pubkey.as_bytes()),
            |mut packet| tunnel_obfuscation::lwo::deobfuscate(&mut packet, pubkey.as_bytes()),
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
    rand::thread_rng().fill_bytes(&mut packet[DATA_OVERHEAD_SZ..]);
    packet
}

fn obfuscated_fake_packet(rng: &mut impl RngCore, key: &[u8; 32]) -> Vec<u8> {
    let mut packet = fake_packet();
    tunnel_obfuscation::lwo::obfuscate(rng, &mut packet, key);
    packet
}

criterion_group!(benches, obfuscate, deobfuscate);
criterion_main!(benches);

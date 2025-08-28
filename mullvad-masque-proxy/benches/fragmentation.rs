use bytes::Bytes;
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use mullvad_masque_proxy::{
    FRAGMENT_HEADER_SIZE_FRAGMENTED,
    fragment::{FRAGMENT_BUFFER_CAP, Fragments, fragment_packet},
};
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use talpid_tunnel::IPV4_HEADER_SIZE;

const MAX_PAYLOAD_SIZE: u16 = 1280 - FRAGMENT_HEADER_SIZE_FRAGMENTED - IPV4_HEADER_SIZE;

fn assemble_fragment_ordered(c: &mut Criterion) {
    let mut group = c.benchmark_group("fragmentation_reconstruction");

    for (n_packets, payload_len) in [(10, 30000u16), (100, 1500)] {
        let mut fragment_buf = Vec::with_capacity(FRAGMENT_BUFFER_CAP);
        for i in 0..n_packets {
            let packet_id = i;
            let mut payload = Bytes::from(vec![i as u8; payload_len as usize]);

            fragment_buf.extend(
                &mut fragment_packet(
                    MAX_PAYLOAD_SIZE + FRAGMENT_HEADER_SIZE_FRAGMENTED,
                    &mut payload,
                    packet_id,
                )
                .unwrap(),
            );
        }
        let n_fragments = fragment_buf.len();
        assert!(
            n_fragments <= FRAGMENT_BUFFER_CAP,
            "Too many fragments generated"
        );
        group.throughput(criterion::Throughput::Bytes(
            (n_packets * payload_len) as u64,
        ));

        group.bench_with_input(
            BenchmarkId::new(
                "assemble_fragment_ordered",
                format!("{n_packets}pkts_{payload_len}B_{n_fragments}frags"),
            ),
            &fragment_buf,
            |b, fragment_buf| {
                b.iter_batched(
                    || (fragment_buf.clone(), Fragments::default()),
                    |(f, mut fragments)| {
                        for frag in f {
                            fragments.handle_incoming_packet(frag).unwrap();
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

fn assemble_fragment_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("fragmentation_reconstruction");

    for (n_packets, payload_len) in [(10, 30000u16), (100, 1500)] {
        let mut fragment_buf = Vec::with_capacity(FRAGMENT_BUFFER_CAP);
        for i in 0..n_packets {
            let packet_id = i;
            let mut payload = Bytes::from(vec![i as u8; payload_len as usize]);

            fragment_buf.extend(
                &mut fragment_packet(
                    MAX_PAYLOAD_SIZE + FRAGMENT_HEADER_SIZE_FRAGMENTED,
                    &mut payload,
                    packet_id,
                )
                .unwrap(),
            );
        }
        let n_fragments = fragment_buf.len();
        assert!(
            n_fragments <= FRAGMENT_BUFFER_CAP,
            "Too many fragments generated"
        );
        group.throughput(criterion::Throughput::Bytes(
            (n_packets * payload_len) as u64,
        ));
        let mut rng = StdRng::seed_from_u64(42);
        fragment_buf.shuffle(&mut rng);

        group.bench_with_input(
            BenchmarkId::new(
                "assemble_fragment_random",
                format!("{n_packets}pkts_{payload_len}B_{n_fragments}frags"),
            ),
            &fragment_buf,
            |b, fragment_buf| {
                b.iter_batched(
                    || (fragment_buf.clone(), Fragments::default()),
                    |(f, mut fragments)| {
                        for frag in f {
                            fragments.handle_incoming_packet(frag).unwrap();
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}
criterion_group!(benches, assemble_fragment_ordered, assemble_fragment_random);
criterion_main!(benches);

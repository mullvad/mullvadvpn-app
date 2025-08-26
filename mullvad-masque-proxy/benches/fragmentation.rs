use std::hint::black_box;

use bytes::Bytes;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use mullvad_masque_proxy::{
    FRAGMENT_HEADER_SIZE_FRAGMENTED,
    fragment::{DefragReceived, FRAGMENT_BUFFER_CAP, Fragments, fragment_packet},
};
use rand::{seq::SliceRandom, thread_rng};
use talpid_tunnel::IPV4_HEADER_SIZE;

const MAX_PAYLOAD_SIZE: u16 = 1280 - FRAGMENT_HEADER_SIZE_FRAGMENTED - IPV4_HEADER_SIZE;

fn fragmentation_reconstruction(c: &mut Criterion) {
    let mut group = c.benchmark_group("fragmentation_reconstruction");

    for (n_packets, payload_len) in [(10, 30000u16), (100, 1500), (200, 800)] {
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
        fragment_buf.shuffle(&mut thread_rng());

        group.bench_with_input(
            BenchmarkId::new(
                "fragmentation_reconstruction",
                format!("{n_packets}pkts_{payload_len}B_{n_fragments}frags"),
            ),
            &fragment_buf,
            |b, f| {
                b.iter(|| {
                    let mut fragments = Fragments::default();

                    assert_eq!(
                        f.iter()
                            .map(|f| fragments
                                .handle_incoming_packet(black_box(f.clone()))
                                .unwrap())
                            .filter(|res| matches!(res, DefragReceived::Reassembled(_)))
                            .count(),
                        n_packets as usize
                    );
                })
            },
        );
    }
    group.finish();
}
criterion_group!(benches, fragmentation_reconstruction);
criterion_main!(benches);

use criterion::{Criterion, criterion_group, criterion_main};
use talpid_types::net::is_ip_allowed_in_tunnel;

fn allowed_nets(c: &mut Criterion) {
    let ip = "1.1.1.1".parse().unwrap();

    c.bench_function("is_ip_allowed_in_tunnel", |b| {
        b.iter(|| is_ip_allowed_in_tunnel(ip));
    });
}

criterion_group!(benches, allowed_nets);
criterion_main!(benches);

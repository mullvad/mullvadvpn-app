//! Benchmark tests for log redaction regex performance.
//!
//! These benchmarks mirror the Swift LogRedactorBenchmarkTests to allow
//! direct comparison of regex performance between Rust and Swift.
//!
//! Run with: cargo bench -p mullvad-ios

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use regex::Regex;
use std::sync::LazyLock;

// Regex patterns matching those used in Swift's LogRedactor

/// IPv4 pattern from https://www.regular-expressions.info/ip.html
static IPV4_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\b"
    ).unwrap()
});

/// IPv6 pattern from https://stackoverflow.com/a/17871737
static IPV6_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
        (
        ([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|
        ([0-9a-fA-F]{1,4}:){1,7}:|
        ([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|
        ([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|
        ([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|
        ([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|
        ([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|
        [0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|
        :((:[0-9a-fA-F]{1,4}){1,7}|:)|
        fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|
        ::(ffff(:0{1,4}){0,1}:){0,1}
        ((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}
        (25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|
        ([0-9a-fA-F]{1,4}:){1,4}:
        ((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}
        (25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])
        )"
    ).unwrap()
});

/// Account number pattern: 16 consecutive digits
static ACCOUNT_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{16}").unwrap());

const REDACTED: &str = "[REDACTED]";
const REDACTED_ACCOUNT: &str = "[REDACTED ACCOUNT NUMBER]";

fn redact_ipv4(input: &str) -> String {
    IPV4_REGEX.replace_all(input, REDACTED).into_owned()
}

fn redact_ipv6(input: &str) -> String {
    IPV6_REGEX.replace_all(input, REDACTED).into_owned()
}

fn redact_account(input: &str) -> String {
    ACCOUNT_REGEX
        .replace_all(input, REDACTED_ACCOUNT)
        .into_owned()
}

fn redact_all(input: &str) -> String {
    let result = redact_ipv4(input);
    let result = redact_ipv6(&result);
    redact_account(&result)
}

// Test data matching Swift benchmarks
const SHORT_IPV4_MESSAGE: &str = "Connected to 192.168.1.1 successfully";
const SHORT_IPV4_FULL_LINE: &str =
    "[2026-01-29 10:30:45][TunnelManager][info] Connected to 192.168.1.1 successfully";

const SHORT_IPV6_MESSAGE: &str = "Connected to 2001:db8:85a3::8a2e:370:7334 successfully";
const SHORT_IPV6_FULL_LINE: &str =
    "[2026-01-29 10:30:45][TunnelManager][info] Connected to 2001:db8:85a3::8a2e:370:7334 successfully";

const LONG_MESSAGE: &str = "Tunnel connection established. Primary endpoint: 192.168.1.1:51820, \
backup endpoint: 10.0.0.1:51820. IPv6 addresses: 2001:db8:85a3::8a2e:370:7334, \
fe80::1%en0. Account verification completed for user session. \
DNS servers configured: 192.168.1.53, 8.8.8.8, 2001:4860:4860::8888. \
Gateway: 192.168.1.254. Network interface ready.";

const LONG_FULL_LINE: &str = "[2026-01-29 10:30:45][TunnelManager][info] pid=12345 session=abc123 \
Tunnel connection established. Primary endpoint: 192.168.1.1:51820, \
backup endpoint: 10.0.0.1:51820. IPv6 addresses: 2001:db8:85a3::8a2e:370:7334, \
fe80::1%en0. Account verification completed for user session. \
DNS servers configured: 192.168.1.53, 8.8.8.8, 2001:4860:4860::8888. \
Gateway: 192.168.1.254. Network interface ready.";

const NO_MATCH_MESSAGE: &str = "Application started successfully";
const NO_MATCH_FULL_LINE: &str =
    "[2026-01-29 10:30:45][AppDelegate][debug] Application started successfully";

const ACCOUNT_MESSAGE: &str = "Login attempt for account 1234567890123456";
const ACCOUNT_FULL_LINE: &str =
    "[2026-01-29 10:30:45][Auth][info] Login attempt for account 1234567890123456";

fn bench_ipv4(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipv4");

    group.bench_function("message_only", |b| {
        b.iter(|| redact_all(black_box(SHORT_IPV4_MESSAGE)))
    });

    group.bench_function("full_line", |b| {
        b.iter(|| redact_all(black_box(SHORT_IPV4_FULL_LINE)))
    });

    group.finish();
}

fn bench_ipv6(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipv6");

    group.bench_function("message_only", |b| {
        b.iter(|| redact_all(black_box(SHORT_IPV6_MESSAGE)))
    });

    group.bench_function("full_line", |b| {
        b.iter(|| redact_all(black_box(SHORT_IPV6_FULL_LINE)))
    });

    // IPv6-only redaction to isolate the regex cost
    group.bench_function("regex_only_message", |b| {
        b.iter(|| redact_ipv6(black_box(SHORT_IPV6_MESSAGE)))
    });

    group.bench_function("regex_only_full_line", |b| {
        b.iter(|| redact_ipv6(black_box(SHORT_IPV6_FULL_LINE)))
    });

    group.finish();
}

fn bench_long_messages(c: &mut Criterion) {
    let mut group = c.benchmark_group("long");

    group.bench_function("message_only", |b| {
        b.iter(|| redact_all(black_box(LONG_MESSAGE)))
    });

    group.bench_function("full_line", |b| {
        b.iter(|| redact_all(black_box(LONG_FULL_LINE)))
    });

    group.finish();
}

fn bench_no_matches(c: &mut Criterion) {
    let mut group = c.benchmark_group("no_matches");

    group.bench_function("message_only", |b| {
        b.iter(|| redact_all(black_box(NO_MATCH_MESSAGE)))
    });

    group.bench_function("full_line", |b| {
        b.iter(|| redact_all(black_box(NO_MATCH_FULL_LINE)))
    });

    group.finish();
}

fn bench_account(c: &mut Criterion) {
    let mut group = c.benchmark_group("account");

    group.bench_function("message_only", |b| {
        b.iter(|| redact_all(black_box(ACCOUNT_MESSAGE)))
    });

    group.bench_function("full_line", |b| {
        b.iter(|| redact_all(black_box(ACCOUNT_FULL_LINE)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_ipv4,
    bench_ipv6,
    bench_long_messages,
    bench_no_matches,
    bench_account,
);
criterion_main!(benches);

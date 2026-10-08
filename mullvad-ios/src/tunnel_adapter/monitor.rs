//! Connectivity state machine for a tunnel. See [`ConnectivityMonitor`].

use std::{
    ops::RangeInclusive,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// How often the tunnel is sampled while establishing.
const ESTABLISH_INTERVAL: Duration = Duration::from_millis(50);

/// Interval between the echo requests sent while establishing.
const ESTABLISH_PING_INTERVAL: Duration = Duration::from_secs(3);

/// How often the tunnel is sampled once connected.
const MONITOR_INTERVAL: Duration = Duration::from_millis(500);

/// How long user traffic may go unanswered before the tunnel is probed.
const UNANSWERED_TX_TIMEOUT: Duration = Duration::from_secs(7);

/// How long the tunnel may go without any traffic before it is probed once. Drawn anew after
/// each rx.
const IDLE_TIMEOUT: RangeInclusive<Duration> = Duration::from_secs(20)..=Duration::from_secs(45);

/// How long a probe waits for any rx before the tunnel is considered dead.
const PROBE_TIMEOUT: Duration = Duration::from_secs(15);

/// ICMP echo bursts. Should send multiple to avoid a single dropped packet failing a tunnel.
const PROBE_BURSTS: [(Duration, u32); 2] = [(Duration::ZERO, 2), (Duration::from_secs(1), 3)];

/// Interval between the single echo requests that follow the bursts.
const PROBE_INTERVAL: Duration = Duration::from_secs(2);

/// The counters of a running tunnel, read at one point in time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TunnelStats {
    /// Packets of user traffic sent into the tunnel.
    pub user_tx: u64,
    /// Bytes received by the innermost device: the exit in multihop, the only device otherwise.
    pub rx_bytes: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TunnelStatus {
    /// Nothing to do until the next sample.
    Healthy,
    /// Send this many echo requests.
    RequiresProbing(u32),
    /// First packet has been received.
    Connected,
    /// Nothing was received in time: by the establish deadline, or during a probe.
    TimedOut,
}

/// Decides from the tunnel's counters whether it is connected, needs pinging, or is dead.
///
/// Pings are requested:
///   * until anything is received, every [`ESTABLISH_PING_INTERVAL`]; the caller gives up at
///     the deadline
///   * once user traffic has gone unanswered for [`UNANSWERED_TX_TIMEOUT`]
///   * once the tunnel has gone without any traffic for a random [`IDLE_TIMEOUT`], a single
///     time until user traffic is sent again
///
/// A probe sends echo requests per [`PROBE_BURSTS`] and then every [`PROBE_INTERVAL`], and
/// declares the tunnel dead if nothing is received within [`PROBE_TIMEOUT`]. If any data is
/// received during a probe, the probe is cancelled and the tunnel is considered to be up.
pub struct ConnectivityMonitor {
    /// When the last establishing echo request was sent.
    last_ping: Option<Instant>,
    /// When to give up establishing.
    deadline: Instant,
    tx_packets: u64,
    rx_bytes: usize,
    /// When rx or tx last changed.
    last_traffic: Instant,
    /// How long after `last_traffic` the idle probe is due.
    idle_timeout: Duration,
    /// Whether the idle probe has been sent since the last tx.
    idle_probed: bool,
    /// When the current probe starts, or started.
    probe_at: Option<Instant>,
    /// Echo requests sent for the current probe.
    pings_sent: u32,
    /// Debug totals over the monitor's life.
    pings_total: u32,
    probes_total: u32,
}

impl ConnectivityMonitor {
    /// Starts establishing if `stats` has no rx yet, otherwise monitoring from `stats` on.
    pub fn new(now: Instant, stats: TunnelStats, deadline: Instant) -> Self {
        Self {
            last_ping: None,
            deadline,
            tx_packets: stats.user_tx,
            rx_bytes: stats.rx_bytes,
            last_traffic: now,
            idle_timeout: rand::random_range(IDLE_TIMEOUT),
            idle_probed: false,
            probe_at: None,
            pings_sent: 0,
            pings_total: 0,
            probes_total: 0,
        }
    }

    fn connected(&self) -> bool {
        self.rx_bytes > 0
    }

    /// How long to wait between samples.
    pub fn interval(&self) -> Duration {
        if self.connected() {
            MONITOR_INTERVAL
        } else {
            ESTABLISH_INTERVAL
        }
    }

    /// Forget any probe, pending or in progress, e.g. after a suspension.
    pub fn reset(&mut self) {
        self.last_ping = None;
        self.probe_at = None;
        self.pings_sent = 0;
    }

    /// Compare `stats` to the previous snapshot and decide, as of `now`, whether the tunnel is
    /// connected, should be pinged, is dead, or needs nothing.
    pub fn evaluate_connectivity(&mut self, now: Instant, stats: TunnelStats) -> TunnelStatus {
        if !self.connected() {
            return self.establish(now, stats);
        }
        self.record(now, stats);
        self.probe(now)
    }

    /// Connected on the first rx; until then a ping every [`ESTABLISH_PING_INTERVAL`].
    /// Evaluates if:
    /// * tunnel should be pinged
    /// * tunnel connection is established
    /// * connection attempt has failed
    fn establish(&mut self, now: Instant, stats: TunnelStats) -> TunnelStatus {
        if stats.rx_bytes > 0 {
            // Count traffic from here on.
            self.tx_packets = stats.user_tx;
            self.rx_bytes = stats.rx_bytes;
            self.last_traffic = now;
            return TunnelStatus::Connected;
        }
        if now >= self.deadline {
            log::warn!("Nothing received by the establish deadline");
            return TunnelStatus::TimedOut;
        }
        let ping_due = self
            .last_ping
            .is_none_or(|last_ping| now.duration_since(last_ping) >= ESTABLISH_PING_INTERVAL);
        if !ping_due {
            return TunnelStatus::Healthy;
        }
        self.last_ping = Some(now);
        self.pings_total += 1;
        log::trace!(
            "establish ping, {} ping(s) in total, at {}",
            self.pings_total,
            unix_millis()
        );
        TunnelStatus::RequiresProbing(1)
    }

    /// Update internal counters with the new stats.
    /// If any data was received, no more probing is required until either:
    ///     * data is sent again and no response is received in [`UNANSWERED_TX_TIMEOUT`]
    ///     * no data is sent or received between roughly [`IDLE_TIMEOUT`]
    fn record(&mut self, now: Instant, stats: TunnelStats) {
        if stats.rx_bytes != self.rx_bytes {
            if self.pings_sent > 0 {
                log::debug!("Probe answered after {} ping(s)", self.pings_sent);
            }
            self.rx_bytes = stats.rx_bytes;
            self.last_traffic = now;
            self.idle_timeout = rand::random_range(IDLE_TIMEOUT);
            self.reset();
        }
        if stats.user_tx != self.tx_packets {
            self.tx_packets = stats.user_tx;
            self.last_traffic = now;
            self.idle_probed = false;
            self.probe_at.get_or_insert(now + UNANSWERED_TX_TIMEOUT);
        }
        if !self.idle_probed
            && self.probe_at.is_none()
            && now.duration_since(self.last_traffic) >= self.idle_timeout
        {
            log::debug!("No traffic at all for a while, starting to probe");
            self.idle_probed = true;
            self.probe_at = Some(now);
        }
    }

    /// Nothing before the probe starts; then the pings that have come due, and [`TunnelStatus::TimedOut`]
    /// once [`PROBE_TIMEOUT`] has passed.
    fn probe(&mut self, now: Instant) -> TunnelStatus {
        let Some(probe_at) = self.probe_at else {
            return TunnelStatus::Healthy;
        };
        let Some(probing_for) = now.checked_duration_since(probe_at) else {
            return TunnelStatus::Healthy;
        };
        if probing_for >= PROBE_TIMEOUT {
            log::warn!(
                "No rx in {probing_for:?} after {} ping(s)",
                self.pings_sent,
            );
            return Action::TimedOut;
        }

        let due = pings_due(probing_for);
        let to_send = due - self.pings_sent;
        if self.pings_sent == 0 && to_send > 0 {
            self.probes_total += 1;
            log::trace!(
                "probe #{} started at {}",
                self.probes_total,
                unix_millis()
            );
        }
        self.pings_sent = due;
        if to_send == 0 {
            TunnelStatus::Healthy
        } else {
            self.pings_total += to_send;
            log::trace!(
                "sending {to_send} ping(s), {} ping(s) and {} probe(s) in total, at {}",
                self.pings_total,
                self.probes_total,
                unix_millis()
            );
            TunnelStatus::RequiresProbing(to_send)
        }
    }
}

/// Wall clock in milliseconds since the Unix epoch, for debug logs.
fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis())
}

/// Total echo requests scheduled within `probing_for` of the probe start.
fn pings_due(probing_for: Duration) -> u32 {
    let bursts: u32 = PROBE_BURSTS
        .iter()
        .filter(|(offset, _)| *offset <= probing_for)
        .map(|(_, count)| count)
        .sum();
    let (last_burst, _) = PROBE_BURSTS[PROBE_BURSTS.len() - 1];
    let singles = probing_for
        .checked_sub(last_burst + PROBE_INTERVAL)
        .map_or(0, |since_first| {
            let intervals = since_first.as_millis() / PROBE_INTERVAL.as_millis();
            u32::try_from(intervals)
                .unwrap_or(u32::MAX)
                .saturating_add(1)
        });
    bursts.saturating_add(singles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng, rngs::StdRng};

    fn secs(s: f32) -> Duration {
        Duration::from_secs_f32(s)
    }

    fn quiet(user_tx: u64) -> TunnelStats {
        TunnelStats {
            user_tx,
            rx_bytes: 0,
        }
    }

    #[test]
    fn establishing_pings_at_once_and_then_at_the_interval() {
        let start = Instant::now();
        let mut monitor =
            ConnectivityMonitor::new(start, TunnelStats::default(), start + secs(60.0));
        assert_eq!(
            monitor.evaluate_connectivity(start, quiet(0)),
            TunnelStatus::RequiresProbing(1)
        );
        assert_eq!(
            monitor.evaluate_connectivity(start + ESTABLISH_PING_INTERVAL / 2, quiet(0)),
            TunnelStatus::Healthy
        );
        assert_eq!(
            monitor.evaluate_connectivity(start + ESTABLISH_PING_INTERVAL, quiet(0)),
            TunnelStatus::RequiresProbing(1)
        );
        // Pings at once after a reset.
        monitor.reset();
        assert_eq!(
            monitor.evaluate_connectivity(start + ESTABLISH_PING_INTERVAL, quiet(0)),
            TunnelStatus::RequiresProbing(1)
        );
        assert_eq!(
            monitor.evaluate_connectivity(start + secs(59.0), quiet(0)),
            TunnelStatus::RequiresProbing(1)
        );
        assert_eq!(
            monitor.evaluate_connectivity(start + secs(60.0), quiet(0)),
            TunnelStatus::TimedOut
        );
    }

    #[test]
    fn establishing_ignores_user_tx() {
        let start = Instant::now();
        let mut monitor =
            ConnectivityMonitor::new(start, TunnelStats::default(), start + secs(60.0));
        assert_eq!(
            monitor.evaluate_connectivity(start, quiet(1)),
            TunnelStatus::RequiresProbing(1)
        );
        assert_eq!(
            monitor.evaluate_connectivity(start + secs(1.0), quiet(2)),
            TunnelStatus::Healthy
        );
        assert_eq!(
            monitor.evaluate_connectivity(start + secs(60.0), quiet(3)),
            TunnelStatus::TimedOut
        );
    }

    #[test]
    fn idle_clock_starts_at_connect() {
        let start = Instant::now();
        let mut monitor =
            ConnectivityMonitor::new(start, TunnelStats::default(), start + secs(60.0));
        let connected = TunnelStats {
            user_tx: 0,
            rx_bytes: 1,
        };
        assert_eq!(
            monitor.evaluate_connectivity(start + secs(50.0), connected),
            TunnelStatus::Connected
        );
        // Short of the earliest idle probe, at 50s plus the idle timeout.
        assert_eq!(
            monitor.evaluate_connectivity(start + secs(69.9), connected),
            TunnelStatus::Healthy
        );
    }

    #[test]
    fn any_ingress_rx_connects() {
        let start = Instant::now();
        let mut monitor =
            ConnectivityMonitor::new(start, TunnelStats::default(), start + secs(60.0));
        let stats = TunnelStats {
            user_tx: 5,
            rx_bytes: 1,
        };
        assert_eq!(
            monitor.evaluate_connectivity(start, stats),
            TunnelStatus::Connected
        );
        // Earlier traffic is not counted.
        assert_eq!(
            monitor.evaluate_connectivity(start + secs(1.0), stats),
            TunnelStatus::Healthy
        );
        // Connected, so the deadline no longer applies.
        assert_ne!(
            monitor.evaluate_connectivity(start + secs(60.0), stats),
            TunnelStatus::TimedOut
        );
    }

    /// Drives a connected [`ConnectivityMonitor`] tick by tick, recording when pings are sent.
    struct Sim {
        monitor: ConnectivityMonitor,
        start: Instant,
        now: Instant,
        tx: u64,
        rx: usize,
        pings: Vec<(Duration, u32)>,
    }

    impl Sim {
        fn new() -> Self {
            let start = Instant::now();
            let rx = 1;
            let connected = TunnelStats {
                user_tx: 0,
                rx_bytes: rx,
            };
            Self {
                monitor: ConnectivityMonitor::new(start, connected, start),
                start,
                now: start,
                tx: 0,
                rx,
                pings: vec![],
            }
        }

        fn elapsed(&self) -> Duration {
            self.now - self.start
        }

        fn tick(&mut self) -> TunnelStatus {
            self.now += self.monitor.interval();
            let stats = TunnelStats {
                user_tx: self.tx,
                rx_bytes: self.rx,
            };
            let action = self.monitor.evaluate_connectivity(self.now, stats);
            if let TunnelStatus::RequiresProbing(n) = action {
                self.pings.push((self.elapsed(), n));
            }
            action
        }

        /// Tick until `until`, returning the first `TimedOut`, if any.
        fn run_until(
            &mut self,
            until: Duration,
            mut each: impl FnMut(&mut Self),
        ) -> Option<Duration> {
            while self.elapsed() < until {
                each(self);
                if self.tick() == TunnelStatus::TimedOut {
                    return Some(self.elapsed());
                }
            }
            None
        }
    }

    /// Answers each ping on the tick after it was sent.
    fn answer_pings(sim: &mut Sim) {
        if sim.pings.last().is_some_and(|(at, _)| sim.elapsed() == *at) {
            sim.rx += 1;
        }
    }

    #[test]
    fn idle_tunnel_is_probed_once() {
        let mut sim = Sim::new();
        assert_eq!(sim.run_until(secs(600.0), answer_pings), None);
        let [(at, count)] = sim.pings[..] else {
            panic!("{:?}", sim.pings);
        };
        assert!(IDLE_TIMEOUT.contains(&at), "{at:?}");
        assert_eq!(count, 2);
    }

    #[test]
    fn idle_tunnel_that_with_failing_probe_times_out() {
        let mut sim = Sim::new();
        let dead = sim.run_until(secs(600.0), |_| {});
        let (probed_at, _) = sim.pings[0];
        assert!(IDLE_TIMEOUT.contains(&probed_at), "{probed_at:?}");
        assert_eq!(dead, Some(probed_at + PROBE_TIMEOUT));
    }

    #[test]
    fn tx_rearms_the_idle_probe() {
        let mut sim = Sim::new();
        let dead = sim.run_until(secs(400.0), |sim| {
            answer_pings(sim);
            // One answered packet at 200s.
            if sim.elapsed() == secs(200.0) {
                sim.tx += 1;
            } else if sim.elapsed() == secs(200.5) {
                sim.rx += 1;
            }
        });
        assert_eq!(dead, None);
        // The reply is seen at 201s, so the tunnel is idle again from then.
        let [(first, 2), (second, 2)] = sim.pings[..] else {
            panic!("{:?}", sim.pings);
        };
        assert!(IDLE_TIMEOUT.contains(&first), "{first:?}");
        assert!(IDLE_TIMEOUT.contains(&(second - secs(201.0))), "{second:?}");
    }

    #[test]
    fn bidirectional_traffic_is_never_probed() {
        let mut sim = Sim::new();
        let dead = sim.run_until(secs(300.0), |sim| {
            sim.tx += 1;
            sim.rx += 100;
        });
        assert_eq!(dead, None);
        assert!(sim.pings.is_empty());
    }

    #[test]
    fn unanswered_traffic_is_probed_then_fails() {
        let mut sim = Sim::new();
        let dead = sim.run_until(secs(60.0), |sim| sim.tx += 1);

        // tx is first seen at 0.5s, so the probe starts at 7.5s.
        assert_eq!(
            sim.pings,
            vec![
                (secs(7.5), 2),
                (secs(8.5), 3),
                (secs(10.5), 1),
                (secs(12.5), 1),
                (secs(14.5), 1),
                (secs(16.5), 1),
                (secs(18.5), 1),
                (secs(20.5), 1),
            ]
        );
        assert_eq!(dead, Some(secs(22.5)));
    }

    #[test]
    fn single_unanswered_packet_fails_tunnel() {
        let mut sim = Sim::new();
        sim.tx = 1;
        assert_eq!(sim.run_until(secs(60.0), |_| {}), Some(secs(22.5)));
    }

    #[test]
    fn rx_before_the_probe_starts_cancels_probe() {
        let mut sim = Sim::new();
        // Short of the earliest idle probe, at 5.5s plus the idle timeout.
        let dead = sim.run_until(secs(25.0), |sim| {
            if sim.elapsed() == secs(0.0) {
                sim.tx += 1;
            } else if sim.elapsed() == secs(5.0) {
                sim.rx += 1;
            }
        });
        assert_eq!(dead, None);
        assert!(sim.pings.is_empty());
    }

    #[test]
    fn rx_during_probe_cancels_probe() {
        let mut sim = Sim::new();
        // Short of the earliest idle probe, at 15.5s plus the idle timeout.
        let dead = sim.run_until(secs(35.0), |sim| {
            if sim.elapsed() < secs(15.0) {
                sim.tx += 1;
            } else if sim.elapsed() == secs(15.0) {
                sim.rx += 1;
            }
        });
        assert_eq!(dead, None);
        assert!(sim.pings.iter().all(|(at, _)| *at < secs(15.0)));
    }

    #[test]
    fn one_way_traffic_answered_by_pings_probes_once_per_window() {
        let mut sim = Sim::new();
        let dead = sim.run_until(secs(120.0), |sim| {
            // Echo replies arrive on the tick after the request.
            if sim.pings.last().is_some_and(|(at, _)| sim.elapsed() == *at) {
                sim.rx += 1;
            }
            sim.tx += 1;
        });
        assert_eq!(dead, None);
        assert!(sim.pings.iter().all(|(_, n)| *n == 2));
        let gaps: Vec<_> = sim.pings.windows(2).map(|w| w[1].0 - w[0].0).collect();
        assert!(
            gaps.iter().all(|gap| *gap >= UNANSWERED_TX_TIMEOUT),
            "{gaps:?}"
        );
    }

    #[test]
    fn reset_restarts_the_timer() {
        let mut sim = Sim::new();
        let dead = sim.run_until(secs(60.0), |sim| {
            sim.tx += 1;
            if sim.elapsed() == secs(20.0) {
                sim.monitor.reset();
            }
        });
        // The reset discards the probe; tx at 20.5s starts a new 22s window.
        assert_eq!(dead, Some(secs(42.5)));
    }

    #[test]
    fn survives_twenty_percent_loss() {
        // Seeded, so the outcome is deterministic.
        let mut rng = StdRng::seed_from_u64(0x2545_f491_4f6c_dd1d);
        let mut lost = move || rng.random_bool(0.2);

        for _ in 0..1000 {
            let mut sim = Sim::new();
            let mut pings_seen = 0;
            let dead = sim.run_until(secs(120.0), |sim| {
                sim.tx += 1;
                let sent: u32 = sim.pings.iter().map(|(_, n)| n).sum();
                for _ in pings_seen..sent {
                    // The request and the reply may each be lost.
                    if !lost() && !lost() {
                        sim.rx += 1;
                    }
                }
                pings_seen = sent;
            });
            assert_eq!(dead, None);
        }
    }

    #[test]
    fn pings_due_schedule() {
        assert_eq!(pings_due(secs(0.0)), 2);
        assert_eq!(pings_due(secs(0.9)), 2);
        assert_eq!(pings_due(secs(1.0)), 5);
        assert_eq!(pings_due(secs(2.9)), 5);
        assert_eq!(pings_due(secs(3.0)), 6);
        assert_eq!(pings_due(secs(5.0)), 7);
        assert_eq!(pings_due(secs(14.9)), 11);
    }
}

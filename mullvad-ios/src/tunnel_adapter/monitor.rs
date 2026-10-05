//! Connectivity state machine for an established tunnel.
//!
//! User traffic sent into the tunnel without any traffic coming back starts a timer. If nothing
//! is received for [`UNANSWERED_TX_TIMEOUT`], the tunnel is probed with ICMP echo requests, and
//! declared dead if nothing is received for a further [`PROBE_TIMEOUT`]. An idle tunnel is never
//! probed.

use std::time::{Duration, Instant};

/// How long user traffic may go unanswered before the tunnel is probed.
const UNANSWERED_TX_TIMEOUT: Duration = Duration::from_secs(15);
/// How long a probe waits for any rx before the tunnel is considered dead.
const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
/// Echo requests sent at fixed offsets from the probe start, so a single lost packet does not
/// fail the tunnel.
const PROBE_BURSTS: [(Duration, u32); 2] = [(Duration::ZERO, 2), (Duration::from_secs(1), 3)];
/// Interval between the single echo requests that follow the bursts.
const PROBE_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    None,
    /// Send this many echo requests.
    Ping(u32),
    Dead,
}

/// Tracks user tx packets against tunnel rx bytes and decides when to probe or give up.
pub struct ConnMonitor {
    tx_packets: u64,
    rx_bytes: usize,
    /// When user traffic was first sent after the last rx.
    first_unanswered_tx: Option<Instant>,
    /// Echo requests sent for the current probe.
    pings_sent: u32,
}

impl ConnMonitor {
    pub fn new(tx_packets: u64, rx_bytes: usize) -> Self {
        Self {
            tx_packets,
            rx_bytes,
            first_unanswered_tx: None,
            pings_sent: 0,
        }
    }

    /// Forget unanswered traffic and any probe in progress, e.g. after a suspension.
    pub fn reset(&mut self) {
        self.first_unanswered_tx = None;
        self.pings_sent = 0;
    }

    pub fn tick(&mut self, now: Instant, tx_packets: u64, rx_bytes: usize) -> Action {
        if rx_bytes != self.rx_bytes {
            self.rx_bytes = rx_bytes;
            self.reset();
        }
        if tx_packets != self.tx_packets {
            self.tx_packets = tx_packets;
            self.first_unanswered_tx.get_or_insert(now);
        }

        let Some(first_unanswered_tx) = self.first_unanswered_tx else {
            return Action::None;
        };
        let Some(probing_for) =
            now.checked_duration_since(first_unanswered_tx + UNANSWERED_TX_TIMEOUT)
        else {
            return Action::None;
        };
        if probing_for >= PROBE_TIMEOUT {
            return Action::Dead;
        }

        let due = pings_due(probing_for);
        let to_send = due - self.pings_sent;
        self.pings_sent = due;
        if to_send == 0 {
            Action::None
        } else {
            Action::Ping(to_send)
        }
    }
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
            (since_first.as_millis() / PROBE_INTERVAL.as_millis()) as u32 + 1
        });
    bursts + singles
}

#[cfg(test)]
mod tests {
    use super::*;

    const TICK: Duration = Duration::from_millis(500);

    /// Drives a [`ConnMonitor`] tick by tick, recording when pings are sent.
    struct Sim {
        monitor: ConnMonitor,
        start: Instant,
        now: Instant,
        tx: u64,
        rx: usize,
        pings: Vec<(Duration, u32)>,
    }

    impl Sim {
        fn new() -> Self {
            let start = Instant::now();
            Self {
                monitor: ConnMonitor::new(0, 0),
                start,
                now: start,
                tx: 0,
                rx: 0,
                pings: vec![],
            }
        }

        fn elapsed(&self) -> Duration {
            self.now - self.start
        }

        fn tick(&mut self) -> Action {
            self.now += TICK;
            let action = self.monitor.tick(self.now, self.tx, self.rx);
            if let Action::Ping(n) = action {
                self.pings.push((self.elapsed(), n));
            }
            action
        }

        /// Tick until `until`, returning the first `Dead`, if any.
        fn run_until(
            &mut self,
            until: Duration,
            mut each: impl FnMut(&mut Self),
        ) -> Option<Duration> {
            while self.elapsed() < until {
                each(self);
                if self.tick() == Action::Dead {
                    return Some(self.elapsed());
                }
            }
            None
        }
    }

    fn secs(s: f32) -> Duration {
        Duration::from_secs_f32(s)
    }

    #[test]
    fn idle_tunnel_is_never_probed() {
        let mut sim = Sim::new();
        assert_eq!(sim.run_until(secs(300.0), |_| {}), None);
        assert!(sim.pings.is_empty());
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

        // tx is first seen at 0.5s, so the probe starts at 15.5s.
        assert_eq!(
            sim.pings,
            vec![
                (secs(15.5), 2),
                (secs(16.5), 3),
                (secs(18.5), 1),
                (secs(20.5), 1),
                (secs(22.5), 1),
                (secs(24.5), 1),
                (secs(26.5), 1),
                (secs(28.5), 1),
            ]
        );
        assert_eq!(dead, Some(secs(30.5)));
    }

    #[test]
    fn single_unanswered_packet_fails_tunnel() {
        let mut sim = Sim::new();
        sim.tx = 1;
        assert_eq!(sim.run_until(secs(60.0), |_| {}), Some(secs(30.5)));
    }

    #[test]
    fn rx_during_probe_cancels_it() {
        let mut sim = Sim::new();
        let dead = sim.run_until(secs(60.0), |sim| {
            if sim.elapsed() < secs(25.0) {
                sim.tx += 1;
            } else if sim.elapsed() == secs(25.0) {
                sim.rx += 1;
            }
        });
        assert_eq!(dead, None);
        assert!(sim.pings.iter().all(|(at, _)| *at < secs(25.0)));
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
        // The reset discards the probe; tx at 20.5s starts a new 30s window.
        assert_eq!(dead, Some(secs(50.5)));
    }

    #[test]
    fn survives_twenty_percent_loss() {
        // xorshift, so the outcome is deterministic.
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut lost = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed.is_multiple_of(5)
        };

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

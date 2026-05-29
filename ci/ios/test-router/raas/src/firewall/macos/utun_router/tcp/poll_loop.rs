//! The async driver that owns the wall clock and services [`SmoltcpStack`] on every wakeup.

use bytes::Bytes;
use smoltcp::{time::Instant as SmoltcpInstant, wire::Ipv4Packet};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Notify, mpsc};

use super::{BLOCKED_INGRESS_RETRY, stack::SmoltcpStack, upstream_socket::UpstreamMsg};

pub async fn run(
    mut stack: SmoltcpStack,
    mut packet_rx: mpsc::Receiver<Ipv4Packet<Bytes>>,
    mut event_rx: mpsc::Receiver<UpstreamMsg>,
    notify: Arc<Notify>,
) {
    let reference = Instant::now();
    let now = || smoltcp_now(&reference);

    loop {
        let deadline = wakeup_deadline(&mut stack, now());
        tokio::select! {
            biased;
            Some(msg) = event_rx.recv() => stack.handle_upstream_event(msg),
            packet = packet_rx.recv() => match packet {
                Some(packet) => stack.handle_downstream_packet(packet),
                None => {
                    log::debug!("The tunnel device stopped producing packets, stopping TCP router");
                    return;
                }
            },
            _ = notify.notified() => {}
            _ = sleep_until(deadline) => {}
        }

        // Drain whatever else has already queued up, so a burst costs one servicing pass instead
        // of one per item.
        while let Ok(msg) = event_rx.try_recv() {
            stack.handle_upstream_event(msg);
        }
        while let Ok(packet) = packet_rx.try_recv() {
            stack.handle_downstream_packet(packet);
        }

        stack.pump();
        stack.poll(now());
        stack.reap();
    }
}

fn smoltcp_now(reference: &Instant) -> SmoltcpInstant {
    SmoltcpInstant::from_millis(reference.elapsed().as_millis() as i64)
}

/// When the loop must wake up even if nothing else happens.
///
/// Ingress smoltcp could not accept needs retrying even though no smoltcp timer is pending. The
/// tunnel writer also notifies us, so this is only a backstop.
fn wakeup_deadline(stack: &mut SmoltcpStack, now: SmoltcpInstant) -> Option<Duration> {
    let deadline = stack.poll_delay(now);
    if !stack.has_queued_rx() {
        return deadline;
    }

    Some(deadline.unwrap_or(BLOCKED_INGRESS_RETRY).min(BLOCKED_INGRESS_RETRY))
}

/// Sleep until the deadline, or forever if there is none. An idle stack needs no timer wakeups.
async fn sleep_until(deadline: Option<Duration>) {
    match deadline {
        Some(delay) => tokio::time::sleep(delay).await,
        None => std::future::pending().await,
    }
}

use std::{io, net::IpAddr, time::Duration};

use futures::{future, stream::FuturesUnordered, Future, TryStreamExt};
use surge_ping::{Client, Config, PingIdentifier, PingSequence, SurgeError};
use talpid_tunnel::{ICMP_HEADER_SIZE, IPV4_HEADER_SIZE, MIN_IPV4_MTU};
use tokio_stream::StreamExt;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to set MTU on the active tunnel
    #[error("Failed to set MTU on the active tunnel")]
    SetMtu(#[source] io::Error),

    /// Failed to detect MTU because every ping was dropped
    #[error("Failed to detect MTU because all pings timed out.")]
    MtuDetectionAllDropped,

    /// Failed to detect MTU because of unexpected ping error
    #[error("Failed to detect MTU because of unexpected ping error.")]
    MtuDetectionUnexpected(#[source] surge_ping::SurgeError),

    /// Failed to detect MTU because of an IO error when setting up the ping socket
    #[error("Failed to detect MTU because of an IO error when setting up the ping socket.")]
    MtuDetectionSetupSocket(#[source] io::Error),

    /// Failed to set buffer size
    #[cfg(target_os = "macos")]
    #[error("Failed to set buffer size")]
    MtuSetBufferSize(#[source] nix::Error),
}

/// Max time to wait for any ping, when this expires, we give up and throw an error.
const PING_TIMEOUT: Duration = Duration::from_secs(10);
/// Max time to wait after the first ping arrives. Every ping after this timeout is
/// considered dropped, so we return the largest collected packet size.
const PING_OFFSET_TIMEOUT: Duration = Duration::from_secs(2);
const MTU_STEP_SIZE: u16 = 20;

/// Verify that the current MTU doesn't cause dropped packets, otherwise lower it to the
/// largest value which doesn't.
///
/// Note: This does not take fragmentation into account, so it should only be used as an extra
/// safety measure after the normal MTU calculation using header sizes and safety margins.
pub async fn automatic_mtu_correction(
    gateway: std::net::Ipv4Addr,
    iface_name: String,
    current_tunnel_mtu: u16,
    #[cfg(windows)] ipv6: bool,
) -> Result<(), Error> {
    log::debug!("Starting MTU detection");
    let verified_mtu = detect_mtu(
        gateway,
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        iface_name.clone(),
        current_tunnel_mtu,
    )
    .await?;

    if verified_mtu != current_tunnel_mtu {
        log::warn!("Lowering MTU from {} to {verified_mtu}", current_tunnel_mtu);

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        crate::unix::set_mtu(&iface_name, verified_mtu).map_err(Error::SetMtu)?;
        #[cfg(windows)]
        set_mtu_windows(verified_mtu, iface_name, ipv6).map_err(Error::SetMtu)?;
    } else {
        log::debug!("MTU {verified_mtu} verified to not drop packets");
    };
    Ok(())
}

#[cfg(windows)]
fn set_mtu_windows(verified_mtu: u16, iface_name: String, ipv6: bool) -> io::Result<()> {
    use talpid_windows::net::{set_mtu, AddressFamily};

    let luid = talpid_windows::net::luid_from_alias(iface_name)?;
    set_mtu(u32::from(verified_mtu), luid, AddressFamily::Ipv4)?;
    if ipv6 {
        let clamped_mtu = if verified_mtu < talpid_tunnel::MIN_IPV6_MTU {
            log::warn!("Cannot set MTU to {verified_mtu} for IPv6, setting to the minimum value 1280 instead");
            talpid_tunnel::MIN_IPV6_MTU
        } else {
            verified_mtu
        };
        set_mtu(u32::from(clamped_mtu), luid, AddressFamily::Ipv6)?;
    }
    Ok(())
}

/// Detects the maximum MTU that does not cause dropped packets.
///
/// The detection works by sending evenly spread out range of pings between 576 and the given
/// current tunnel MTU, and returning the maximum packet size that was returned within a
/// timeout.
async fn detect_mtu(
    gateway: std::net::Ipv4Addr,
    #[cfg(any(target_os = "macos", target_os = "linux"))] iface_name: String,
    current_mtu: u16,
) -> Result<u16, Error> {
    let linspace = mtu_spacing(MIN_IPV4_MTU, current_mtu, MTU_STEP_SIZE);

    let config_builder = Config::builder().kind(surge_ping::ICMP::V4);
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    let config_builder = config_builder.interface(&iface_name);
    let client = Client::new(&config_builder.build()).map_err(Error::MtuDetectionSetupSocket)?;

    // For macos, the default socket receive buffer size seems to be too small to handle the
    // data we are sending here. The consequence will be dropped packets causing the MTU
    // detection to set a low value. Here we manually increase this value, which fixes
    // the problem.
    // NOTE: If pings drop on other unix platforms too, then enable this fix for them
    #[cfg(target_os = "macos")]
    {
        use nix::sys::socket::{setsockopt, sockopt};
        let fd = client.get_socket().get_native_sock();
        let buf_size = linspace.iter().map(|sz| usize::from(*sz)).sum();
        setsockopt(fd, sockopt::SndBuf, &buf_size).map_err(Error::MtuSetBufferSize)?;
        setsockopt(fd, sockopt::RcvBuf, &buf_size).map_err(Error::MtuSetBufferSize)?;
    }

    // Shared buffer to reduce allocations
    let payload_buf = vec![0; current_mtu as usize];

    // Send a ping for each MTU in the linspace
    let ping_stream = linspace
        .into_iter()
        .enumerate()
        .map(|(sequence, mtu)| {
            let client = client.clone();
            let payload_size = (mtu - IPV4_HEADER_SIZE - ICMP_HEADER_SIZE) as usize;
            let payload = &payload_buf[0..payload_size];
            // Return a future that sends a ping of size MTU, receives the result, and returns the
            // validated MTU
            async move {
                log::trace!("Sending ICMP ping of total size {mtu}");
                let (packet, _duration) = client
                    .pinger(IpAddr::V4(gateway), PingIdentifier(0))
                    .await
                    .timeout(PING_TIMEOUT)
                    .ping(PingSequence(sequence as u16), payload)
                    .await?;

                // Validate the received ping response
                {
                    let surge_ping::IcmpPacket::V4(packet) = packet else {
                        unreachable!("ICMP ping response was not of IPv4 type");
                    };
                    let size = u16::try_from(packet.get_size())
                        .expect("ICMP packet size should fit in u16")
                        + IPV4_HEADER_SIZE;
                    log::trace!("Got ICMP ping response of total size {size}");
                    debug_assert_eq!(
                        size, mtu,
                        "Ping response should be of identical size to request"
                    );
                }
                Ok(mtu)
            }
        })
        .collect::<FuturesUnordered<_>>();

    max_ping_size(ping_stream, PING_OFFSET_TIMEOUT).await
}

/// Consumes a stream of pings, and returns the largest packet size within a given timeout from the
/// first ping response. Short circuits on errors.
async fn max_ping_size(
    mut ping_stream: FuturesUnordered<impl Future<Output = Result<u16, SurgeError>>>,
    ping_offset_timeout: Duration,
) -> Result<u16, Error> {
    let first_ping_size = ping_stream
        .next()
        .await
        .expect("At least one pings should be sent")
        // Short-circuit and return on error
        .map_err(|e| match e {
            // If the first ping we get back timed out, then all of them did
            SurgeError::Timeout { .. } => Error::MtuDetectionAllDropped,
            // Unexpected error type
            e => Error::MtuDetectionUnexpected(e),
        })?;

    ping_stream
        .timeout(ping_offset_timeout) // Start the timeout after the first ping has arrived
        .map_while(|res| res.ok()) // Stop waiting for more pings after this timeout
        .try_fold(first_ping_size, |acc, mtu| future::ready(Ok(acc.max(mtu)))) // Get largest ping
        .await
        .map_err(Error::MtuDetectionUnexpected)
}

/// Creates a linear spacing of MTU values with the given step size. Always includes the given
/// end points.
fn mtu_spacing(mtu_min: u16, mtu_max: u16, step_size: u16) -> Vec<u16> {
    assert!(mtu_min < mtu_max);
    assert!(step_size < mtu_max);
    assert_ne!(step_size, 0);

    let second_mtu = (mtu_min + 1).next_multiple_of(step_size);
    let in_between = (second_mtu..mtu_max).step_by(step_size as usize);

    let mut ret = Vec::with_capacity(in_between.clone().count() + 2);
    ret.push(mtu_min);
    ret.extend(in_between);
    ret.push(mtu_max);
    ret
}

#[cfg(test)]
mod tests {
    use std::{
        marker::{Send, Unpin},
        pin::Pin,
    };

    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn mtu_spacing_properties(mtu_min in 0..800u16, mtu_max in 800..2000u16, step_size in 1..800u16)  {
            let mtu_spacing = mtu_spacing(mtu_min, mtu_max, step_size);

            // The MTU linspace should contain the end points exactly once
            prop_assert_eq!(mtu_spacing.iter().filter(|mtu| mtu == &&mtu_min).count(), 1);
            prop_assert_eq!(mtu_spacing.iter().filter(|mtu| mtu == &&mtu_max).count(), 1);
            // It should be allocated with no wasted capacity
            prop_assert_eq!(mtu_spacing.capacity(), mtu_spacing.len());
            // The spacing should be no greater than step size
            let mut diffs = mtu_spacing.windows(2).map(|win| win[1]-win[0]);
            prop_assert!(diffs.all(|diff| diff <= step_size));

        }
    }

    fn ready_ping<T: Send + 'static>(x: T) -> Pin<Box<dyn Future<Output = T>>> {
        Box::pin(future::ready(x))
    }

    fn ok_ping<T: Send + 'static, E: Send + 'static>(
        x: T,
    ) -> Pin<Box<dyn Future<Output = Result<T, E>>>> {
        ready_ping(Ok(x))
    }

    fn err_ping<T: Send + 'static, E: Send + 'static>(
        e: E,
    ) -> Pin<Box<dyn Future<Output = Result<T, E>>>> {
        ready_ping(Err(e))
    }

    fn delayed_ping<T: Send + 'static + Unpin>(
        x: T,
        duration: Duration,
    ) -> Pin<Box<dyn Future<Output = T>>> {
        Box::pin(async move {
            tokio::time::sleep(duration).await;
            x
        })
    }

    /// The largest ping size should be chosen if all of them return, regardless of return order.
    #[tokio::test]
    async fn all_pings_ok() {
        let pings = (0..=100).rev().map(ok_ping).collect();
        let max = max_ping_size(pings, Duration::from_millis(10))
            .await
            .unwrap();
        assert_eq!(max, 100);
    }

    /// If one ping times out, all the following are considered timed out too. The largest response
    /// before that point is chosen.
    #[tokio::test]
    async fn ping_timeout() {
        let mut pings = FuturesUnordered::new();
        let early_pings = (0..=50).map(ok_ping);
        pings.extend(early_pings);
        let late_pings = (51..=100).map(|p| delayed_ping(Ok(p), Duration::from_millis(10)));
        pings.extend(late_pings);

        let max = max_ping_size(pings, Duration::from_millis(5))
            .await
            .unwrap();
        assert_eq!(max, 50);
    }

    /// The [`PING_OFFSET_TIMEOUT`] is counted from the return of the first ping, not from the
    /// function call.
    #[tokio::test]
    async fn delay_first_ping() {
        let pings = (0..=100)
            .map(|p| delayed_ping(Ok(p), Duration::from_millis(10)))
            .collect();
        let max = max_ping_size(pings, Duration::from_millis(5))
            .await
            .unwrap();
        assert_eq!(max, 100);
    }

    /// If an unknown error type occurs, the MTU detection is aborted and that error is propagated,
    /// even if some ping response came back ok.
    #[tokio::test]
    async fn unknown_error() {
        let pings = FuturesUnordered::new();
        pings.push(ok_ping(0));
        pings.push(err_ping(SurgeError::NetworkError));
        pings.push(ok_ping(10));

        let e = max_ping_size(pings, Duration::from_millis(10))
            .await
            .unwrap_err();
        assert!(matches!(
            e,
            Error::MtuDetectionUnexpected(SurgeError::NetworkError)
        ));
    }

    /// An error of type [`SurgeError::Timeout`] signals that the total [`PING_TIMEOUT`] has been
    /// reached. If this happens to the first ping we consider alls pings timed out.
    #[tokio::test]
    async fn all_dropped() {
        let pings = FuturesUnordered::new();
        pings.push(err_ping(SurgeError::Timeout {
            seq: PingSequence(0),
        }));
        pings.push(delayed_ping(Ok(10), Duration::from_millis(10)));

        let e = max_ping_size(pings, Duration::from_millis(10))
            .await
            .unwrap_err();
        assert!(matches!(e, Error::MtuDetectionAllDropped));
    }

    /// In the rare case that [`PING_TIMEOUT`] triggers before [`PING_OFFSET_TIMEOUT`], even though
    /// some of the ping responses have come back, we still consider it abnormal and choose to
    /// return an error instead of trusting result.
    #[tokio::test]
    async fn max_timeout_error() {
        let pings = FuturesUnordered::new();
        pings.push(delayed_ping(Ok(0), Duration::from_millis(9)));
        pings.push(delayed_ping(
            Err(SurgeError::Timeout {
                seq: PingSequence(0),
            }),
            Duration::from_millis(10),
        ));

        let e = max_ping_size(pings, Duration::from_millis(5))
            .await
            .unwrap_err();
        assert!(matches!(
            e,
            Error::MtuDetectionUnexpected(SurgeError::Timeout { seq: _ })
        ));
    }
}

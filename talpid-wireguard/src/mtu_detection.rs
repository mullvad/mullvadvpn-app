use std::{io, net::IpAddr, time::Duration};

use futures::{future, stream::FuturesUnordered, TryStreamExt};
use surge_ping::{Client, Config, PingIdentifier, PingSequence, SurgeError};
use talpid_tunnel::{ICMP_HEADER_SIZE, IPV4_HEADER_SIZE, MIN_IPV4_MTU};
use tokio_stream::StreamExt;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to set MTU on the active tunnel
    #[error(display = "Failed to set MTU on the active tunnel")]
    SetMtu(#[error(source)] io::Error),

    /// Failed to set MTU
    #[error(display = "Failed to detect MTU because every ping was dropped.")]
    MtuDetectionAllDropped,

    /// Failed to set MTU
    #[error(display = "Failed to detect MTU because of unexpected ping error.")]
    MtuDetectionPing(#[error(source)] surge_ping::SurgeError),

    /// Failed to set MTU
    #[error(
        display = "Failed to detect MTU because of an IO error when setting up the ping socket."
    )]
    MtuDetectionSetupSocket(#[error(source)] io::Error),

    /// Failed to set MTU
    #[cfg(target_os = "macos")]
    #[error(display = "Failed to set buffer size")]
    MtuSetBufferSize(#[error(source)] nix::Error),
}
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
        talpid_windows::net::luid_from_alias(iface_name)
            .and_then(|luid| talpid_windows::net::set_mtu(luid, verified_mtu as u32, ipv6))
            .map_err(Error::SetMtu)?;
    } else {
        log::debug!("MTU {verified_mtu} verified to not drop packets");
    };
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
    /// Max time to wait for any ping, when this expires, we give up and throw an error.
    const PING_TIMEOUT: Duration = Duration::from_secs(10);
    /// Max time to wait after the first ping arrives. Every ping after this timeout is
    /// considered dropped, so we return the largest collected packet size.
    const PING_OFFSET_TIMEOUT: Duration = Duration::from_secs(2);

    let step_size = 20;
    let linspace = mtu_spacing(MIN_IPV4_MTU, current_mtu, step_size);

    let config_builder = Config::builder().kind(surge_ping::ICMP::V4);
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    let config_builder = config_builder.interface(&iface_name);
    let client = Client::new(&config_builder.build()).map_err(Error::MtuDetectionSetupSocket)?;

    // For macos, the default socket receive buffer size seems to be too small to handle the
    // data we are sending here. The consequence will be dropped packets causing the MTU
    // detection to set a low value. Here we manually increase this value, which fixes
    // the problem.
    // TODO: Make sure this fix is not needed for any other target OS
    #[cfg(target_os = "macos")]
    {
        use nix::sys::socket::{setsockopt, sockopt};
        let fd = client.get_socket().get_native_sock();
        let buf_size = linspace.iter().map(|sz| usize::from(*sz)).sum();
        setsockopt(fd, sockopt::SndBuf, &buf_size).map_err(Error::MtuSetBufferSize)?;
        setsockopt(fd, sockopt::RcvBuf, &buf_size).map_err(Error::MtuSetBufferSize)?;
    }

    let payload_buf = vec![0; current_mtu as usize];

    let mut ping_stream = linspace
        .iter()
        .enumerate()
        .map(|(i, &mtu)| {
            let client = client.clone();
            let payload_size = (mtu - IPV4_HEADER_SIZE - ICMP_HEADER_SIZE) as usize;
            let payload = &payload_buf[0..payload_size];
            async move {
                log::trace!("Sending ICMP ping of total size {mtu}");
                client
                    .pinger(IpAddr::V4(gateway), PingIdentifier(0))
                    .await
                    .timeout(PING_TIMEOUT)
                    .ping(PingSequence(i as u16), payload)
                    .await
            }
        })
        .collect::<FuturesUnordered<_>>()
        .map_ok(|(packet, _rtt)| {
            let surge_ping::IcmpPacket::V4(packet) = packet else {
                unreachable!("ICMP ping response was not of IPv4 type");
            };
            let size = u16::try_from(packet.get_size()).expect("ICMP packet size should fit in 16")
                + IPV4_HEADER_SIZE;
            log::trace!("Got ICMP ping response of total size {size}");
            debug_assert_eq!(size, linspace[packet.get_sequence().0 as usize]);
            size
        });

    let first_ping_size = ping_stream
        .next()
        .await
        .expect("At least one pings should be sent")
        // Short-circuit and return on error
        .map_err(|e| match e {
            // If the first ping we get back timed out, then all of them did
            SurgeError::Timeout { .. } => Error::MtuDetectionAllDropped,
            // Unexpected error type
            e => Error::MtuDetectionPing(e),
        })?;

    ping_stream
        .timeout(PING_OFFSET_TIMEOUT) // Start a new, shorter, timeout
        .map_while(|res| res.ok()) // Stop waiting for pings after this timeout
        .try_fold(first_ping_size, |acc, mtu| future::ready(Ok(acc.max(mtu)))) // Get largest ping
        .await
        .map_err(Error::MtuDetectionPing)
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
    use super::mtu_spacing;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_mtu_spacing(mtu_min in 0..800u16, mtu_max in 800..2000u16, step_size in 1..800u16)  {
            let mtu_spacing = mtu_spacing(mtu_min, mtu_max, step_size);

            prop_assert_eq!(mtu_spacing.iter().filter(|mtu| mtu == &&mtu_min).count(), 1);
            prop_assert_eq!(mtu_spacing.iter().filter(|mtu| mtu == &&mtu_max).count(), 1);
            prop_assert_eq!(mtu_spacing.capacity(), mtu_spacing.len());
            let mut diffs = mtu_spacing.windows(2).map(|win| win[1]-win[0]);
            prop_assert!(diffs.all(|diff| diff <= step_size));

        }
    }
}

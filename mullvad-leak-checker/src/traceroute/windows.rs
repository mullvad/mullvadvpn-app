use std::{
    mem::MaybeUninit,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ptr::null_mut,
};

use anyhow::{Context, anyhow};
use futures::{FutureExt, StreamExt, select, stream::FuturesUnordered};

use tokio::time::sleep;
use windows_sys::Win32::{
    Foundation::{GetLastError, INVALID_HANDLE_VALUE},
    NetworkManagement::IpHelper::{
        ICMP_ECHO_REPLY, ICMPV6_ECHO_REPLY_LH, IP_GENERAL_FAILURE, IP_OPTION_INFORMATION,
        IP_SUCCESS, IP_TTL_EXPIRED_TRANSIT, Icmp6CreateFile, Icmp6SendEcho2, IcmpCreateFile,
        IcmpSendEcho2Ex,
    },
    Networking::WinSock::{AF_INET6, IN6_ADDR, IN6_ADDR_0, SOCKADDR_IN6, SOCKADDR_IN6_0},
};

use crate::{
    LeakInfo, LeakStatus,
    traceroute::{DEFAULT_TTL_RANGE, LEAK_TIMEOUT, PROBE_INTERVAL, SEND_TIMEOUT, TracerouteOpt},
    util::{Ip, get_interface_ip},
};

pub async fn try_run_leak_test(opt: &TracerouteOpt) -> anyhow::Result<LeakStatus> {
    let ip_version = match opt.destination {
        IpAddr::V4(..) => Ip::v4(),
        IpAddr::V6(..) => Ip::v6(),
    };

    let interface_ip = get_interface_ip(&opt.interface, ip_version)?;

    let mut ping_tasks = FuturesUnordered::new();

    for (i, ttl) in DEFAULT_TTL_RANGE.enumerate() {
        // Don't send all pings at once, wait a bit in between
        // each one to avoid sending more than necessary
        let probe_delay = PROBE_INTERVAL * i as u32;

        ping_tasks.push(async move {
            sleep(probe_delay).await;

            let destination = opt.destination.clone();
            // ping.exe will send ICMP Echo packets to the destination, and since it's running in
            // the kernel it will be able to receive TimeExceeded responses.
            tokio::task::spawn_blocking(move || -> Result<Option<IpAddr>, anyhow::Error> {
                log::debug!("sending probe packet (ttl={ttl})");
                log::trace!("pinging {destination} with interface {interface_ip}");

                let output_err = || anyhow!("Unexpected error while pinging`");
                let options = IP_OPTION_INFORMATION {
                    Ttl: ttl as u8,
                    Tos: 0,
                    Flags: 0,
                    OptionsSize: 0,
                    OptionsData: null_mut(),
                };

                match (destination, interface_ip) {
                    (IpAddr::V4(ipv4_destination_addr), IpAddr::V4(ipv4_source_addr)) => {
                        let reply_size = std::mem::size_of::<ICMP_ECHO_REPLY>();
                        let mut reply_buffer = MaybeUninit::<ICMP_ECHO_REPLY>::uninit();

                        let reply = unsafe {
                            // SAFETY: IcmpCreateFile is always safe to call.
                            let handle = IcmpCreateFile();

                            if handle == INVALID_HANDLE_VALUE {
                                return Err(std::io::Error::last_os_error())
                                    .with_context(output_err);
                            }

                            // SAFETY: handle has been checked for validity.
                            let replies = dbg!(IcmpSendEcho2Ex(
                                handle,
                                null_mut(),
                                None,
                                null_mut(),
                                ipv4_source_addr.to_bits().to_be(),
                                ipv4_destination_addr.to_bits().to_be(),
                                null_mut(),
                                0,
                                &options,
                                reply_buffer.as_mut_ptr() as _,
                                reply_size as _,
                                SEND_TIMEOUT.as_millis() as u32,
                            ));

                            if replies == 0 {
                                let error_code = GetLastError();
                                // For some reason. Windows will return an IP_GENERAL_FAILURE here instead of where it is supposed to in the ICMP_ECHO_REPLY.Status field. IP_GENERAL_FAILURE should mean that the firewall blocked the route.
                                if error_code == IP_GENERAL_FAILURE {
                                    return Ok(None);
                                } else {
                                    return Err(std::io::Error::from_raw_os_error(
                                        error_code as i32,
                                    ))
                                    .with_context(output_err);
                                }
                            }

                            log::trace!("Successful call to IcmpSendEcho2Ex {reply_buffer:?}");

                            // SAFETY: the buffer will be initialized since IcmpSendEcho2Ex returned without an error.
                            reply_buffer.assume_init()
                        };

                        // There are many possible return values that would indicate a possible leak, but these two statuses indicate a definite leak and should occur during at least one of the ping attempts.
                        if reply.Status != IP_SUCCESS || reply.Status != IP_TTL_EXPIRED_TRANSIT {
                            return Ok(None);
                        }

                        Ok(Some(IpAddr::V4(Ipv4Addr::from_bits(reply.Address))))
                    }
                    (IpAddr::V6(ipv6_destination_addr), IpAddr::V6(ipv6_source_addr)) => {
                        let source = SOCKADDR_IN6 {
                            sin6_family: AF_INET6,
                            sin6_port: 0,
                            sin6_flowinfo: 0,
                            sin6_addr: IN6_ADDR {
                                u: IN6_ADDR_0 {
                                    Byte: ipv6_source_addr.octets(),
                                },
                            },
                            Anonymous: SOCKADDR_IN6_0 { sin6_scope_id: 0 },
                            ..Default::default()
                        };

                        let destination = SOCKADDR_IN6 {
                            sin6_family: AF_INET6,
                            sin6_port: 0,
                            sin6_flowinfo: 0,
                            sin6_addr: IN6_ADDR {
                                u: IN6_ADDR_0 {
                                    Byte: ipv6_destination_addr.octets(),
                                },
                            },
                            Anonymous: SOCKADDR_IN6_0 { sin6_scope_id: 0 },
                            ..Default::default()
                        };

                        let reply_size = std::mem::size_of::<ICMPV6_ECHO_REPLY_LH>();
                        let mut reply_buffer = MaybeUninit::<ICMPV6_ECHO_REPLY_LH>::uninit();

                        let reply = unsafe {
                            // SAFETY: Icmp6CreateFile is always safe to call.
                            let handle = Icmp6CreateFile();

                            if handle == INVALID_HANDLE_VALUE {
                                return Err(std::io::Error::last_os_error())
                                    .with_context(output_err);
                            }

                            // SAFETY: handle has been checked for validity.
                            let replies = dbg!(Icmp6SendEcho2(
                                handle,
                                null_mut(),
                                None,
                                null_mut(),
                                &source,
                                &destination,
                                null_mut(),
                                0,
                                &options,
                                reply_buffer.as_mut_ptr() as _,
                                reply_size as _,
                                SEND_TIMEOUT.as_millis() as u32,
                            ));

                            if replies == 0 {
                                let error_code = GetLastError();
                                // For some reason. Windows will return an IP_GENERAL_FAILURE here instead of where it is supposed to in the ICMP_ECHO_REPLY.Status field. IP_GENERAL_FAILURE should mean that the firewall blocked the route.
                                if error_code == IP_GENERAL_FAILURE {
                                    return Ok(None);
                                } else {
                                    return Err(std::io::Error::from_raw_os_error(
                                        error_code as i32,
                                    ))
                                    .with_context(output_err);
                                }
                            }

                            log::trace!("Successful call to Icmp6SendEcho2 {reply_buffer:?}");

                            // SAFETY: the buffer will be initialized since Icmp6SendEcho2 returned without an error.
                            reply_buffer.assume_init()
                        };
                        // There are many possible return values that would indicate a possible leak, but these two statuses indicate a definite leak and should occur during at least one of the ping attempts.
                        if reply.Status != IP_SUCCESS || reply.Status != IP_TTL_EXPIRED_TRANSIT {
                            return Ok(None);
                        }

                        Ok(Some(IpAddr::V6(Ipv6Addr::from_segments(
                            reply.Address.sin6_addr,
                        ))))
                    }
                    _ => Err(anyhow!("Mismatched source and destination ip version")),
                }
            })
            .await?
        });
    }

    let wait_for_first_leak = async move {
        while let Some(result) = ping_tasks.next().await {
            let Some(ip) = result? else { continue };

            return Ok(LeakStatus::LeakDetected(LeakInfo {
                reachable_nodes: vec![ip],
                interface: opt.interface.clone(),
            }));
        }

        anyhow::Ok(LeakStatus::NoLeak)
    };

    select! {
        _ = sleep(LEAK_TIMEOUT).fuse() => Ok(LeakStatus::NoLeak),
        result = wait_for_first_leak.fuse() => result,
    }
}

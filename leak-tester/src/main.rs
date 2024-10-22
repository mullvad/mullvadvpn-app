use std::{
    io::Read,
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use eyre::*;
use match_cfg::match_cfg;
use socket2::{Domain, Protocol, Socket, Type};
use zerocopy::{transmute, FromBytes, IntoBytes};

#[derive(Clone, Copy, Debug, FromBytes, IntoBytes)]
#[repr(C, packed)]
pub struct Ipv4Header {
    version_and_ihl: u8,
    _stuff: [u8; 7],
    ttl: u8,
    protocol: u8,
    header_checksum: [u8; 2],
    source_address: [u8; 4],
    destination_address: [u8; 4],
}

#[derive(Clone, Copy, Debug, FromBytes, IntoBytes)]
#[repr(C, packed)]
pub struct Icmpv4Header {
    icmp_type: u8,
    code: u8,
    checksum: [u8; 2],
}

#[derive(Clone, Copy, Debug, FromBytes, IntoBytes)]
#[repr(C, packed)]
pub struct Packet {
    ip: Ipv4Header,
    icmp: Icmpv4Header,
}

#[derive(Clone, Copy, Debug, FromBytes, IntoBytes)]
#[repr(C, packed)]
pub struct Icmpv4TimeExceededPayload {
    _unused: [u8; 4],
    ip: Ipv4Header,
}

match_cfg! {
    #[cfg(target_os = "windows")] => {
        const INTERFACE: &str = "todo";
    }
    #[cfg(target_os = "linux")] => {
        const INTERFACE: &str = "wlan0";
    }
    #[cfg(target_os = "macos")] => {
        const INTERFACE: &str = "en0";
    }
    _ => {
        compile_error!("unsupported platform");
    }
}

fn main() -> eyre::Result<()> {
    let destination: IpAddr = "45.83.223.209".parse().unwrap();

    let mut listen_socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))
        .wrap_err("Failed to open raw socket")?;

    let mut send_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .wrap_err("Failed to open datagram socket")?;

    bind_socket_to_interface(&mut send_socket)
        .wrap_err("Failed to bind send socket to interface")?;

    send_socket
        .set_write_timeout(Some(Duration::from_secs(5)))
        .wrap_err("Failed to set socket timeout")?;

    std::thread::spawn(move || {
        print_error_and_exit::<()>(|| {
            let mut read_buf = vec![0u8; usize::from(u16::MAX)].into_boxed_slice();
            loop {
                let n = listen_socket.read(&mut read_buf)?;

                let packet = &read_buf[..n];

                let Some((header, payload)) = packet.split_first_chunk::<{ size_of::<Packet>() }>()
                else {
                    continue;
                };

                let header: Packet = transmute!(*header);

                if header.ip.version_and_ihl != 0x45 {
                    println!("ignoring packet with weird ip version");
                    continue;
                }

                if header.icmp.icmp_type != 11 {
                    continue;
                }

                let Some((icmp_time_exceeded, original_payload)) =
                    payload.split_first_chunk::<{ size_of::<Icmpv4TimeExceededPayload>() }>()
                else {
                    continue;
                };

                let icmp_time_exceeded: Icmpv4TimeExceededPayload = transmute!(*icmp_time_exceeded);

                println!();
                println!("Got a Time Exceeded ICMP message");
                println!("icmp header:  {header:?}");
                println!("original ip:  {icmp_time_exceeded:?}");
                println!("original udp: {original_payload:02x?}");
                println!();
            }
        });
    });

    let payload = b"FOOBARBAZBUZhehe";
    for ttl in 1..=5 {
        let port = 33434u16 + ttl;

        println!("setting ttl (ttl={ttl})");
        send_socket
            .set_ttl(ttl.into())
            .wrap_err("Failed to set TTL on socket")?;

        println!("sending packet (ttl={ttl})");
        let address = SocketAddr::from((destination, port));
        send_socket
            .send_to(payload, &address.into())
            .wrap_err("Failed to send packet")?;
    }

    std::thread::sleep(Duration::from_secs(5));

    Ok(())
}

match_cfg! {
    #[cfg(target_os = "windows")] => {
        fn bind_socket_to_interface(socket: &mut Socket) -> eyre::Result<()> {
            use talpid_windows::net::{get_ip_address_for_interface, luid_from_alias, AddressFamily};

            let interface_luid = luid_from_alias(INTERFACE)?;
            let interface_ip = get_ip_address_for_interface(AddressFamily::Ipv4, interface_luid)?
                .ok_or(eyre!("No IP for interface {INTERFACE:?}"))?;
            socket.bind(&SocketAddr::new(interface_ip, 0).into())?;
            Ok(())
        }
    }
    #[cfg(target_os = "linux")] => {
        fn bind_socket_to_interface(socket: &mut Socket) -> eyre::Result<()> {
            socket
                .bind_device(Some(INTERFACE.as_bytes()))
                .wrap_err("Failed to bind socket to interface")?;
            Ok(())
        }
    }
    #[cfg(target_os = "macos")] => {
        fn bind_socket_to_interface(socket: &mut Socket) -> eyre::Result<()> {
            use nix::net::if_::if_nametoindex;
            use std::num::NonZero;

            let interface_index = if_nametoindex(INTERFACE)
                .map_err(eyre::Report::from)
                .and_then(|code| NonZero::new(code).ok_or_eyre("Non-zero error code"))
                .wrap_err("Failed to get interface index")?;

            socket.bind_device_by_index_v4(Some(interface_index));
            Ok(())
        }
    }
}

fn print_error_and_exit<T>(f: impl FnOnce() -> eyre::Result<T>) {
    if let Err(e) = f() {
        eprintln!("{e:#?}");
        std::process::exit(1);
    }
}

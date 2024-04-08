use eyre::{eyre, Context};
use std::{
    io::Write,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use crate::cli::Opt;

pub fn send_tcp(opt: &Opt, destination: SocketAddr) -> eyre::Result<()> {
    let bind_addr: SocketAddr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 0);

    let family = match &destination {
        SocketAddr::V4(_) => socket2::Domain::IPV4,
        SocketAddr::V6(_) => socket2::Domain::IPV6,
    };
    let sock = socket2::Socket::new(family, socket2::Type::STREAM, Some(socket2::Protocol::TCP))
        .wrap_err(eyre!("Failed to create TCP socket"))?;

    eprintln!("Leaking TCP packets to {destination}");

    sock.bind(&socket2::SockAddr::from(bind_addr))
        .wrap_err(eyre!("Failed to bind TCP socket to {bind_addr}"))?;

    let timeout = Duration::from_millis(opt.leak_timeout);
    sock.set_write_timeout(Some(timeout))?;
    sock.set_read_timeout(Some(timeout))?;

    sock.connect_timeout(&socket2::SockAddr::from(destination), timeout)
        .wrap_err(eyre!("Failed to connect to {destination}"))?;

    let mut stream = std::net::TcpStream::from(sock);
    stream
        .write_all(b"hello there")
        .wrap_err(eyre!("Failed to send message to {destination}"))?;

    Ok(())
}

pub fn send_udp(_opt: &Opt, destination: SocketAddr) -> Result<(), eyre::Error> {
    let bind_addr: SocketAddr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 0);

    eprintln!("Leaking UDP packets to {destination}");

    let family = match &destination {
        SocketAddr::V4(_) => socket2::Domain::IPV4,
        SocketAddr::V6(_) => socket2::Domain::IPV6,
    };
    let sock = socket2::Socket::new(family, socket2::Type::DGRAM, Some(socket2::Protocol::UDP))
        .wrap_err("Failed to create UDP socket")?;

    sock.bind(&socket2::SockAddr::from(bind_addr))
        .wrap_err(eyre!("Failed to bind UDP socket to {bind_addr}"))?;

    // log::debug!("Send message from {bind_addr} to {destination}/UDP");

    let std_socket = std::net::UdpSocket::from(sock);
    std_socket
        .send_to(b"Hello there!", destination)
        .wrap_err(eyre!("Failed to send message to {destination}"))?;

    Ok(())
}

pub fn send_ping(opt: &Opt, destination: IpAddr) -> eyre::Result<()> {
    eprintln!("Leaking IMCP packets to {destination}");

    ping::ping(
        destination,
        Some(Duration::from_millis(opt.leak_timeout)),
        None,
        None,
        None,
        None,
    )?;

    Ok(())
}

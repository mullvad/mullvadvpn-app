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
        .write_all(opt.payload.as_bytes())
        .wrap_err(eyre!("Failed to send message to {destination}"))?;

    Ok(())
}

pub fn send_udp(opt: &Opt, destination: SocketAddr) -> Result<(), eyre::Error> {
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

    let std_socket = std::net::UdpSocket::from(sock);
    std_socket
        .send_to(opt.payload.as_bytes(), destination)
        .wrap_err(eyre!("Failed to send message to {destination}"))?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn send_ping(opt: &Opt, destination: IpAddr) -> eyre::Result<()> {
    eprintln!("Leaking ICMP packets to {destination}");

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

#[cfg(target_os = "macos")]
pub fn send_ping(opt: &Opt, destination: IpAddr) -> eyre::Result<()> {
    eprintln!("Leaking ICMP packets to {destination}");

    ping::dgramsock::ping(
        destination,
        Some(Duration::from_millis(opt.leak_timeout)),
        None,
        None,
        None,
        None,
    )?;

    Ok(())
}

// Some Linux distributions don't allow unprivileged users to send ICMP packets.
// We use the ping command (which has capabilities/setuid set) to get around that.
#[cfg(target_os = "linux")]
pub fn send_ping(opt: &Opt, destination: IpAddr) -> eyre::Result<()> {
    eprintln!("Leaking ICMP packets to {destination}");

    let mut cmd = std::process::Command::new("ping");

    // NOTE: Rounding up to nearest second, since some versions don't support fractional
    //       seconds
    let timeout_sec = ((opt.leak_timeout + 1000 - 1) / 1000).to_string();

    cmd.args(["-c", "1", "-W", &timeout_sec, &destination.to_string()]);

    let output = cmd.output().wrap_err(eyre!(
        "Failed to execute ping for destination {destination}"
    ))?;

    if !output.status.success() {
        eprintln!(
            "ping stdout:\n\n{}",
            std::str::from_utf8(&output.stdout).unwrap_or("invalid utf8")
        );
        eprintln!(
            "ping stderr:\n\n{}",
            std::str::from_utf8(&output.stderr).unwrap_or("invalid utf8")
        );

        return Err(eyre!("ping for destination {destination} failed"));
    }

    Ok(())
}

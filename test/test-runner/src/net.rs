use socket2::SockAddr;
#[cfg(target_os = "macos")]
use std::{ffi::CString, num::NonZeroU32};
use std::{
    io::Write,
    net::{IpAddr, SocketAddr},
};

pub async fn send_tcp(
    bind_interface: Option<String>,
    bind_addr: SocketAddr,
    destination: SocketAddr,
) -> Result<(), test_rpc::Error> {
    let family = match &destination {
        SocketAddr::V4(_) => socket2::Domain::IPV4,
        SocketAddr::V6(_) => socket2::Domain::IPV6,
    };
    let sock = socket2::Socket::new(family, socket2::Type::STREAM, Some(socket2::Protocol::TCP))
        .map_err(|error| {
            log::error!("Failed to create TCP socket: {error}");
            test_rpc::Error::SendTcp
        })?;

    if let Some(iface) = bind_interface {
        #[cfg(target_os = "macos")]
        let interface_index = unsafe {
            let name = CString::new(iface).unwrap();
            let index = libc::if_nametoindex(name.as_bytes_with_nul().as_ptr() as _);
            NonZeroU32::new(index).ok_or_else(|| {
                log::error!("Invalid interface index");
                test_rpc::Error::SendTcp
            })?
        };

        #[cfg(target_os = "macos")]
        sock.bind_device_by_index_v4(Some(interface_index))
            .map_err(|error| {
                log::error!("Failed to set IP_BOUND_IF on socket: {error}");
                test_rpc::Error::SendTcp
            })?;

        #[cfg(target_os = "linux")]
        sock.bind_device(Some(iface.as_bytes())).map_err(|error| {
            log::error!("Failed to bind TCP socket to {iface}: {error}");
            test_rpc::Error::SendTcp
        })?;

        #[cfg(windows)]
        log::trace!("Bind interface {iface} is ignored on Windows")
    }

    log::debug!("Connecting from {bind_addr} to {destination}/TCP");

    tokio::task::spawn_blocking(move || {
        sock.bind(&SockAddr::from(bind_addr)).map_err(|error| {
            log::error!("Failed to bind TCP socket to {bind_addr}: {error}");
            test_rpc::Error::SendTcp
        })?;

        sock.connect(&SockAddr::from(destination))
            .map_err(|error| {
                log::error!("Failed to connect to {destination}: {error}");
                test_rpc::Error::SendTcp
            })?;

        let mut stream = std::net::TcpStream::from(sock);
        stream.write_all(b"hello").map_err(|error| {
            log::error!("Failed to send message to {destination}: {error}");
            test_rpc::Error::SendTcp
        })
    })
    .await
    .unwrap()
}

pub async fn send_udp(
    bind_interface: Option<String>,
    bind_addr: SocketAddr,
    destination: SocketAddr,
) -> Result<(), test_rpc::Error> {
    let family = match &destination {
        SocketAddr::V4(_) => socket2::Domain::IPV4,
        SocketAddr::V6(_) => socket2::Domain::IPV6,
    };
    let sock = socket2::Socket::new(family, socket2::Type::DGRAM, Some(socket2::Protocol::UDP))
        .map_err(|error| {
            log::error!("Failed to create UDP socket: {error}");
            test_rpc::Error::SendUdp
        })?;

    if let Some(iface) = bind_interface {
        #[cfg(target_os = "macos")]
        let interface_index = unsafe {
            let name = CString::new(iface).unwrap();
            let index = libc::if_nametoindex(name.as_bytes_with_nul().as_ptr() as _);
            NonZeroU32::new(index).ok_or_else(|| {
                log::error!("Invalid interface index");
                test_rpc::Error::SendUdp
            })?
        };

        #[cfg(target_os = "macos")]
        sock.bind_device_by_index_v4(Some(interface_index))
            .map_err(|error| {
                log::error!("Failed to set IP_BOUND_IF on socket: {error}");
                test_rpc::Error::SendUdp
            })?;

        #[cfg(target_os = "linux")]
        sock.bind_device(Some(iface.as_bytes())).map_err(|error| {
            log::error!("Failed to bind UDP socket to {iface}: {error}");
            test_rpc::Error::SendUdp
        })?;

        #[cfg(windows)]
        log::trace!("Bind interface {iface} is ignored on Windows")
    }

    let _ = tokio::task::spawn_blocking(move || {
        sock.bind(&SockAddr::from(bind_addr)).map_err(|error| {
            log::error!("Failed to bind UDP socket to {bind_addr}: {error}");
            test_rpc::Error::SendUdp
        })?;

        log::debug!("Send message from {bind_addr} to {destination}/UDP");

        let std_socket = std::net::UdpSocket::from(sock);
        std_socket.send_to(b"hello", destination).map_err(|error| {
            log::error!("Failed to send message to {destination}: {error}");
            test_rpc::Error::SendUdp
        })
    })
    .await
    .unwrap()?;

    Ok(())
}

pub async fn send_ping(
    destination: IpAddr,
    interface: Option<&str>,
    size: usize,
) -> Result<(), test_rpc::Error> {
    use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};

    const IPV4_HEADER_SIZE: usize = 20;
    const ICMP_HEADER_SIZE: usize = 8;
    let payload_size = size - IPV4_HEADER_SIZE - ICMP_HEADER_SIZE;
    let payload: &[u8] = &vec![0; payload_size];

    let config = match destination {
        IpAddr::V4(_) => Config::builder(),
        IpAddr::V6(_) => Config::builder().kind(ICMP::V6),
    };
    let config = if let Some(interface) = interface {
        let interface_ip = get_interface_ip(interface)?;
        config.interface(interface).bind((interface_ip, 0).into())
    } else {
        config
    };
    let client = Client::new(&config.build()).map_err(|e| test_rpc::Error::Ping(e.to_string()))?;
    let mut pinger = client
        .pinger(destination, PingIdentifier(rand::random()))
        .await;
    pinger
        .ping(PingSequence(0), payload)
        .await
        .map_err(|e| test_rpc::Error::Ping(e.to_string()))?;

    Ok(())
}

#[cfg(unix)]
pub fn get_interface_ip(interface: &str) -> Result<IpAddr, test_rpc::Error> {
    // TODO: IPv6
    use std::net::Ipv4Addr;

    let addrs = nix::ifaddrs::getifaddrs().map_err(|error| {
        log::error!("Failed to obtain interfaces: {}", error);
        test_rpc::Error::Syscall
    })?;
    for addr in addrs {
        if addr.interface_name == interface {
            if let Some(address) = addr.address {
                if let Some(sockaddr) = address.as_sockaddr_in() {
                    return Ok(IpAddr::V4(Ipv4Addr::from(sockaddr.ip())));
                }
            }
        }
    }

    log::error!("Could not find tunnel interface");
    Err(test_rpc::Error::InterfaceNotFound)
}

#[cfg(target_os = "windows")]
pub fn get_interface_ip(interface: &str) -> Result<IpAddr, test_rpc::Error> {
    // TODO: IPv6

    get_interface_ip_for_family(interface, talpid_windows::net::AddressFamily::Ipv4)
        .map_err(|_error| test_rpc::Error::Syscall)?
        .ok_or(test_rpc::Error::InterfaceNotFound)
}

#[cfg(target_os = "windows")]
fn get_interface_ip_for_family(
    interface: &str,
    family: talpid_windows::net::AddressFamily,
) -> Result<Option<IpAddr>, ()> {
    let luid = talpid_windows::net::luid_from_alias(interface).map_err(|error| {
        log::error!("Failed to obtain interface LUID: {error}");
    })?;
    talpid_windows::net::get_ip_address_for_interface(family, luid).map_err(|error| {
        log::error!("Failed to obtain interface IP: {error}");
    })
}

#[cfg(target_os = "windows")]
pub fn get_default_interface() -> &'static str {
    use once_cell::sync::OnceCell;
    use talpid_platform_metadata::WindowsVersion;

    static WINDOWS_VERSION: OnceCell<WindowsVersion> = OnceCell::new();
    let version = WINDOWS_VERSION
        .get_or_init(|| WindowsVersion::new().expect("failed to obtain Windows version"));

    if version.build_number() >= 22000 {
        // Windows 11
        return "Ethernet";
    }

    "Ethernet Instance 0"
}

#[cfg(target_os = "linux")]
pub fn get_default_interface() -> &'static str {
    "ens3"
}

#[cfg(target_os = "macos")]
pub fn get_default_interface() -> &'static str {
    "en0"
}

#[cfg(target_os = "macos")]
pub fn get_interface_mtu(_interface_name: &str) -> Result<u16, test_rpc::Error> {
    todo!("Implement setting MTU on macOS")
}

#[cfg(target_os = "linux")]
pub fn get_interface_mtu(interface_name: &str) -> Result<u16, test_rpc::Error> {
    use std::os::fd::AsRawFd;

    let sock = socket2::Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP),
    )
    .map_err(|e| test_rpc::Error::Io(e.to_string()))?;

    let mut ifr: libc::ifreq = unsafe { std::mem::zeroed() };
    if interface_name.len() >= ifr.ifr_name.len() {
        panic!("Interface '{interface_name}' name too long")
    }

    unsafe {
        std::ptr::copy_nonoverlapping(
            interface_name.as_ptr() as *const libc::c_char,
            &mut ifr.ifr_name as *mut _,
            interface_name.len(),
        )
    };

    // TODO: define SIOCGIFMTU for macos
    if unsafe { libc::ioctl(sock.as_raw_fd(), libc::SIOCGIFMTU, &mut ifr) } < 0 {
        let e = std::io::Error::last_os_error();

        log::error!("{}", e);
        return Err(test_rpc::Error::Io(e.to_string()));
    }
    Ok(unsafe { ifr.ifr_ifru.ifru_mtu }
        .try_into()
        .expect("MTU should fit in u16"))
}

#[cfg(target_os = "windows")]
pub fn get_interface_mtu(interface: &str) -> Result<u16, test_rpc::Error> {
    let luid = talpid_windows::net::luid_from_alias(interface).map_err(|error| {
        log::error!("Failed to obtain interface LUID: {error}");
        test_rpc::Error::Syscall
    })?;
    talpid_windows::net::get_ip_interface_entry(talpid_windows::net::AddressFamily::Ipv4, &luid)
        .map_err(|_error| test_rpc::Error::InterfaceNotFound)
        .map(|row| row.NlMtu.try_into().unwrap())
}

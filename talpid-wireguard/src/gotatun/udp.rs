// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: MPL-2.0

use std::{future, io, net::SocketAddr};

use gotatun::{
    packet::{Packet, PacketBufPool},
    udp::{
        UdpRecv, UdpSend, UdpTransportFactory, UdpTransportFactoryParams,
        socket::{SockOpt, UdpSocket, UdpSocketFactory},
    },
};

pub(super) struct LinuxUdpSocketFactory {
    inner: UdpSocketFactory,
}

impl LinuxUdpSocketFactory {
    pub const fn new(inner: UdpSocketFactory) -> Self {
        Self { inner }
    }
}

impl UdpTransportFactory for LinuxUdpSocketFactory {
    type RecvV4 = UdpSocket;
    type RecvV6 = MaybeDisabledUdpSocket;
    type SendV4 = UdpSocket;
    type SendV6 = MaybeDisabledUdpSocket;

    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<((Self::SendV4, Self::RecvV4), (Self::SendV6, Self::RecvV6))> {
        match self.inner.bind(params).await {
            Ok(((send_v4, recv_v4), (send_v6, recv_v6))) => Ok((
                (send_v4, recv_v4),
                (
                    MaybeDisabledUdpSocket::Socket(send_v6),
                    MaybeDisabledUdpSocket::Socket(recv_v6),
                ),
            )),
            Err(error) if is_ipv6_unavailable(&error) => {
                log::warn!(
                    "IPv6 UDP sockets are unavailable; continuing with an IPv4-only GotaTun UDP transport"
                );
                let (send_v4, recv_v4) = bind_ipv4_socket(params, &self.inner)?;
                Ok((
                    (send_v4, recv_v4),
                    (
                        MaybeDisabledUdpSocket::Disabled,
                        MaybeDisabledUdpSocket::Disabled,
                    ),
                ))
            }
            Err(error) => Err(error),
        }
    }
}

#[derive(Clone)]
pub(super) enum MaybeDisabledUdpSocket {
    Socket(UdpSocket),
    Disabled,
}

impl UdpSend for MaybeDisabledUdpSocket {
    type SendManyBuf = <UdpSocket as UdpSend>::SendManyBuf;

    async fn send_to(&self, packet: Packet, destination: SocketAddr) -> io::Result<()> {
        match self {
            Self::Socket(socket) => socket.send_to(packet, destination).await,
            Self::Disabled => Err(disabled_ipv6_error()),
        }
    }

    fn max_number_of_packets_to_send(&self) -> usize {
        match self {
            Self::Socket(socket) => socket.max_number_of_packets_to_send(),
            Self::Disabled => 1,
        }
    }

    async fn send_many_to(
        &self,
        send_buf: &mut Self::SendManyBuf,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        match self {
            Self::Socket(socket) => socket.send_many_to(send_buf, packets).await,
            Self::Disabled => Err(disabled_ipv6_error()),
        }
    }

    fn local_addr(&self) -> io::Result<Option<SocketAddr>> {
        match self {
            Self::Socket(socket) => UdpSend::local_addr(socket),
            Self::Disabled => Ok(None),
        }
    }

    #[cfg(target_os = "linux")]
    fn set_fwmark(&self, mark: u32) -> io::Result<()> {
        match self {
            Self::Socket(socket) => socket.set_fwmark(mark),
            Self::Disabled => Ok(()),
        }
    }
}

impl UdpRecv for MaybeDisabledUdpSocket {
    type RecvManyBuf = <UdpSocket as UdpRecv>::RecvManyBuf;

    async fn recv_from(&mut self, pool: &mut PacketBufPool) -> io::Result<(Packet, SocketAddr)> {
        match self {
            Self::Socket(socket) => socket.recv_from(pool).await,
            Self::Disabled => future::pending::<io::Result<(Packet, SocketAddr)>>().await,
        }
    }

    async fn recv_many_from(
        &mut self,
        recv_buf: &mut Self::RecvManyBuf,
        pool: &mut PacketBufPool,
        packets: &mut Vec<(Packet, SocketAddr)>,
    ) -> io::Result<()> {
        match self {
            Self::Socket(socket) => socket.recv_many_from(recv_buf, pool, packets).await,
            Self::Disabled => future::pending::<io::Result<()>>().await,
        }
    }

    fn enable_udp_gro(&self) -> io::Result<()> {
        match self {
            Self::Socket(socket) => socket.enable_udp_gro(),
            Self::Disabled => Ok(()),
        }
    }
}

fn bind_ipv4_socket(
    params: &UdpTransportFactoryParams,
    factory: &UdpSocketFactory,
) -> io::Result<(UdpSocket, UdpSocket)> {
    let opts = SockOpt {
        fwmark: params.fwmark,
        recv_buffer_size: factory.recv_buffer_size,
        send_buffer_size: factory.send_buffer_size,
    };
    let socket = UdpSocket::bind((params.addr_v4, params.port).into(), opts)?;

    if let Err(error) = socket.enable_udp_gro() {
        log::warn!("Failed to enable UDP GRO for IPv4 socket: {error}");
    }

    Ok((socket.clone(), socket))
}

fn is_ipv6_unavailable(error: &io::Error) -> bool {
    matches!(
        error.raw_os_error(),
        Some(libc::EAFNOSUPPORT | libc::EADDRNOTAVAIL)
    )
}

fn disabled_ipv6_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "IPv6 UDP sockets are unavailable",
    )
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::*;

    #[test]
    fn recognizes_unavailable_ipv6_errors() {
        let error = io::Error::from_raw_os_error(libc::EAFNOSUPPORT);
        assert!(is_ipv6_unavailable(&error));

        let error = io::Error::from_raw_os_error(libc::EADDRNOTAVAIL);
        assert!(is_ipv6_unavailable(&error));

        let error = io::Error::from(io::ErrorKind::AddrInUse);
        assert!(!is_ipv6_unavailable(&error));
    }

    #[tokio::test]
    async fn bind_ipv4_socket_uses_one_socket_for_send_and_receive() {
        let params = UdpTransportFactoryParams {
            addr_v4: Ipv4Addr::LOCALHOST,
            addr_v6: Ipv6Addr::LOCALHOST,
            port: 0,
            fwmark: None,
        };

        let (send, receive) = bind_ipv4_socket(&params, &UdpSocketFactory::default()).unwrap();

        let send_addr = UdpSend::local_addr(&send).unwrap().unwrap();
        assert!(send_addr.is_ipv4());
        assert_eq!(send_addr, receive.local_addr().unwrap());
    }

    #[tokio::test]
    async fn disabled_ipv6_send_returns_unsupported() {
        let socket = MaybeDisabledUdpSocket::Disabled;
        let error = socket
            .send_to(Packet::default(), (Ipv6Addr::LOCALHOST, 1).into())
            .await
            .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn disabled_ipv6_socket_options_are_noops() {
        let socket = MaybeDisabledUdpSocket::Disabled;

        assert!(socket.set_fwmark(1).is_ok());
        assert!(socket.enable_udp_gro().is_ok());
    }
}

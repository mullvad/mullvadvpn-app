use ipnetwork::IpNetwork;
use nix::{
    ifaddrs::{getifaddrs, InterfaceAddress},
    sys::socket::{socket, AddressFamily, SockFlag, SockType, SockaddrLike, SockaddrStorage},
};
use std::{
    collections::BTreeMap,
    io::{self, Read},
    mem,
    net::{IpAddr, SocketAddr},
};
use tokio::io::unix::AsyncFd;

use crate::imp::imp::watch::data::RouteSocketAddress;

use self::data::{
    Destination, Interface, MessageType, RouteFlag, RouteMessage, RouteSocketMessage,
};

pub(crate) mod data;
pub(crate) mod routing_socket;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, err_derive::Error)]
pub enum Error {
    #[error(display = "Failed to resolve interface name to index")]
    ResolveInterfaceName(nix::Error),
    #[error(display = "Failed to open routing socket")]
    RoutingSocket(routing_socket::Error),
    #[error(display = "Invalid message")]
    InvalidMessage(data::Error),
    #[error(display = "Failed to send routing message")]
    SendError(routing_socket::Error),
    #[error(display = "Unexpected message type")]
    UnexpectedMessageType(RouteSocketMessage, MessageType),
    #[error(display = "Route not found")]
    RouteNotFound,
    #[error(display = "Destination unreachable")]
    Unreachable,
    #[error(display = "Failed to delete a route")]
    Deletion(RouteMessage),
    #[error(display = "Failed to add a route")]
    Add(RouteMessage),
    #[error(display = "Faield to fetch a route")]
    Get(RouteMessage),
}

/// Provides an interface for manipulating the routing table on macOS using a PF_ROUTE socket.
pub struct RoutingTable {
    socket: routing_socket::RoutingSocket,
}

// TODO: Ensure that route socket messages with error messages in them get returned as a result
impl RoutingTable {
    pub fn new() -> Result<Self> {
        let socket = routing_socket::RoutingSocket::new().map_err(Error::RoutingSocket)?;

        Ok(Self { socket })
    }

    // TODO: add_listener instead? but then self becomes actor
    pub async fn next_message(&mut self) -> Result<RouteSocketMessage> {
        let mut buf = [0u8; 2048];
        let bytes_read = self
            .socket
            .recv_msg(&mut buf)
            .await
            .map_err(Error::RoutingSocket)?;
        let msg_buf = &buf[0..bytes_read];
        data::RouteSocketMessage::parse_message(&msg_buf).map_err(Error::InvalidMessage)
    }

    pub async fn add_route(&mut self, message: &RouteMessage) -> Result<()> {
        let msg = self
            .alter_routing_table(&message, MessageType::RTM_ADD)
            .await;

        match msg {
            Ok(RouteSocketMessage::AddRoute(_route)) => Ok(()),
            Err(Error::SendError(routing_socket::Error::WriteError(err)))
                if err.kind() == io::ErrorKind::AlreadyExists =>
            {
                Ok(())
            }
            // FIXME: probably wrong? but fails in some cases without this, despite route being added. errno 3
            Ok(RouteSocketMessage::GetRoute(_route)) => Ok(()),
            Ok(anything_else) => {
                log::error!("Unexpected route message: {anything_else:?}");
                Err(Error::UnexpectedMessageType(
                    anything_else,
                    MessageType::RTM_ADD,
                ))
            }

            Err(err) => Err(err),
        }
    }

    pub async fn change_route(&mut self, message: &RouteMessage) -> Result<()> {
        let response = self
            .alter_routing_table(message, MessageType::RTM_CHANGE)
            .await?;

        match response {
            RouteSocketMessage::ChangeRoute(_route) => Ok(()),
            anything_else => Err(Error::UnexpectedMessageType(
                anything_else,
                MessageType::RTM_CHANGE,
            )),
        }
    }

    async fn alter_routing_table(
        &mut self,
        message: &RouteMessage,
        message_type: MessageType,
    ) -> Result<RouteSocketMessage> {
        let result = self.socket.send_route_message(&message, message_type).await;

        match result {
            Ok(response) => {
                data::RouteSocketMessage::parse_message(&response).map_err(Error::InvalidMessage)
            }

            Err(routing_socket::Error::WriteError(err))
                if err.kind() == io::ErrorKind::NotFound =>
            {
                Err(Error::RouteNotFound)
            }
            Err(routing_socket::Error::WriteError(err))
                if [Some(libc::ENETUNREACH), Some(libc::ESRCH)].contains(&err.raw_os_error()) =>
            {
                Err(Error::Unreachable)
            }
            Err(err) => Err(Error::SendError(err)),
        }
    }

    pub async fn delete_route(&mut self, message: &RouteMessage) -> Result<()> {
        let response = self
            .alter_routing_table(message, MessageType::RTM_DELETE)
            .await?;

        match response {
            RouteSocketMessage::DeleteRoute(route) if route.errno() == 0 => Ok(()),
            RouteSocketMessage::DeleteRoute(route) if route.errno() != 0 => {
                Err(Error::Deletion(route))
            }
            // FIXME: probably wrong? but fails in some cases without this, despite route being added. errno 3
            RouteSocketMessage::GetRoute(_route) => Ok(()),
            anything_else => Err(Error::UnexpectedMessageType(
                anything_else,
                MessageType::RTM_DELETE,
            )),
        }
    }

    pub async fn get_route(
        &mut self,
        destination: impl Into<Destination>,
    ) -> Result<Option<data::RouteMessage>> {
        let destination = destination.into();

        let mut msg = RouteMessage::new_route(destination);
        if destination.is_network() {
            msg = msg.set_gateway_route(true);
        }

        let response = self
            .socket
            .send_route_message(&msg, MessageType::RTM_GET)
            .await;

        let response = match response {
            Ok(response) => response,
            Err(routing_socket::Error::WriteError(err)) => {
                if let Some(err) = err.raw_os_error() {
                    if [libc::ENETUNREACH, libc::ESRCH].contains(&err) {
                        return Ok(None);
                    }
                }
                return Err(Error::RoutingSocket(routing_socket::Error::WriteError(err)));
            }
            Err(other_err) => {
                return Err(Error::RoutingSocket(other_err));
            }
        };

        match data::RouteSocketMessage::parse_message(&response).map_err(Error::InvalidMessage)? {
            data::RouteSocketMessage::GetRoute(route) => Ok(Some(route)),
            unexpected_route_message => Err(Error::UnexpectedMessageType(
                unexpected_route_message,
                MessageType::RTM_GET,
            )),
        }
    }
}

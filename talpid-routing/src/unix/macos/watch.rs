use super::{
    data::{self, MessageType, RouteMessage, RouteSocketMessage},
    routing_socket,
};
use std::io;

type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur for a PF_ROUTE socket
#[derive(Debug, err_derive::Error)]
pub enum Error {
    /// Generic routing socket error
    #[error(display = "Routing socket error: {}", _0)]
    RoutingSocket(routing_socket::Error),
    /// Failed to parse route message
    #[error(display = "Invalid message")]
    InvalidMessage(data::Error),
    /// Failed to send route message
    #[error(display = "Failed to send routing message")]
    Send(routing_socket::Error),
    /// Received unexpected response to route message
    #[error(display = "Unexpected message type")]
    UnexpectedMessageType(RouteSocketMessage, MessageType),
    /// Route not found
    #[error(display = "Route not found")]
    RouteNotFound,
    /// No route to destination
    #[error(display = "Destination unreachable")]
    Unreachable,
    /// Failed to delete route
    #[error(display = "Failed to delete a route")]
    Deletion(RouteMessage),
}

/// Provides an interface for manipulating the routing table on macOS using a PF_ROUTE socket.
pub struct RoutingTable {
    socket: routing_socket::RoutingSocket,
}

impl RoutingTable {
    pub fn new() -> Result<Self> {
        let socket = routing_socket::RoutingSocket::new().map_err(Error::RoutingSocket)?;

        Ok(Self { socket })
    }

    pub async fn next_message(&mut self) -> Result<RouteSocketMessage> {
        let mut buf = [0u8; 2048];
        let bytes_read = self
            .socket
            .recv_msg(&mut buf)
            .await
            .map_err(Error::RoutingSocket)?;
        let msg_buf = &buf[0..bytes_read];
        data::RouteSocketMessage::parse_message(msg_buf).map_err(Error::InvalidMessage)
    }

    pub async fn add_route(&mut self, message: &RouteMessage) -> Result<()> {
        if let Ok(destination) = message.destination_ip() {
            if Some(destination.ip()) == message.gateway_ip() {
                // Workaround that allows us to reach a wg peer on our router.
                // If we don't do this, adding the route fails due to errno 49
                // ("Can't assign requested address").
                log::warn!("Ignoring route because the destination equals its gateway");
                return Ok(());
            }
        }

        let msg = self
            .alter_routing_table(message, MessageType::RTM_ADD)
            .await;

        match msg {
            Ok(RouteSocketMessage::AddRoute(_route)) => Ok(()),
            Err(Error::Send(routing_socket::Error::Write(err)))
                if err.kind() == io::ErrorKind::AlreadyExists =>
            {
                Ok(())
            }
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

    async fn alter_routing_table(
        &mut self,
        message: &RouteMessage,
        message_type: MessageType,
    ) -> Result<RouteSocketMessage> {
        let result = self.socket.send_route_message(message, message_type).await;

        match result {
            Ok(response) => {
                data::RouteSocketMessage::parse_message(&response).map_err(Error::InvalidMessage)
            }

            Err(routing_socket::Error::Write(err)) if err.kind() == io::ErrorKind::NotFound => {
                Err(Error::RouteNotFound)
            }
            Err(routing_socket::Error::Write(err))
                if [Some(libc::ENETUNREACH), Some(libc::ESRCH)].contains(&err.raw_os_error()) =>
            {
                Err(Error::Unreachable)
            }
            Err(err) => Err(Error::Send(err)),
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
            anything_else => Err(Error::UnexpectedMessageType(
                anything_else,
                MessageType::RTM_DELETE,
            )),
        }
    }

    pub async fn get_route(
        &mut self,
        message: &RouteMessage,
    ) -> Result<Option<data::RouteMessage>> {
        let response = self
            .socket
            .send_route_message(message, MessageType::RTM_GET)
            .await;

        let response = match response {
            Ok(response) => response,
            Err(routing_socket::Error::Write(err)) => {
                if let Some(err) = err.raw_os_error() {
                    if [libc::ENETUNREACH, libc::ESRCH].contains(&err) {
                        return Ok(None);
                    }
                }
                return Err(Error::RoutingSocket(routing_socket::Error::Write(err)));
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

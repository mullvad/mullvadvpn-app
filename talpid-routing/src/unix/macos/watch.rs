use super::{
    data::{self, MessageType, RouteMessage, RouteSocketMessage},
    routing_socket,
};
use std::io;

type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur for a PF_ROUTE socket
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Generic routing socket error
    #[error("Routing socket error")]
    RoutingSocket(#[source] routing_socket::Error),
    /// Failed to parse route message
    #[error("Invalid message")]
    InvalidMessage(#[source] data::Error),
    /// Failed to send route message
    #[error("Failed to send routing message")]
    Send(#[source] routing_socket::Error),
    /// Received unexpected response to route message
    #[error("Unexpected message type")]
    UnexpectedMessageType(RouteSocketMessage, MessageType),
    /// Route not found
    #[error("Route not found")]
    RouteNotFound,
    /// No route to destination
    #[error("Destination unreachable")]
    Unreachable,
    /// Failed to delete route
    #[error("Failed to delete a route")]
    Deletion(RouteMessage),
}

/// Provides an interface for manipulating the routing table on macOS using a PF_ROUTE socket.
pub struct RoutingTable {
    socket: routing_socket::RoutingSocket,
}

/// Result of successfully adding a route
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AddResult {
    /// A new route was created
    Ok,
    /// The route already exists
    AlreadyExists,
}

impl RoutingTable {
    pub fn new() -> Result<Self> {
        let socket = routing_socket::RoutingSocket::new().map_err(Error::RoutingSocket)?;

        Ok(Self { socket })
    }

    pub async fn next_message(&mut self) -> Result<RouteSocketMessage> {
        let mut buf = [0u8; 2048];

        let bytes_read = loop {
            match self.socket.recv_msg(&mut buf).await {
                Ok(bytes_read) => break bytes_read,
                Err(error) if error.is_shutdown() => {
                    log::debug!("Recreating shut down socket");
                    self.socket =
                        routing_socket::RoutingSocket::new().map_err(Error::RoutingSocket)?;
                }
                Err(error) => return Err(Error::RoutingSocket(error)),
            }
        };

        let msg_buf = &buf[0..bytes_read];
        data::RouteSocketMessage::parse_message(msg_buf).map_err(Error::InvalidMessage)
    }

    pub async fn add_route(&mut self, message: &RouteMessage) -> Result<AddResult> {
        if let Ok(destination) = message.destination_ip() {
            if Some(destination.ip()) == message.gateway_ip() {
                // Workaround that allows us to reach a wg peer on our router.
                // If we don't do this, adding the route fails due to errno 49
                // ("Can't assign requested address").
                log::warn!("Ignoring route because the destination equals its gateway");
                return Ok(AddResult::AlreadyExists);
            }
        }

        let msg = self
            .alter_routing_table(message, MessageType::RTM_ADD)
            .await;

        match msg {
            Ok(RouteSocketMessage::AddRoute(_route)) => Ok(AddResult::Ok),
            Err(Error::Send(routing_socket::Error::Write(err)))
                if err.kind() == io::ErrorKind::AlreadyExists =>
            {
                Ok(AddResult::AlreadyExists)
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

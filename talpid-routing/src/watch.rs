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
    #[error(display = "Unexepcted message type")]
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

pub struct RoutingTable {
    socket: routing_socket::RoutingSocket,
}

// TODO: Ensure that route socket messages with error messages in them get returned as a result
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
            Ok(anything_else) => Err(Error::UnexpectedMessageType(
                anything_else,
                MessageType::RTM_ADD,
            )),

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

fn if_sockaddr_for_default_route_fetching() -> SockaddrStorage {
    let interface_sockaddr = nix::libc::sockaddr_dl {
        sdl_len: mem::size_of::<nix::libc::sockaddr_dl>() as u8,
        sdl_family: libc::AF_LINK as u8,
        sdl_index: 0,
        sdl_type: 0,
        sdl_nlen: 0,
        sdl_alen: 0,
        sdl_slen: 0,
        sdl_data: [0; 12],
    };
    unsafe {
        SockaddrStorage::from_raw(
            &interface_sockaddr as *const _ as *const _,
            Some(interface_sockaddr.sdl_len.into()),
        )
    }
    .unwrap()
}

/// Watch routes
pub async fn watch_routes() -> Result<()> {
    // read_from_file(&"add-two-no-slash");
    // read_from_file(&"add-two-slash-28");
    // read_from_file(&"add-two-slash-32");
    read_from_file(&"add-tensixtyfour");
    let mut table = RoutingTable::new().expect("failed to open routing table");
    let default_route_v4 = table.get_route(Destination::default_v4()).await;
    let default_route_v6 = table.get_route(Destination::default_v6()).await;
    let route_to_gw = table
        .get_route(Destination::Host("10.64.0.1".parse().unwrap()))
        .await;

    let dest: IpNetwork = "173.11.33.44/32".parse().unwrap();
    let gateway: IpAddr = "192.168.1.1".parse().unwrap();
    let new_route = RouteMessage::new_route(dest.into()).set_gateway_addr(gateway);

    // let add_route = table
    //     .add_route(
    //         "1.1.1.1/32".parse().unwrap(),
    //         Some("8.8.8.8".parse().unwrap()),
    //         None,
    //         false,
    //     )
    //     .await;
    // let _ = std::io::stdin().read(&mut [0u8; 1]);
    //

    // let remove_route = table
    //     .delete_route(
    //         "1.1.1.1/32".parse().unwrap(),
    //         None,
    //         false,
    //     )
    //     .await;

    //     let interface = nix::ifaddrs::getifaddrs()
    //         .unwrap()
    //         .find(|iface| iface.interface_name == "en0")
    //         .unwrap();

    //     let new_route = RouteMessage::new_route(Destination::Host("1.1.1.1".parse().unwrap()))
    //         .set_gateway_addr("192.168.88.1".parse().unwrap());

    // let add = table.add_route(&new_route);

    //     .delete_route(
    //         "1.1.1.1/32".parse().unwrap(),
    //         // Some("192.168.88.1".parse().unwrap()),
    //         None,
    //         Some(&interface),
    //         false,
    //     )
    //     .await;
    // println!("delet {delet:?}");
    // println!("add_route - {add_route:?}\nremove_route {remove_route:?}");

    loop {
        let msg = table.next_message().await?;
        print_msg(msg);
    }
    Ok(())
}

fn read_from_file(path: impl AsRef<std::path::Path>) {
    match std::fs::read(path.as_ref()) {
        Ok(bytes) => {
            let msg = data::RouteSocketMessage::parse_message(&bytes).unwrap();
            println!("{} contains {msg:?}", path.as_ref().display());
        }
        Err(err) => {
            println!("Failed to read file {}:{err}", path.as_ref().display());
        }
    }
}

fn print_msg(msg: data::RouteSocketMessage) {
    match msg {
        data::RouteSocketMessage::GetRoute(route) => {
            println!(
                "================================================================================"
            );
            println!("get route");
            route.print_route();
            println!("");
        }
        data::RouteSocketMessage::AddRoute(route) => {
            println!(
                "================================================================================"
            );
            println!("add route");
            route.print_route();
            println!("");
        }
        data::RouteSocketMessage::ChangeRoute(route) => {
            println!(
                "================================================================================"
            );
            println!("change route");
            route.print_route();
            println!("");
        }
        data::RouteSocketMessage::DeleteRoute(route) => {
            println!(
                "================================================================================"
            );
            let addrs = route.route_addrs().collect::<Vec<_>>();
            println!("route-addrs = {}", addrs.len());
            route.print_route();
            println!("");
        }

        data::RouteSocketMessage::Interface(interface) => {
            println!(
                "================================================================================"
            );
            let action_msg = if interface.is_up() {
                "added"
            } else {
                "removed"
            };
            let idx = interface.index();
            println!("{action_msg} interface {idx}");
        }

        data::RouteSocketMessage::AddAddress(address) => {
            println!(
                "================================================================================"
            );
            let idx = address.index();
            println!("Added address {:?} for interface {idx}", address.address());
            address.print_sockaddrs();
        }

        data::RouteSocketMessage::DeleteAddress(address) => {
            println!(
                "================================================================================"
            );
            let idx = address.index();
            let address = address.address();
            println!("Deleted address {address:?} for interface {idx}");
        }
        // ignoring other kinds of route messages
        _ => {
            return;
        }
    };
}

const REMOVE_ROUTE_MSG: &[u8] = &[
    164, 0, 5, 2, 11, 0, 0, 0, 66, 8, 1, 67, 55, 0, 0, 0, 64, 1, 0, 0, 71, 8, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 220, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    16, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 2, 0, 0, 192, 168, 185, 1, 11, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 20, 18, 11, 0, 6, 3, 6, 0, 101, 110, 48, 60, 6, 48, 3, 54, 249, 0, 0, 0,
    16, 2, 0, 0, 192, 168, 185, 116, 0, 0, 0, 0, 0, 0, 0, 0,
];

const GET_ROUTE_MSG: &'static str = "qAAFBBIAAABBCQBANwAAAAcPAAABAAAAAAAAAAAAAAAAAAAAAAAAAIwFAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAgAAAAAAAAAAAAAAAAAAFBISAAAAAAAAAAAAAAAAAAAAAAAAAgAAFBISAAEFAAB1dHVuMwAAAAAAAAAQAgAACnYA7wAAAAAAAAAA";

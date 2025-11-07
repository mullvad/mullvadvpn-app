use super::{Config, Tunnel, TunnelError};
use futures::future::{AbortHandle, abortable};
use netlink_packet_core::DecodeError;
use netlink_packet_core::NLM_F_ACK;
use netlink_packet_core::NLM_F_CREATE;
use netlink_packet_core::NLM_F_DUMP;
use netlink_packet_core::NLM_F_MATCH;
use netlink_packet_core::NLM_F_REPLACE;
use netlink_packet_core::NLM_F_REQUEST;
use netlink_packet_core::NetlinkDeserializable;
use netlink_packet_core::NetlinkMessage;
use netlink_packet_core::NetlinkPayload;
use netlink_packet_route::RouteNetlinkMessage;
use netlink_packet_route::address::AddressMessage;
use netlink_packet_route::link::LinkMessage;
use netlink_proto::sys::{SocketAddr, protocols::NETLINK_GENERIC};
use netlink_proto::{ConnectionHandle, Error as NetlinkError};
use rtnetlink::AddressMessageBuilder;
use rtnetlink::LinkMessageBuilder;
use rtnetlink::LinkWireguard;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use tokio_stream::StreamExt;

mod parsers;
mod stats;
mod timespec;

pub mod wg_message;
use wg_message::{DeviceMessage, DeviceNla};
pub mod nl_message;
use nl_message::{ControlNla, NetlinkControlMessage};

pub mod netlink_tunnel;
pub use netlink_tunnel::NetlinkTunnel;
pub mod nm_tunnel;
pub use nm_tunnel::NetworkManagerTunnel;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to decode netlink message")]
    Decode(#[source] DecodeError),

    #[error("Failed to execute netlink control request")]
    NetlinkControlMessage(#[source] nl_message::Error),

    #[error("Failed to open netlink socket")]
    NetlinkSocket(#[source] std::io::Error),

    #[error("Failed to send netlink control request")]
    NetlinkRequest(#[source] netlink_proto::Error<NetlinkControlMessage>),

    #[error("WireGuard netlink interface unavailable. Is the kernel module loaded?")]
    WireguardNetlinkInterfaceUnavailable,

    #[error("Unknown WireGuard command: {0}")]
    UnknownWireguardCommand(u8),

    #[error("Received no response")]
    NoResponse,

    #[error("Received truncated message")]
    Truncated,

    #[error("WireGuard device does not exist")]
    NoDevice,

    #[error("Failed to get config: {0}")]
    WgGetConf(netlink_packet_core::ErrorMessage),

    #[error("Failed to apply config: {0}")]
    WgSetConf(netlink_packet_core::ErrorMessage),

    #[error("Interface name too long")]
    InterfaceName,

    #[error("Send request error")]
    SendRequest(#[source] NetlinkError<DeviceMessage>),

    #[error("Create device error")]
    NetlinkCreateDevice(#[source] rtnetlink::Error),

    #[error("Add IP to device error")]
    NetlinkSetIp(rtnetlink::Error),

    #[error("Failed to delete device")]
    DeleteDevice(#[source] rtnetlink::Error),

    #[error("NetworkManager error")]
    NetworkManager(#[source] nm_tunnel::Error),
}

#[derive(Debug)]
pub struct Handle {
    pub wg_handle: WireguardConnection,
    route_handle: rtnetlink::Handle,
    wg_abort_handle: AbortHandle,
    route_abort_handle: AbortHandle,
}

impl Handle {
    pub async fn connect() -> Result<Self, Error> {
        let message_type = Self::get_wireguard_message_type().await?;
        let (conn, wireguard_connection, _messages) =
            netlink_proto::new_connection(NETLINK_GENERIC).map_err(Error::NetlinkSocket)?;
        let wg_handle = WireguardConnection {
            message_type,
            connection: wireguard_connection,
        };
        let (abortable_connection, wg_abort_handle) = abortable(conn);
        tokio::spawn(abortable_connection);
        let (conn, route_handle, _messages) =
            rtnetlink::new_connection().map_err(Error::NetlinkSocket)?;
        let (abortable_connection, route_abort_handle) = abortable(conn);
        tokio::spawn(abortable_connection);

        Ok(Self {
            wg_handle,
            route_handle,
            wg_abort_handle,
            route_abort_handle,
        })
    }

    async fn get_wireguard_message_type() -> Result<u16, Error> {
        let (conn, handle, _messages) =
            netlink_proto::new_connection(NETLINK_GENERIC).map_err(Error::NetlinkSocket)?;
        let (conn, abort_handle) = abortable(conn);
        tokio::spawn(conn);

        let result = async move {
            let mut message: NetlinkMessage<NetlinkControlMessage> =
                NetlinkControlMessage::get_netlink_family_id(c"wireguard".to_owned())
                    .map_err(Error::NetlinkControlMessage)?
                    .into();

            message.header.flags = NLM_F_REQUEST | NLM_F_ACK;

            let mut req = handle
                .request(message, SocketAddr::new(0, 0))
                .map_err(Error::NetlinkRequest)?;
            let response = req.next().await;
            if let Some(response) = response
                && let NetlinkPayload::InnerMessage(msg) = response.payload
            {
                for nla in msg.nlas.into_iter() {
                    if let ControlNla::FamilyId(id) = nla {
                        return Ok(id);
                    }
                }
            }

            Err(Error::WireguardNetlinkInterfaceUnavailable)
        }
        .await;

        abort_handle.abort();
        result
    }

    // create a wireguard device with the given name.
    pub async fn create_device(&mut self, name: String, mtu: u32) -> Result<u32, Error> {
        let message_builder = LinkMessageBuilder::<LinkWireguard>::new(&name)
            // set link to be up
            .up() // IFF_UP
            // set link MTU
            .mtu(mtu);
        let message = message_builder.build();

        let reply = self
            .route_handle
            .link()
            .add(message)
            .set_flags(NLM_F_REQUEST | NLM_F_ACK | NLM_F_REPLACE | NLM_F_CREATE | NLM_F_MATCH)
            .execute()
            .await;

        if let Err(rtnetlink::Error::NetlinkError(err)) = reply
            && -err.raw_code() != libc::EEXIST
        {
            return Err(Error::NetlinkCreateDevice(rtnetlink::Error::NetlinkError(
                err,
            )));
        };

        // fetch interface index of new device
        self.wg_handle
            .get_by_name(name)
            .await?
            .nlas
            .into_iter()
            .find_map(|nla| match nla {
                DeviceNla::IfIndex(index) => Some(index),
                _ => None,
            })
            .ok_or(Error::NoDevice)
    }

    pub async fn set_ip_address(&mut self, index: u32, addr: IpAddr) -> Result<(), Error> {
        let address_message = add_ip_addr_message(index, addr);
        let mut request = NetlinkMessage::from(RouteNetlinkMessage::NewAddress(address_message));
        request.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_REPLACE;

        let mut response = self
            .route_handle
            .request(request)
            .map_err(Error::NetlinkSetIp)?;
        while let Some(response_message) = response.next().await {
            consume_netlink_error(response_message, Error::NetlinkSetIp)?;
        }

        Ok(())
    }

    pub async fn delete_device(&mut self, index: u32) -> Result<(), Error> {
        let mut link_message = LinkMessage::default();
        link_message.header.index = index;

        let mut request = NetlinkMessage::from(RouteNetlinkMessage::DelLink(link_message));
        request.header.flags = NLM_F_REQUEST | NLM_F_ACK;

        let mut response = self
            .route_handle
            .request(request)
            .map_err(Error::DeleteDevice)?;
        while let Some(message) = response.next().await {
            consume_netlink_error(message, Error::DeleteDevice)?;
        }

        Ok(())
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.wg_abort_handle.abort();
        self.route_abort_handle.abort();
    }
}

#[derive(Debug, Clone)]
pub struct WireguardConnection {
    connection: ConnectionHandle<DeviceMessage>,
    message_type: u16,
}

impl WireguardConnection {
    pub async fn get_by_name(&mut self, name: String) -> Result<DeviceMessage, Error> {
        self.fetch_device(DeviceMessage::get_by_name(self.message_type, name)?)
            .await
    }

    pub async fn get_by_index(&mut self, index: u32) -> Result<DeviceMessage, Error> {
        self.fetch_device(DeviceMessage::get_by_index(self.message_type, index))
            .await
    }

    pub async fn fetch_device(
        &mut self,
        device_message: DeviceMessage,
    ) -> Result<DeviceMessage, Error> {
        let mut netlink_message = NetlinkMessage::from(device_message);
        netlink_message.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_DUMP;

        let mut response = self
            .connection
            .request(netlink_message, SocketAddr::new(0, 0))
            .map_err(Error::SendRequest)?;
        match response.next().await {
            Some(received_message) => match received_message.payload {
                NetlinkPayload::InnerMessage(inner) => Ok(inner),
                NetlinkPayload::Error(err) => {
                    if err.raw_code() == -libc::ENODEV {
                        Err(Error::NoDevice)
                    } else {
                        Err(Error::WgGetConf(err))
                    }
                }
                anything_else => {
                    log::error!("Received unexpected response: {:?}", anything_else);
                    Err(Error::NoResponse)
                }
            },
            None => Err(Error::NoResponse),
        }
    }

    pub async fn set_config(&mut self, interface_index: u32, config: &Config) -> Result<(), Error> {
        let message = DeviceMessage::reset_config(self.message_type, interface_index, config);
        let mut netlink_message = NetlinkMessage::from(message);
        netlink_message.header.flags = NLM_F_REQUEST | NLM_F_ACK;

        let mut request = self
            .connection
            .request(netlink_message, SocketAddr::new(0, 0))
            .map_err(Error::SendRequest)?;

        while let Some(response) = request.next().await {
            if let NetlinkPayload::Error(err) = response.payload {
                return Err(Error::WgSetConf(err));
            }
        }
        Ok(())
    }
}

fn consume_netlink_error<
    I: NetlinkDeserializable + Clone + Eq + std::fmt::Debug,
    F: Fn(rtnetlink::Error) -> Error,
>(
    message: NetlinkMessage<I>,
    err_constructor: F,
) -> Result<(), Error> {
    if let NetlinkPayload::Error(err) = message.payload {
        return Err(err_constructor(rtnetlink::Error::NetlinkError(err)));
    }
    Ok(())
}

// the built-in support for adding addresses is too helpful, so a simple AddressMessage with a
// single Address nla is created
fn add_ip_addr_message(if_index: u32, addr: IpAddr) -> AddressMessage {
    // Note: Default scope is RT_SCOPE_UNIVERSE;
    match addr {
        IpAddr::V4(ipv4_addr) => AddressMessageBuilder::<Ipv4Addr>::new()
            .address(ipv4_addr, 32)
            .index(if_index)
            .build(),
        IpAddr::V6(ipv6_addr) => AddressMessageBuilder::<Ipv6Addr>::new()
            .address(ipv6_addr, 128)
            .index(if_index)
            .build(),
    }
}

use super::{stats::Stats, Config, Tunnel, TunnelError};
use futures::future::{abortable, AbortHandle};
use netlink_packet_core::{constants::*, NetlinkDeserializable};
use netlink_packet_route::{
    rtnl::{
        address::nlas::Nla as AddressNla,
        link::nlas::{Info, InfoKind, Nla as LinkNla},
        AddressMessage, LinkMessage, RtnlMessage, RT_SCOPE_UNIVERSE,
    },
    NetlinkMessage, NetlinkPayload,
};
use netlink_packet_utils::DecodeError;
use netlink_proto::{
    sys::{Protocol, SocketAddr},
    ConnectionHandle, Error as NetlinkError,
};
use std::{ffi::CString, net::IpAddr};
use tokio::stream::StreamExt;

mod parsers;

pub mod wg_message;
use wg_message::{DeviceMessage, DeviceNla, PeerNla};
mod nl_message;
use nl_message::{ControlNla, NetlinkControlMessage};


#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to decode netlink message")]
    DecodeError(#[error(source)] DecodeError),

    #[error(display = "Failed to execute netlink control request")]
    NetlinkControlMessageError(#[error(source)] nl_message::Error),

    #[error(display = "Failed to open netlink socket")]
    NetlinkSocketError(#[error(source)] std::io::Error),

    #[error(display = "Failed to send netlink control request")]
    NetlinkRequestError(#[error(source)] netlink_proto::Error<NetlinkControlMessage>),

    #[error(display = "WireGuard netlink interface unavailable. Is the kernel module loaded?")]
    WireguardNetlinkInterfaceUnavailable,

    #[error(display = "Unknown WireGuard command _0")]
    UnnkownWireguardCommmand(u8),

    #[error(display = "Received no response")]
    NoResponse,

    #[error(display = "Received truncated message")]
    Truncated,

    #[error(display = "WireGuard device does not exist")]
    NoDevice,

    #[error(display = "Failed to get config: _0")]
    WgGetConfError(netlink_packet_core::error::ErrorMessage),

    #[error(display = "Failed to apply config: _0")]
    WgSetConfError(netlink_packet_core::error::ErrorMessage),

    #[error(display = "Interface name too long")]
    InterfaceNameError,

    #[error(display = "Send request error")]
    SendRequestError(#[error(source)] NetlinkError<DeviceMessage>),

    #[error(display = "Create device error")]
    NetlinkCreateDeviceError(#[error(source)] rtnetlink::Error),

    #[error(display = "Add IP to device error")]
    NetlinkSetIpError(rtnetlink::Error),

    #[error(display = "Failed to delete device")]
    DeleteDeviceError(#[error(source)] rtnetlink::Error),
}

pub struct KernelTunnel {
    interface_index: u32,
    netlink_connections: Handle,
    tokio_handle: tokio::runtime::Handle,
}

const MULLVAD_INTERFACE_NAME: &str = "wg-mullvad";

impl KernelTunnel {
    pub fn new(tokio_handle: tokio::runtime::Handle, config: &Config) -> Result<Self, Error> {
        tokio_handle.clone().block_on(async {
            let mut netlink_connections = Handle::connect().await?;
            let interface_index = netlink_connections
                .create_device(MULLVAD_INTERFACE_NAME.to_string(), config.mtu as u32)
                .await?;

            let mut tunnel = Self {
                interface_index,
                netlink_connections,
                tokio_handle,
            };

            if let Err(err) = tunnel.setup(config).await {
                if let Err(teardown_err) = tunnel
                    .netlink_connections
                    .delete_device(interface_index)
                    .await
                {
                    log::error!(
                        "Failed to tear down WireGuard interface after failing to apply config: {}",
                        teardown_err
                    );
                }
                return Err(err);
            }


            Ok(tunnel)
        })
    }

    async fn setup(&mut self, config: &Config) -> Result<(), Error> {
        self.netlink_connections
            .wg_handle
            .set_config(self.interface_index, config)
            .await?;

        for tunnel_ip in config.tunnel.addresses.iter() {
            self.netlink_connections
                .set_ip_address(self.interface_index, *tunnel_ip)
                .await?;
        }

        Ok(())
    }
}

impl Tunnel for KernelTunnel {
    fn get_interface_name(&self) -> String {
        let mut wg = self.netlink_connections.wg_handle.clone();
        let result = self.tokio_handle.block_on(async move {
            let device = wg.get_by_index(self.interface_index).await?;
            for nla in device.nlas {
                if let DeviceNla::IfName(name) = nla {
                    return Ok(name);
                }
            }
            return Err(Error::Truncated);
        });

        match result {
            Ok(name) => name.to_string_lossy().to_string(),
            Err(err) => {
                log::error!("Failed to deduce interface name at runtime, will attempt to use the default name. {}", err);
                MULLVAD_INTERFACE_NAME.to_string()
            }
        }
    }

    fn stop(self: Box<Self>) -> std::result::Result<(), TunnelError> {
        let Self {
            mut netlink_connections,
            interface_index,
            tokio_handle,
        } = *self;
        tokio_handle.block_on(async move {
            if let Err(err) = netlink_connections.delete_device(interface_index).await {
                log::error!("Failed to remove WireGuard device - {}", err);
                Err(TunnelError::FatalStartWireguardError)
            } else {
                Ok(())
            }
        })
    }

    fn get_tunnel_stats(&self) -> std::result::Result<Stats, TunnelError> {
        let mut wg = self.netlink_connections.wg_handle.clone();
        let interface_index = self.interface_index;
        let result = self.tokio_handle.block_on(async move {
            let device = wg.get_by_index(interface_index).await.map_err(|err| {
                log::error!("Failed to fetch WireGuard device config: {}", err);
                TunnelError::GetConfigError
            })?;

            // iterate over device attributes
            let mut tx_bytes = 0;
            let mut rx_bytes = 0;
            for nla in device.nlas {
                if let DeviceNla::Peers(peers) = nla {
                    // iterate over all peer attributes
                    let peer_iter = peers.iter().map(|peer| peer.0.as_slice()).flatten();

                    for peer_nla in peer_iter {
                        match peer_nla {
                            PeerNla::TxBytes(bytes) => tx_bytes += *bytes,
                            PeerNla::RxBytes(bytes) => rx_bytes += *bytes,
                            _ => continue,
                        };
                    }
                }
            }

            Ok(Stats { tx_bytes, rx_bytes })
        });

        result
    }
}


#[derive(Debug)]
pub struct Handle {
    pub wg_handle: WireguardConnection,
    route_handle: rtnetlink::Handle,
    wg_abort_handle: AbortHandle,
    route_abort_handle: AbortHandle,
    message_type: u16,
}


impl Handle {
    pub async fn connect() -> Result<Self, Error> {
        let message_type = Self::get_wireguard_message_type().await?;
        let (conn, wireguard_connection, _messages) =
            netlink_proto::new_connection(Protocol::Generic).map_err(Error::NetlinkSocketError)?;
        let wg_handle = WireguardConnection {
            message_type,
            connection: wireguard_connection,
        };
        let (abortable_connection, wg_abort_handle) = abortable(conn);
        tokio::spawn(abortable_connection);
        let (conn, route_handle, _messages) =
            rtnetlink::new_connection().map_err(Error::NetlinkSocketError)?;
        let (abortable_connection, route_abort_handle) = abortable(conn);
        tokio::spawn(abortable_connection);


        Ok(Self {
            wg_handle,
            route_handle,
            message_type,
            wg_abort_handle,
            route_abort_handle,
        })
    }

    async fn get_wireguard_message_type() -> Result<u16, Error> {
        let (conn, mut handle, _messages) =
            netlink_proto::new_connection(Protocol::Generic).map_err(Error::NetlinkSocketError)?;
        let (conn, abort_handle) = abortable(conn);
        tokio::spawn(conn);

        let result = async move {
            let mut message: NetlinkMessage<NetlinkControlMessage> =
                NetlinkControlMessage::get_netlink_family_id(CString::new("wireguard").unwrap())
                    .map_err(Error::NetlinkControlMessageError)?
                    .into();

            message.header.flags = NLM_F_REQUEST | NLM_F_ACK;

            let mut req = handle
                .request(message, SocketAddr::new(0, 0))
                .map_err(Error::NetlinkRequestError)?;
            let response = req.next().await;
            if let Some(response) = response {
                if let NetlinkPayload::InnerMessage(msg) = response.payload {
                    for nla in msg.nlas.into_iter() {
                        if let ControlNla::FamilyId(id) = nla {
                            return Ok(id);
                        }
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
        let mut message = LinkMessage::default();

        // set link to be up
        message.header.flags = netlink_packet_route::IFF_UP;
        // message.header.change_mask = netlink_packet_route::IFF_UP;
        // set link name
        message.nlas.push(LinkNla::IfName(name.clone()));
        // set link MTU
        message.nlas.push(LinkNla::Mtu(mtu));
        // set link type
        message
            .nlas
            .push(LinkNla::Info(vec![Info::Kind(InfoKind::Other(
                "wireguard".to_string(),
            ))]));

        let mut add_request = NetlinkMessage::from(RtnlMessage::NewLink(message));
        add_request.header.flags =
            NLM_F_REQUEST | NLM_F_ACK | NLM_F_REPLACE | NLM_F_CREATE | NLM_F_MATCH;
        let mut response = self
            .route_handle
            .request(add_request)
            .map_err(Error::NetlinkCreateDeviceError)?;
        while let Some(response_message) = response.next().await {
            if let NetlinkPayload::Error(err) = response_message.payload {
                // if the device exists, verify that it's a wireguard device
                if -err.code != libc::EEXIST {
                    return Err(Error::NetlinkCreateDeviceError(
                        rtnetlink::Error::NetlinkError(err),
                    ));
                }
            }
        }

        // fetch interface index of new device
        let new_device = self.wg_handle.get_by_name(name).await?;
        for nla in new_device.nlas {
            if let DeviceNla::IfIndex(index) = nla {
                return Ok(index);
            }
        }


        Err(Error::NoDevice)
    }

    pub async fn set_ip_address(&mut self, index: u32, addr: IpAddr) -> Result<(), Error> {
        let address_message = add_ip_addr_message(index, addr);
        let mut request = NetlinkMessage::from(RtnlMessage::NewAddress(address_message));
        request.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_REPLACE;


        let mut response = self
            .route_handle
            .request(request)
            .map_err(Error::NetlinkSetIpError)?;
        while let Some(response_message) = response.next().await {
            consume_netlink_error(response_message, Error::NetlinkSetIpError)?;
        }

        Ok(())
    }

    pub async fn delete_device(&mut self, index: u32) -> Result<(), Error> {
        let mut link_message = LinkMessage::default();
        link_message.header.index = index;

        let mut request = NetlinkMessage::from(RtnlMessage::DelLink(link_message));
        request.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_EXCL | NLM_F_CREATE;

        let mut response = self
            .route_handle
            .request(request)
            .map_err(Error::DeleteDeviceError)?;
        while let Some(message) = response.next().await {
            consume_netlink_error(message, Error::DeleteDeviceError)?;
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
            .map_err(Error::SendRequestError)?;
        match response.next().await {
            Some(received_message) => match received_message.payload {
                NetlinkPayload::InnerMessage(inner) => Ok(inner),
                NetlinkPayload::Error(err) => {
                    if err.code == -libc::ENODEV {
                        Err(Error::NoDevice)
                    } else {
                        Err(Error::WgGetConfError(err))
                    }
                }
                anything_else => {
                    log::error!("Received unexpected response - {:?}", anything_else);
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
            .map_err(Error::SendRequestError)?;

        while let Some(response) = request.next().await {
            if let NetlinkPayload::Error(err) = response.payload {
                return Err(Error::WgSetConfError(err));
            }
        }
        Ok(())
    }
}


fn consume_netlink_error<
    T,
    I: NetlinkDeserializable<T> + Clone + Eq + std::fmt::Debug,
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
    let prefix_len = if addr.is_ipv4() { 32 } else { 128 };
    let mut message = AddressMessage::default();
    message.header.prefix_len = prefix_len;
    message.header.index = if_index;
    message.header.scope = RT_SCOPE_UNIVERSE;

    match addr {
        IpAddr::V4(ipv4) => {
            message.header.family = libc::AF_INET as u8;
            let ip_bytes = ipv4.octets().to_vec();

            message.nlas.push(AddressNla::Address(ip_bytes.clone()));
            message.nlas.push(AddressNla::Local(ip_bytes));
        }
        IpAddr::V6(ipv6) => {
            message.header.family = libc::AF_INET6 as u8;
            message
                .nlas
                .push(AddressNla::Address(ipv6.octets().to_vec()));
        }
    };

    message
}

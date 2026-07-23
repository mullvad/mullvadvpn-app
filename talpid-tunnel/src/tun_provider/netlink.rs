use std::io;
use std::net::Ipv6Addr;

use netlink_packet_core::{
    NLM_F_ACK, NLM_F_CREATE, NLM_F_REPLACE, NLM_F_REQUEST, NetlinkMessage, NetlinkPayload,
};
use netlink_packet_route::{RouteNetlinkMessage, address::AddressHeaderFlags};
use netlink_sys::{Socket, SocketAddr, protocols::NETLINK_ROUTE};
use rtnetlink::AddressMessageBuilder;

/// Add IPv6 `addr` to the interface with index `if_index`.
// TODO: Upstream this to `tun`.
pub fn add_ipv6_address(if_index: u32, addr: Ipv6Addr) -> io::Result<()> {
    let mut message = AddressMessageBuilder::<Ipv6Addr>::new()
        .index(if_index)
        .address(addr, 128)
        .build();
    // Skip DAD detection.
    message.header.flags |= AddressHeaderFlags::Nodad;

    let mut request = NetlinkMessage::from(RouteNetlinkMessage::NewAddress(message));
    request.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_REPLACE;
    request.finalize();

    let mut buffer = vec![0u8; request.buffer_len()];
    request.serialize(&mut buffer);

    // Note: Not using `rtnetlink::new_connection` here because caller is sync.
    let socket = Socket::new(NETLINK_ROUTE)?;
    // Port and group zero addresses the kernel itself
    socket.send_to(&buffer, &SocketAddr::new(0, 0), 0)?;

    let (response, _) = socket.recv_from_full()?;
    let response =
        NetlinkMessage::<RouteNetlinkMessage>::deserialize(&response).map_err(io::Error::other)?;

    match response.payload {
        // An error message without a code is the acknowledgement we asked for
        NetlinkPayload::Error(err) => match err.code {
            Some(_) => Err(err.to_io()),
            None => Ok(()),
        },
        _ => Ok(()),
    }
}

use super::{
    rt_msghdr_short, AddressMessage, Error, Interface, Result, RouteMessage,
    ROUTE_MESSAGE_HEADER_SHORT_SIZE,
};

#[derive(Debug)]
pub enum RouteSocketMessage {
    AddRoute(RouteMessage),
    DeleteRoute(RouteMessage),
    ChangeRoute(RouteMessage),
    GetRoute(RouteMessage),
    Interface(Interface),
    AddAddress(AddressMessage),
    DeleteAddress(AddressMessage),
    Other {
        header: rt_msghdr_short,
        payload: Vec<u8>,
    },
    Error {
        header: rt_msghdr_short,
        payload: Vec<u8>,
    },
}

impl RouteSocketMessage {
    pub fn parse_message(buffer: &[u8]) -> Result<Self> {
        let route_message = |route_constructor: fn(RouteMessage) -> RouteSocketMessage, buffer| {
            let route = RouteMessage::from_byte_buffer(buffer)?;
            Ok(route_constructor(route))
        };

        match rt_msghdr_short::from_bytes(buffer) {
            Some(header) if header.is_type(libc::RTM_ADD) => route_message(Self::AddRoute, buffer),

            Some(header) if header.is_type(libc::RTM_CHANGE) => {
                route_message(Self::ChangeRoute, buffer)
            }

            Some(header) if header.is_type(libc::RTM_DELETE) => {
                route_message(Self::DeleteRoute, buffer)
            }

            Some(header) if header.is_type(libc::RTM_GET) => route_message(Self::GetRoute, buffer),

            Some(header) if header.is_type(libc::RTM_IFINFO) => Ok(RouteSocketMessage::Interface(
                Interface::from_byte_buffer(buffer)?,
            )),

            Some(header) if header.is_type(libc::RTM_NEWADDR) => Ok(
                RouteSocketMessage::AddAddress(AddressMessage::from_byte_buffer(buffer)?),
            ),
            Some(header) if header.is_type(libc::RTM_DELADDR) => Ok(
                RouteSocketMessage::DeleteAddress(AddressMessage::from_byte_buffer(buffer)?),
            ),
            Some(header) => Ok(Self::Other {
                header,
                payload: buffer.to_vec(),
            }),
            None => Err(Error::BufferTooSmall(
                "rt_msghdr_short",
                buffer.len(),
                ROUTE_MESSAGE_HEADER_SHORT_SIZE,
            )),
        }
    }
}

#[test]
fn test_failing_rtmsg() {
    let bytes = [
        135, 0, 5, 1, 11, 0, 0, 0, 1, 1, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 16, 2, 0, 0, 192, 168, 88, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 18, 11, 0, 6, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 255, 255, 255, 255, 255, 255,
    ];
    let _ = RouteSocketMessage::parse_message(&bytes).unwrap();
}

use super::{Error, Result};

#[derive(Debug)]
pub struct Interface {
    header: libc::if_msghdr,
}

impl Interface {
    pub fn is_up(&self) -> bool {
        self.header.ifm_flags & nix::libc::IFF_UP != 0
    }

    pub fn index(&self) -> u16 {
        self.header.ifm_index
    }

    pub(crate) fn from_byte_buffer(buffer: &[u8]) -> Result<Self> {
        const INTERFACE_MESSAGE_HEADER_SIZE: usize = std::mem::size_of::<libc::if_msghdr>();
        if INTERFACE_MESSAGE_HEADER_SIZE > buffer.len() {
            return Err(Error::BufferTooSmall(
                "if_msghdr",
                buffer.len(),
                INTERFACE_MESSAGE_HEADER_SIZE,
            ));
        }
        let header: libc::if_msghdr = unsafe { std::ptr::read(buffer.as_ptr() as *const _) };
        Ok(Self { header })
    }
}

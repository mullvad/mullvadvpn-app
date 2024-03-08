use super::{AddressFlag, Result, RouteSocketAddress};

/// An iterator to consume a byte buffer containing socket address structures originating from a
/// routing socket message.
pub struct RouteSockAddrIterator<'a> {
    buffer: &'a [u8],
    flags: AddressFlag,
    // Cursor used to iterate through address flags
    flag_cursor: i32,
}

impl<'a> RouteSockAddrIterator<'a> {
    pub(crate) fn new(buffer: &'a [u8], flags: AddressFlag) -> Self {
        Self {
            buffer,
            flags,
            flag_cursor: AddressFlag::RTA_DST.bits(),
        }
    }

    /// Advances internal byte buffer by given amount. The byte amount will be padded to be
    /// aligned to 4 bytes if there's more data in the buffer.
    fn advance_buffer(&mut self, saddr_len: u8) {
        let saddr_len = usize::from(saddr_len);

        // if consumed as many bytes as are left in the buffer, the buffer can be cleared
        if saddr_len == self.buffer.len() {
            self.buffer = &[];
            return;
        }

        let padded_saddr_len = if saddr_len % 4 != 0 {
            saddr_len + (4 - saddr_len % 4)
        } else {
            saddr_len
        };

        // if offset is larger than current buffer, ensure slice gets truncated
        // since the socket address should've already be read from the buffer at this point, this
        // probably should be an invariant?
        self.buffer = &self.buffer[padded_saddr_len..];
    }
}

impl<'a> Iterator for RouteSockAddrIterator<'a> {
    type Item = Result<RouteSocketAddress>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If address flags don't contain the current one, try the next one.
            // Will return None if it runs out of valid flags.
            let current_flag = AddressFlag::from_bits(self.flag_cursor)?;
            self.flag_cursor <<= 1;

            if !self.flags.contains(current_flag) {
                continue;
            }
            return match RouteSocketAddress::new(current_flag, self.buffer) {
                Ok((next_addr, addr_len)) => {
                    self.advance_buffer(addr_len);
                    Some(Ok(next_addr))
                }
                Err(err) => {
                    self.buffer = &[];
                    Some(Err(err))
                }
            };
        }
    }
}

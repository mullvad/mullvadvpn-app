use bytes::Bytes;
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::time::Instant;
use std::cell::RefCell;
use std::collections::VecDeque;

pub const BUF_SIZE: usize = 16096;

pub struct VirtualDevice {
    pub rx_queue: VecDeque<Bytes>,
    pub tx_queue: VecDeque<Vec<u8>>,
    buffer_pool: RefCell<VecDeque<Vec<u8>>>,
}

impl VirtualDevice {
    pub fn recycle_buffer(&mut self, buffer: Vec<u8>) {
        self.buffer_pool.borrow_mut().push_back(buffer)
    }
}

impl VirtualDevice {
    pub fn new() -> Self {
        Self {
            rx_queue: VecDeque::new(),
            tx_queue: VecDeque::new(),
            buffer_pool: RefCell::new((0..16).map(|_| Vec::with_capacity(BUF_SIZE)).collect()),
        }
    }
}

impl Device for VirtualDevice {
    type RxToken<'b>
        = RxTokenImpl
    where
        Self: 'b;

    type TxToken<'b>
        = TxTokenImpl<'b>
    where
        Self: 'b;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if let Some(buffer) = self.rx_queue.pop_front() {
            Some((
                RxTokenImpl { buffer },
                TxTokenImpl {
                    tx_queue: &mut self.tx_queue,
                    pool: &self.buffer_pool,
                },
            ))
        } else {
            None
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(TxTokenImpl {
            tx_queue: &mut self.tx_queue,
            pool: &self.buffer_pool,
        })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ip;
        caps.max_transmission_unit = 1500;
        caps
    }
}

pub struct RxTokenImpl {
    buffer: Bytes,
}

impl RxToken for RxTokenImpl {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.buffer)
    }
}

pub struct TxTokenImpl<'a> {
    tx_queue: &'a mut VecDeque<Vec<u8>>,
    pool: &'a RefCell<VecDeque<Vec<u8>>>,
}

impl TxToken for TxTokenImpl<'_> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = self
            .pool
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| Vec::with_capacity(BUF_SIZE));

        buffer.resize(len, 0);
        let result = f(&mut buffer);

        self.tx_queue.push_back(buffer);
        result
    }
}

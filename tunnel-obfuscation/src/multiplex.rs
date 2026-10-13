use std::{net::SocketAddr, sync::Arc};

use talpid_net::bypass::SocketBypass;

#[derive(Debug, Clone)]
pub struct Settings {
    pub extra_addrs: SocketAddr,
}
impl Settings {
    pub fn packet_overhead(&self) -> u16 {
        0
    }
}

pub struct Multiplex {}
impl Multiplex {
    pub fn new(bypass: Arc<dyn SocketBypass>, settings: Settings) -> Self {
        todo!()
    }
}

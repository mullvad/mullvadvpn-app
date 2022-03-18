use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub enum ObfuscatorConfig {
    Udp2Tcp { endpoint: SocketAddr },
}

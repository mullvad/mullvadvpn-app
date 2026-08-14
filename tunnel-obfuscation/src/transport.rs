//! The interface between plaintext WireGuard traffic and an obfuscation method.

use std::{io, net::SocketAddr};

use async_trait::async_trait;

/// The largest datagram that can be sent or received. Buffers passed to
/// [ObfuscatedTransport::recv] should be this large.
pub const MAX_DATAGRAM_SIZE: usize = u16::MAX as usize;

/// An obfuscation method, as seen by whoever has plaintext WireGuard datagrams to move.
///
/// [Self::send] obfuscates a datagram and transmits it to the remote; [Self::recv] receives one
/// from the remote and deobfuscates it.
///
/// # Cancel safety
///
/// Both methods must be cancel safe. A caller may drop a [Self::recv] future before it completes,
/// and an implementation that loses a datagram when that happens would drop packets silently.
///
/// However, callers must treat the buffer as clobbered once [Self::send] has been called, as
/// implementations may mutate it in place.
#[async_trait]
pub trait ObfuscatedTransport: Send + Sync {
    /// Obfuscate `packet` and send it to the remote.
    ///
    /// `packet` may be modified arbitrarily.
    async fn send(&self, packet: &mut [u8]) -> io::Result<()>;

    /// Receive a packet from the remote into `buf`, deobfuscate it, and return its length.
    ///
    /// Blocks until a packet arrives, which may be indefinitely.
    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize>;

    /// The address to report as the source of packets from [Self::recv].
    ///
    /// An obfuscation protocol delivers a datagram without saying which address it arrived from,
    /// so a caller that needs one per packet has nowhere else to take it from.
    fn endpoint(&self) -> SocketAddr;

    /// The overhead (in bytes) of this obfuscation protocol.
    fn packet_overhead(&self) -> u16;
}

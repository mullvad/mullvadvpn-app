//! Constants used throughout the relay selector

/// All the valid ports when using UDP2TCP obfuscation.
pub(crate) const UDP2TCP_PORTS: [u16; 2] = [80, 5001];

/// The standard port on which an exit relay accepts connections from an entry relay in a
/// multihop circuit.
pub(crate) const WIREGUARD_EXIT_PORT: u16 = 51820;

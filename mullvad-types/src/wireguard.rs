use serde::{Deserialize, Serialize};
use talpid_types::net::wireguard;

/// Contains account specific wireguard data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WireguardData {
    pub private_key: wireguard::PrivateKey,
    pub addresses: AssociatedAddresses,
}

/// Contains a pair of local link addresses that are paired with a specific wireguard
/// public/private keypair.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AssociatedAddresses {
    pub ipv4_address: ipnetwork::Ipv4Network,
    pub ipv6_address: ipnetwork::Ipv6Network,
}

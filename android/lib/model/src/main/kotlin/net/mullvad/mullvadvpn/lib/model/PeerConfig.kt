package net.mullvad.mullvadvpn.lib.model

import java.net.InetSocketAddress

data class PeerConfig(
    val publicKey: WireguardKey,
    val endpoint: InetSocketAddress,
    val allowedIps: Set<String>,
)

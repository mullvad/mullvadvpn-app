package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.lib.model.TransportProtocol
import net.mullvad.mullvadvpn.lib.model.TunnelEndpoint

fun TunnelEndpoint.toInAddress(): Triple<String, Int, TransportProtocol> {
    val relayEndpoint = this.obfuscation?.endpoint ?: this.endpoint
    val host = relayEndpoint.address.address.hostAddress ?: ""
    val port = relayEndpoint.address.port
    val protocol = relayEndpoint.protocol
    return Triple(host, port, protocol)
}

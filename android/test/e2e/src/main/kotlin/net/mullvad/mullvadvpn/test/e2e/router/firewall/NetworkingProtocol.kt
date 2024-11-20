package net.mullvad.mullvadvpn.test.e2e.router.firewall

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
enum class NetworkingProtocol {
    @SerialName("tcp") TCP,
    @SerialName("udp") UDP,
    @SerialName("icmp") ICMP,
}

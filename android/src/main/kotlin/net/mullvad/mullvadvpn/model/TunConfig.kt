package net.mullvad.mullvadvpn.model

import java.net.InetAddress
import net.mullvad.talpid.tun_provider.InetNetwork

data class TunConfig(
    val addresses: List<InetAddress>,
    val dnsServers: List<InetAddress>,
    val routes: List<InetNetwork>,
    val mtu: Int
)

package net.mullvad.talpid.tun_provider

import java.net.InetAddress

data class TunConfig(
    val addresses: List<InetAddress>,
    val dnsServers: List<InetAddress>,
    val routes: List<InetNetwork>,
    val mtu: Int
)

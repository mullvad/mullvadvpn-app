package net.mullvad.talpid.model

import java.net.InetAddress

data class TunConfig(
    val addresses: ArrayList<InetAddress>,
    val dnsServers: ArrayList<InetAddress>,
    val routes: ArrayList<InetNetwork>,
    val excludedPackages: ArrayList<String>,
    val mtu: Int,
)

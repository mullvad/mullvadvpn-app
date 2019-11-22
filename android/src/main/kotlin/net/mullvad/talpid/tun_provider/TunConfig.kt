package net.mullvad.talpid.tun_provider

import java.net.InetAddress
import java.util.ArrayList

data class TunConfig(
    val addresses: ArrayList<InetAddress>,
    val dnsServers: ArrayList<InetAddress>,
    val routes: ArrayList<InetNetwork>,
    val mtu: Int
)

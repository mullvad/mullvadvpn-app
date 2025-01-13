package net.mullvad.talpid.model

import android.net.RouteInfo
import java.net.InetAddress

data class NetworkState(
    val networkHandle: Long,
    // TODO Do we care about null vs empty list?
    val routes: ArrayList<RouteInfo>,
    val dnsServers: ArrayList<InetAddress>,
) {
    constructor(
        networkHandle: Long,
        routes: List<RouteInfo>,
        dnsServers: List<InetAddress>,
    ) : this(
        networkHandle = networkHandle,
        routes = ArrayList(routes),
        dnsServers = ArrayList(dnsServers),
    )
}

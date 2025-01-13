package net.mullvad.talpid.model

import java.net.InetAddress

data class NetworkState(
    val networkHandle: Long,
    val routes: ArrayList<RouteInfo>?,
    val dnsServers: ArrayList<InetAddress>?,
) {
    constructor(
        networkHandle: Long,
        routes: List<AndroidRouteInfo>?,
        dnsServers: List<InetAddress>?,
    ) : this(
        networkHandle = networkHandle,
        routes = routes?.map { it.toRoute() }?.let { ArrayList(it) },
        dnsServers = dnsServers?.let { ArrayList(it) },
    )
}

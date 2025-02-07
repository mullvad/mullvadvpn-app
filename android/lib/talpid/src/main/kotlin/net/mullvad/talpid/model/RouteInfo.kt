package net.mullvad.talpid.model

import java.net.InetAddress

typealias AndroidRouteInfo = android.net.RouteInfo

data class RouteInfo(
    val destination: InetNetwork,
    val gateway: InetAddress?,
    val interfaceName: String?,
)

fun AndroidRouteInfo.toRoute() =
    RouteInfo(
        destination = InetNetwork(destination.address, destination.prefixLength.toShort()),
        gateway = gateway,
        interfaceName = `interface`,
    )

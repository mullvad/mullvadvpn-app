package net.mullvad.mullvadvpn.model

import java.net.InetAddress

data class GeoIpLocation(
    val ipv4: InetAddress?,
    val ipv6: InetAddress?,
    val country: String,
    val city: String?,
    val latitude: Double,
    val longitude: Double,
    val hostname: String?,
)

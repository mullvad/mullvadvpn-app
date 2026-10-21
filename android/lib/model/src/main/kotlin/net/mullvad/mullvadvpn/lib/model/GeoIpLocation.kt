package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics
import java.net.InetAddress

@optics
data class GeoIpLocation(
    val ipv4: InetAddress?,
    val ipv6: InetAddress?,
    val country: String,
    val city: String?,
    val latitude: Double,
    val longitude: Double,
    val location: GeoLocationId.Hostname?,
    val entryCountry: String?,
    val entryCity: String?,
    val entryLocation: GeoLocationId.Hostname?,
) {
    companion object
}

package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics
import java.net.InetAddress

@optics
data class GeoIpLocation(
    val ipv4: InetAddress?,
    val ipv6: InetAddress?,
    val country: String,
    val countryCode: String?,
    val city: String?,
    val cityCode: String?,
    val latitude: Double,
    val longitude: Double,
    val hostname: String?,
    val entryCountry: String?,
    val entryCountryCode: String?,
    val entryCity: String?,
    val entryCityCode: String?,
    val entryHostname: String?,
) {
    companion object
}

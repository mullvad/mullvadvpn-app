package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.model.GeoIpLocation

fun GeoIpLocation?.toOutAddress(): String =
    if (this != null && (this.ipv4 != null || this.ipv6 != null)) {
        val ipv4 = this.ipv4
        val ipv6 = this.ipv6

        if (ipv6 == null) {
            ipv4?.hostAddress ?: ""
        } else if (ipv4 == null) {
            ipv6.hostAddress ?: ""
        } else {
            "${ipv4.hostAddress} / ${ipv6.hostAddress}"
        }
    } else {
        ""
    }

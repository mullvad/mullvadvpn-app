package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.model.GeoIpLocation

fun GeoIpLocation.toOutAddress(): String =
    when {
        ipv6 != null && ipv4 != null -> "${ipv4!!.hostAddress} / ${ipv6!!.hostAddress}"
        ipv6 != null -> ipv6!!.hostAddress ?: ""
        ipv4 != null -> ipv4!!.hostAddress ?: ""
        else -> ""
    }

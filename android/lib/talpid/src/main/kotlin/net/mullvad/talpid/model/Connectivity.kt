package net.mullvad.talpid.model

import android.net.LinkAddress
import java.net.Inet4Address
import java.net.Inet6Address

sealed class Connectivity {
    data class Online(val ipAvailability: IpAvailability) : Connectivity()

    data object Offline : Connectivity()

    // Required by jni
    data object PresumeOnline : Connectivity()

    companion object {
        fun fromIpAvailability(ipv4: Boolean, ipv6: Boolean) =
            when {
                ipv4 && ipv6 -> Online(IpAvailability.Ipv4AndIpv6)
                ipv4 -> Online(IpAvailability.Ipv4)
                ipv6 -> Online(IpAvailability.Ipv6)
                else -> Offline
            }

        fun fromLinkAddresses(linkAddresses: List<LinkAddress>): Connectivity {
            val ipv4 = linkAddresses.any { it.address is Inet4Address }
            val ipv6 = linkAddresses.any { it.address is Inet6Address }
            return fromIpAvailability(ipv4, ipv6)
        }
    }
}

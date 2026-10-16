package net.mullvad.talpid.model

import android.net.LinkAddress
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress

sealed interface Connectivity {
    data class Online(val ipAvailability: IpAvailability) : Connectivity

    data object Offline : Connectivity

    // Required by jni
    data object PresumeOnline : Connectivity

    companion object {
        fun fromIpAvailability(ipv4: Boolean, ipv6: Boolean) =
            when {
                ipv4 && ipv6 -> Online(IpAvailability.Ipv4AndIpv6)
                ipv4 -> Online(IpAvailability.Ipv4)
                ipv6 -> Online(IpAvailability.Ipv6)
                else -> Offline
            }

        /**
         * An IP version is only considered available if the network has an address of that version
         * that is usable for internet communication, i.e. that is globally routable. Addresses that
         * are not globally routable, such as IPv6 link-local addresses, are present on most
         * networks even when the IP version has no internet connectivity.
         */
        fun fromLinkAddresses(linkAddresses: List<LinkAddress>): Connectivity {
            val ipv4 = linkAddresses.any {
                it.address is Inet4Address && it.address.isUsableForInternet()
            }
            val ipv6 = linkAddresses.any {
                it.address is Inet6Address && it.address.isUsableForInternet()
            }
            return fromIpAvailability(ipv4, ipv6)
        }
    }
}

/**
 * Returns whether the address can be used for internet communication, i.e. if it is globally
 * routable and not reserved for other purposes.
 */
internal fun InetAddress.isUsableForInternet(): Boolean =
    when {
        isAnyLocalAddress || isLoopbackAddress || isLinkLocalAddress || isMulticastAddress -> false
        this is Inet6Address && isUniqueLocal() -> false
        else -> true
    }

/** Returns whether this is a unique local address (fc00::/7), which is not routable. */
private fun Inet6Address.isUniqueLocal(): Boolean =
    address[0].toInt() and IPV6_UNIQUE_LOCAL_PREFIX_MASK == IPV6_UNIQUE_LOCAL_PREFIX

private const val IPV6_UNIQUE_LOCAL_PREFIX = 0xfc
private const val IPV6_UNIQUE_LOCAL_PREFIX_MASK = 0xfe

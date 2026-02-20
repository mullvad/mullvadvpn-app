package net.mullvad.talpid.model

import java.net.InetAddress
import java.util.ArrayList

sealed interface CreateTunResult {
    val isOpen: Boolean

    data class Success(val tunFd: Int) : CreateTunResult {
        override val isOpen = true
    }

    sealed interface Error : CreateTunResult

    // Prepare errors
    data object OtherLegacyAlwaysOnVpn : Error {
        override val isOpen: Boolean = false
    }

    data class OtherAlwaysOnApp(val appName: String) : Error {
        override val isOpen: Boolean = false
    }

    data object NotPrepared : Error {
        override val isOpen: Boolean = false
    }

    // Establish error
    data object EstablishError : Error {
        override val isOpen: Boolean = false
    }

    data class InvalidDnsServers(val addresses: ArrayList<InetAddress>, val tunFd: Int) : Error {
        constructor(addresses: List<InetAddress>, tunFd: Int) : this(ArrayList(addresses), tunFd)

        override val isOpen = true
    }

    data class InvalidIpv6Config(
        val addresses: ArrayList<InetAddress>,
        val routes: ArrayList<InetNetwork>,
        val dnsServers: ArrayList<InetAddress>,
        val tunFd: Int,
    ) : Error {
        constructor(
            addresses: List<InetAddress>,
            routes: List<InetNetwork>,
            dnsServers: List<InetAddress>,
            tunFd: Int,
        ) : this(
            addresses = ArrayList(addresses),
            routes = ArrayList(routes),
            dnsServers = ArrayList(dnsServers),
            tunFd = tunFd,
        )

        override val isOpen = true
    }
}

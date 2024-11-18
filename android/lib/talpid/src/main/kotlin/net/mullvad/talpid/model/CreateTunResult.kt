package net.mullvad.talpid.model

import java.net.InetAddress

sealed class CreateTunResult {
    open val isOpen
        get() = false

    class Success(val tunFd: Int) : CreateTunResult() {
        override val isOpen
            get() = true
    }

    class InvalidDnsServers(val addresses: ArrayList<InetAddress>, val tunFd: Int) :
        CreateTunResult() {
        override val isOpen
            get() = true
    }

    // Establish error
    data object TunnelDeviceError : CreateTunResult()

    // Prepare errors
    data object OtherLegacyAlwaysOnVpn : CreateTunResult()

    data class OtherAlwaysOnApp(val appName: String) : CreateTunResult()

    data object NotPrepared : CreateTunResult()
}

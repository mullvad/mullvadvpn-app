package net.mullvad.talpid

import java.net.InetAddress

sealed class CreateTunResult {
    open val isOpen
        get() = false

    class Success(val tunFd: Int) : CreateTunResult() {
        override val isOpen
            get() = true
    }

    class InvalidDnsServers(
        val addresses: ArrayList<InetAddress>,
        val tunFd: Int
    ) : CreateTunResult() {
        override val isOpen
            get() = true
    }

    class PermissionDenied : CreateTunResult() {
        companion object {
            @JvmStatic
            val INSTANCE = PermissionDenied()
        }
    }

    class TunnelDeviceError : CreateTunResult() {
        companion object {
            @JvmStatic
            val INSTANCE = TunnelDeviceError()
        }
    }
}

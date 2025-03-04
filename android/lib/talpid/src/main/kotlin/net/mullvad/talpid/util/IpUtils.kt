package net.mullvad.talpid.util

import co.touchlab.kermit.Logger
import java.net.DatagramSocket
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.SocketException

object IpUtils {
    fun hasIPv4(protect: (socket: DatagramSocket) -> Boolean): Boolean =
        hasIpVersion(InetAddress.getByName(PUBLIC_IPV4_ADDRESS), protect)

    fun hasIPv6(protect: (socket: DatagramSocket) -> Boolean): Boolean =
        hasIpVersion(InetAddress.getByName(PUBLIC_IPV6_ADDRESS), protect)

    // Fake a connection to a public ip address using a UDP socket.
    // We don't care about the result of the connection, only that it is possible to create.
    // This is done this way since otherwise there is not way to check the availability of an ip
    // version on the underlying network if the VPN is turned on.
    // Since we are protecting the socket it will use the underlying network regardless
    // if the VPN is turned on or not.
    // If the ip version is not supported on the underlying network it will trigger a socket
    // exception. Otherwise we assume it is available.
    private inline fun <reified T : InetAddress> hasIpVersion(
        ip: T,
        protect: (socket: DatagramSocket) -> Boolean,
    ): Boolean {
        val socket = DatagramSocket()
        if (!protect(socket)) {
            Logger.e("Unable to protect the socket VPN is not set up correctly")
            return false
        }
        return try {
            socket.connect(InetSocketAddress(ip, 1))
            socket.localSocketAddress.also { Logger.d("Public Local address: $it") }
            true
        } catch (_: SocketException) {
            Logger.e("Socket could not be set up")
            false
        } finally {
            socket.close()
        }
    }

    private const val PUBLIC_IPV4_ADDRESS = "1.1.1.1"
    private const val PUBLIC_IPV6_ADDRESS = "2606:4700:4700::1001"
}

package net.mullvad.talpid.util

import co.touchlab.kermit.Logger
import java.net.DatagramSocket
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.SocketException

object IPAvailabilityUtils {
    fun isIPv4Available(protect: (socket: DatagramSocket) -> Unit): Boolean =
        isIPAvailable(InetAddress.getByName(PUBLIC_IPV4_ADDRESS), protect)

    fun isIPv6Available(protect: (socket: DatagramSocket) -> Unit): Boolean =
        isIPAvailable(InetAddress.getByName(PUBLIC_IPV6_ADDRESS), protect)

    // Fake a connection to a public ip address using a UDP socket.
    // Since we are protecting the socket it will use the underlying network regardless
    // if the VPN is turned on or not.
    // If the ip version is not supported on the underlying network it will trigger a socket
    // exception. If not it should return the local ip address.
    private inline fun <reified T : InetAddress> isIPAvailable(
        ip: T,
        protect: (socket: DatagramSocket) -> Unit,
    ): Boolean {
        val socket = DatagramSocket()
        return try {
            protect(socket)
            socket.connect(InetSocketAddress(ip, 1))
            socket.localAddress.hostAddress?.isNotEmpty() == true && socket.localAddress is T
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

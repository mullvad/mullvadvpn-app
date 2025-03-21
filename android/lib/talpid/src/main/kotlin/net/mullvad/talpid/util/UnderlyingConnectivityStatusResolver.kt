package net.mullvad.talpid.util

import arrow.core.Either
import arrow.core.raise.result
import co.touchlab.kermit.Logger
import java.net.DatagramSocket
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import java.net.InetSocketAddress
import net.mullvad.talpid.model.Connectivity

/** This class is used to check the ip version of the underlying network when a VPN is active. */
class UnderlyingConnectivityStatusResolver(
    private val protect: (socket: DatagramSocket) -> Boolean
) {
    fun currentStatus(): Connectivity.Status =
        Connectivity.Status(ipv4 = hasIpv4(), ipv6 = hasIpv6())

    private fun hasIpv4(): Boolean =
        hasIpVersion(Inet4Address.getByName(PUBLIC_IPV4_ADDRESS), protect)

    private fun hasIpv6(): Boolean =
        hasIpVersion(Inet6Address.getByName(PUBLIC_IPV6_ADDRESS), protect)

    // Fake a connection to a public ip address using a UDP socket.
    // We don't care about the result of the connection, only that it is possible to create.
    // This is done this way since otherwise there is not way to check the availability of an ip
    // version on the underlying network if the VPN is turned on.
    // Since we are protecting the socket it will use the underlying network regardless
    // if the VPN is turned on or not.
    // If the ip version is not supported on the underlying network it will trigger a socket
    // exception. Otherwise we assume it is available.
    private fun hasIpVersion(
        ip: InetAddress,
        protect: (socket: DatagramSocket) -> Boolean,
    ): Boolean =
        result {
                // Open socket
                val socket = openSocket().bind()

                val protected = protect(socket)

                // Protect so we can get underlying network
                if (!protected) {
                    // We shouldn't be doing this if we don't have a VPN, then we should of checked
                    // the network directly.
                    Logger.w("Failed to protect socket")
                }

                // "Connect" to public ip to see IP version is available
                val address = InetSocketAddress(ip, 1)
                socket.connectSafe(address).bind()
            }
            .isSuccess

    private fun openSocket(): Either<Throwable, DatagramSocket> =
        Either.catch { DatagramSocket() }.onLeft { Logger.e("Could not open socket or bind port") }

    private fun DatagramSocket.connectSafe(address: InetSocketAddress): Either<Throwable, Unit> =
        Either.catch { connect(address) }
            .onLeft { Logger.e("Socket could not be set up") }
            .also { close() }

    companion object {
        private const val PUBLIC_IPV4_ADDRESS = "1.1.1.1"
        private const val PUBLIC_IPV6_ADDRESS = "2606:4700:4700::1001"
    }
}

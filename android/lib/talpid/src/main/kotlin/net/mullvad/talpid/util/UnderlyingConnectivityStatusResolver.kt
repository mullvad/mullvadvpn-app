package net.mullvad.talpid.util

import arrow.core.Either
import arrow.core.getOrElse
import co.touchlab.kermit.Logger
import java.net.DatagramSocket
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import java.net.InetSocketAddress
import net.mullvad.talpid.model.Connectivity
import net.mullvad.talpid.model.isUsableForInternet

/** This class is used to check the ip version of the underlying network when a VPN is active. */
class UnderlyingConnectivityStatusResolver(
    private val protect: (socket: DatagramSocket) -> Boolean
) {
    fun currentStatus(): Connectivity =
        Connectivity.fromIpAvailability(ipv4 = hasIpv4(), ipv6 = hasIpv6())

    private fun hasIpv4(): Boolean = hasIpVersion(Inet4Address.getByName(PUBLIC_IPV4_ADDRESS))

    private fun hasIpv6(): Boolean = hasIpVersion(Inet6Address.getByName(PUBLIC_IPV6_ADDRESS))

    /**
     * Check if the version of [ip] is available on the underlying network by creating a fake
     * connection to a public ip address using a UDP socket. No packets are sent, the kernel only
     * does a route lookup and picks a local address. This is done this way since otherwise there
     * are no way to check the availability of an ip version on the underlying network if the VPN is
     * turned on.
     *
     * If the ip version is unsupported on the underlying network it will trigger a socket
     * exception, otherwise we assume it is available. If we fail for local reasons (socket can't be
     * opened or protected) the failure says nothing about the network, so we assume the ip version
     * is available.
     */
    private fun hasIpVersion(ip: InetAddress): Boolean =
        openSocket()
            .map { socket -> socket.use { it.isIpVersionAvailable(ip) } }
            .getOrElse {
                // Local failure, says nothing about the network
                Logger.w("Failed to open socket, assuming ${ip.toIpVersionString()} is available")
                true
            }

    /**
     * Checks if the ip version of [ip] is available on the underlying network. If the socket can't
     * be protected we assume it is available.
     */
    private fun DatagramSocket.isIpVersionAvailable(ip: InetAddress): Boolean {
        if (!protect(this)) {
            Logger.w("Failed to protect socket, assuming ${ip.toIpVersionString()} is available")
            return true
        }

        return connectSafe(InetSocketAddress(ip, 1))
            .map { localAddress.isUsableForInternet() }
            .getOrElse {
                Logger.w("Failed to connect to $ip, assuming it is unavailable", it)
                false
            }
    }

    private fun openSocket(): Either<Throwable, DatagramSocket> =
        Either.catch { DatagramSocket() }
            .onLeft { Logger.e("Could not open socket or bind port", it) }

    private fun DatagramSocket.connectSafe(address: InetSocketAddress): Either<Throwable, Unit> =
        Either.catch {
            connect(address)
        }

    private fun InetAddress.toIpVersionString(): String =
        when (this) {
            is Inet4Address -> "IPv4"
            is Inet6Address -> "IPv6"
            else -> "Unknown IP version of $this"
        }

    companion object {
        private const val PUBLIC_IPV4_ADDRESS = "1.1.1.1"
        private const val PUBLIC_IPV6_ADDRESS = "2606:4700:4700::1001"
    }
}

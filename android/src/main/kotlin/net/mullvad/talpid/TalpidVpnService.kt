package net.mullvad.talpid

import android.net.VpnService
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import net.mullvad.talpid.tun_provider.TunConfig

open class TalpidVpnService : VpnService() {
    val connectivityListener = ConnectivityListener()

    override fun onCreate() {
        connectivityListener.register(this)
    }

    override fun onDestroy() {
        connectivityListener.unregister(this)
    }

    fun createTun(config: TunConfig): Int {
        val builder = Builder().apply {
            for (address in config.addresses) {
                addAddress(address, prefixForAddress(address))
            }

            for (dnsServer in config.dnsServers) {
                addDnsServer(dnsServer)
            }

            for (route in config.routes) {
                addRoute(route.address, route.prefixLength.toInt())
            }

            setMtu(config.mtu)
            setBlocking(false)
        }

        val vpnInterface = builder.establish()

        return vpnInterface?.detachFd() ?: 0
    }

    fun bypass(socket: Int): Boolean {
        return protect(socket)
    }

    private fun prefixForAddress(address: InetAddress): Int {
        when (address) {
            is Inet4Address -> return 32
            is Inet6Address -> return 128
            else -> throw RuntimeException("Invalid IP address (not IPv4 nor IPv6)")
        }
    }
}

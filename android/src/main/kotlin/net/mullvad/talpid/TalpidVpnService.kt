package net.mullvad.talpid

import android.net.VpnService
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
                addAddress(address, 32)
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

        return vpnInterface.detachFd()
    }

    fun bypass(socket: Int): Boolean {
        return protect(socket)
    }
}

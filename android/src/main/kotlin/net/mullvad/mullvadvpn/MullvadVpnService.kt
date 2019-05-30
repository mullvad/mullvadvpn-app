package net.mullvad.mullvadvpn

import java.net.InetAddress

import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.CompletableDeferred

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.VpnService
import android.os.Binder
import android.os.IBinder

import net.mullvad.mullvadvpn.model.TunConfig

class MullvadVpnService : VpnService() {
    private val binder = LocalBinder()

    override fun onBind(intent: Intent): IBinder {
        return super.onBind(intent) ?: binder
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
        }

        val vpnInterface = builder.establish()

        return vpnInterface.detachFd()
    }

    fun bypass(socket: Int): Boolean {
        return protect(socket)
    }

    inner class LocalBinder : Binder() {
        val service
            get() = this@MullvadVpnService
    }
}

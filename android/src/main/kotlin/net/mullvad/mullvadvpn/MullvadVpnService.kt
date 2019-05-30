package net.mullvad.mullvadvpn

import java.net.InetAddress

import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.CompletableDeferred

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.VpnService

import net.mullvad.mullvadvpn.model.TunConfig

var INNER_VPN_SERVICE = CompletableDeferred<MullvadVpnService.InnerVpnService>()
var SERVICE_NOT_RUNNING = true

class MullvadVpnService(val context: Context) {
    class InnerVpnService : VpnService() {
        override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
            INNER_VPN_SERVICE.complete(this)

            return super.onStartCommand(intent, flags, startId)
        }

        override fun onDestroy() {
            INNER_VPN_SERVICE = CompletableDeferred<MullvadVpnService.InnerVpnService>()
            SERVICE_NOT_RUNNING = true
            super.onDestroy()
        }

        fun builder(): Builder {
            return Builder()
        }
    }

    fun createTun(config: TunConfig): Int {
        return createTun(config, startService())
    }

    fun bypass(socket: Int): Boolean {
        return startService().protect(socket)
    }

    private fun startService(): InnerVpnService {
        lateinit var service: InnerVpnService

        if (SERVICE_NOT_RUNNING) {
            SERVICE_NOT_RUNNING = false
            context.startService(Intent(context, InnerVpnService::class.java))
        }

        runBlocking { service = INNER_VPN_SERVICE.await() }

        return service
    }

    private fun createTun(config: TunConfig, service: InnerVpnService): Int {
        val builder = service.builder().apply {
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
}

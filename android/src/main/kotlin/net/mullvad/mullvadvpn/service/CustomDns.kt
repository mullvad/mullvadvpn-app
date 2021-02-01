package net.mullvad.mullvadvpn.service

import java.net.InetAddress
import java.util.ArrayList
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.service.endpoint.ServiceEndpoint

class CustomDns(private val endpoint: ServiceEndpoint) {
    private val daemon
        get() = runBlocking { endpoint.intermittentDaemon.await() }

    private var dnsServers = ArrayList<InetAddress>()
    private var enabled = false

    init {
        endpoint.settingsListener.dnsOptionsNotifier.subscribe(this) { maybeDnsOptions ->
            maybeDnsOptions?.let { dnsOptions ->
                enabled = dnsOptions.custom
                dnsServers = dnsOptions.addresses
            }
        }

        endpoint.dispatcher.apply {
            registerHandler(Request.AddCustomDnsServer::class) { request ->
                addDnsServer(request.address)
            }

            registerHandler(Request.RemoveCustomDnsServer::class) { request ->
                removeDnsServer(request.address)
            }

            registerHandler(Request.ReplaceCustomDnsServer::class) { request ->
                replaceDnsServer(request.oldAddress, request.newAddress)
            }

            registerHandler(Request.SetEnableCustomDns::class) { request ->
                if (request.enable) {
                    enable()
                } else {
                    disable()
                }
            }
        }
    }

    fun onDestroy() {
        endpoint.settingsListener.dnsOptionsNotifier.unsubscribe(this)
    }

    fun enable() {
        synchronized(this) {
            changeDnsOptions(true, dnsServers)
        }
    }

    fun disable() {
        synchronized(this) {
            changeDnsOptions(false, dnsServers)
        }
    }

    fun addDnsServer(server: InetAddress): Boolean {
        synchronized(this) {
            if (!dnsServers.contains(server)) {
                dnsServers.add(server)
                changeDnsOptions(enabled, dnsServers)

                return true
            }
        }

        return false
    }

    fun replaceDnsServer(oldServer: InetAddress, newServer: InetAddress): Boolean {
        synchronized(this) {
            if (oldServer == newServer) {
                return true
            } else if (!dnsServers.contains(newServer)) {
                val index = dnsServers.indexOf(oldServer)

                if (index >= 0) {
                    dnsServers.removeAt(index)
                    dnsServers.add(index, newServer)
                    changeDnsOptions(enabled, dnsServers)

                    return true
                }
            }
        }

        return false
    }

    fun removeDnsServer(server: InetAddress) {
        synchronized(this) {
            if (dnsServers.remove(server)) {
                changeDnsOptions(enabled, dnsServers)
            }
        }
    }

    private fun changeDnsOptions(enable: Boolean, dnsServers: ArrayList<InetAddress>) {
        val options = DnsOptions(enable, dnsServers)

        daemon.setDnsOptions(options)
    }
}

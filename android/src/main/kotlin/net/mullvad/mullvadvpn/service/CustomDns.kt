package net.mullvad.mullvadvpn.service

import java.net.InetAddress
import java.util.ArrayList
import net.mullvad.mullvadvpn.model.DnsOptions

class CustomDns(val daemon: MullvadDaemon, val settingsListener: SettingsListener) {
    private var dnsServers = ArrayList<InetAddress>()
    private var enabled = false

    init {
        settingsListener.dnsOptionsNotifier.subscribe(this) { maybeDnsOptions ->
            maybeDnsOptions?.let { dnsOptions ->
                enabled = dnsOptions.custom
                dnsServers = ArrayList(dnsOptions.addresses)
            }
        }
    }

    fun onDestroy() {
        settingsListener.dnsOptionsNotifier.unsubscribe(this)
    }

    fun addDnsServer(server: InetAddress) {
        synchronized(this) {
            if (!dnsServers.contains(server)) {
                dnsServers.add(server)
                changeDnsOptions(enabled, dnsServers)
            }
        }
    }

    fun replaceDnsServer(oldServer: InetAddress, newServer: InetAddress) {
        synchronized(this) {
            if (oldServer != newServer && !dnsServers.contains(newServer)) {
                val index = dnsServers.indexOf(oldServer)

                if (index >= 0) {
                    dnsServers.removeAt(index)
                    dnsServers.add(index, newServer)
                    changeDnsOptions(enabled, dnsServers)
                }
            }
        }
    }

    fun removeDnsServer(server: InetAddress) {
        synchronized(this) {
            if (dnsServers.remove(server)) {
                changeDnsOptions(enabled, dnsServers)
            }
        }
    }

    fun setEnabled(enable: Boolean) {
        changeDnsOptions(enable, dnsServers)
    }

    private fun changeDnsOptions(enable: Boolean, dnsServers: ArrayList<InetAddress>) {
        val options = DnsOptions(enable, dnsServers)

        daemon.setDnsOptions(options)
    }
}

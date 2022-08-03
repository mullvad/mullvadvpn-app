package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import java.net.InetAddress
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.talpid.util.EventNotifier

class CustomDns(private val connection: Messenger, private val settingsListener: SettingsListener) {
    val onEnabledChanged = EventNotifier(false)
    val onDnsServersChanged = EventNotifier<List<InetAddress>>(emptyList())

    init {
        settingsListener.dnsOptionsNotifier.subscribe(this) { maybeDnsOptions ->
            maybeDnsOptions?.let { dnsOptions ->
                synchronized(this) {
                    onEnabledChanged.notifyIfChanged(dnsOptions.custom)
                    onDnsServersChanged.notifyIfChanged(dnsOptions.addresses)
                }
            }
        }
    }

    fun enable() {
        connection.send(Request.SetEnableCustomDns(true).message)
    }

    fun disable() {
        connection.send(Request.SetEnableCustomDns(false).message)
    }

    fun isCustomDnsEnabled(): Boolean {
        return onEnabledChanged.latestEvent ?: false
    }

    fun addDnsServer(server: InetAddress): Boolean {
        val didntAlreadyHaveServer = !onDnsServersChanged.latestEvent.contains(server)

        connection.send(Request.AddCustomDnsServer(server).message)

        return didntAlreadyHaveServer
    }

    fun replaceDnsServer(oldServer: InetAddress, newServer: InetAddress): Boolean {
        synchronized(this) {
            val dnsServers = onDnsServersChanged.latestEvent
            val containsOldServer = dnsServers.contains(oldServer)
            val replacementIsValid = oldServer == newServer || !dnsServers.contains(newServer)

            connection.send(Request.ReplaceCustomDnsServer(oldServer, newServer).message)

            return containsOldServer && replacementIsValid
        }
    }

    fun removeDnsServer(server: InetAddress) {
        connection.send(Request.RemoveCustomDnsServer(server).message)
    }

    fun onDestroy() {
        onEnabledChanged.unsubscribeAll()
        onDnsServersChanged.unsubscribeAll()

        settingsListener.dnsOptionsNotifier.unsubscribe(this)
    }
}

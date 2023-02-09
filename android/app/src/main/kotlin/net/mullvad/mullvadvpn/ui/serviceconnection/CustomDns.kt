package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import java.net.InetAddress
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.DnsState
import net.mullvad.mullvadvpn.util.trySendRequest
import net.mullvad.talpid.util.EventNotifier

class CustomDns(private val connection: Messenger, private val settingsListener: SettingsListener) {
    @Deprecated(
        message = "Will soon be removed in favor of onDnsOptionsChanged.",
        replaceWith = ReplaceWith("onDnsOptionsChanged")
    )
    val onEnabledChanged = EventNotifier(false)
    @Deprecated(
        message = "Will soon be removed in favor of onDnsOptionsChanged.",
        replaceWith = ReplaceWith("onDnsOptionsChanged")
    )
    val onDnsServersChanged = EventNotifier<List<InetAddress>>(emptyList())
    val onDnsOptionsChanged = EventNotifier<DnsOptions?>(null)

    init {
        settingsListener.dnsOptionsNotifier.subscribe(this) { maybeDnsOptions ->
            maybeDnsOptions?.let { dnsOptions ->
                synchronized(this) {
                    onEnabledChanged.notifyIfChanged(dnsOptions.state == DnsState.Custom)
                    onDnsServersChanged.notifyIfChanged(dnsOptions.customOptions.addresses)
                    onDnsOptionsChanged.notifyIfChanged(dnsOptions)
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

    fun setDnsOptions(dnsOptions: DnsOptions) {
        connection.trySendRequest(Request.SetDnsOptions(dnsOptions), false)
    }

    fun onDestroy() {
        onEnabledChanged.unsubscribeAll()
        onDnsServersChanged.unsubscribeAll()
        onDnsOptionsChanged.unsubscribeAll()

        settingsListener.dnsOptionsNotifier.unsubscribe(this)
    }
}

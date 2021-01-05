package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import java.net.InetAddress
import net.mullvad.talpid.util.EventNotifier

class CustomDns(val connection: Messenger, val settingsListener: SettingsListener) {
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

    fun onDestroy() {
        onEnabledChanged.unsubscribeAll()
        onDnsServersChanged.unsubscribeAll()

        settingsListener.dnsOptionsNotifier.unsubscribe(this)
    }
}

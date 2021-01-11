package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.service.Event
import net.mullvad.mullvadvpn.service.Request
import net.mullvad.talpid.util.EventNotifier

class SettingsListener(val connection: Messenger, eventDispatcher: EventDispatcher) {
    val accountNumberNotifier = EventNotifier<String?>(null)
    val dnsOptionsNotifier = EventNotifier<DnsOptions?>(null)
    val relaySettingsNotifier = EventNotifier<RelaySettings?>(null)
    val settingsNotifier = EventNotifier<Settings?>(null)

    private var settings by settingsNotifier.notifiable()

    var account: String?
        get() = accountNumberNotifier.latestEvent
        set(value) { connection.send(Request.SetAccount(value).message) }

    init {
        eventDispatcher.registerHandler(Event.Type.SettingsUpdate) { event: Event.SettingsUpdate ->
            event.settings?.let { newSettings ->
                handleNewSettings(newSettings)
            }
        }
    }

    fun onDestroy() {
        accountNumberNotifier.unsubscribeAll()
        dnsOptionsNotifier.unsubscribeAll()
        relaySettingsNotifier.unsubscribeAll()
        settingsNotifier.unsubscribeAll()
    }

    private fun handleNewSettings(newSettings: Settings) {
        if (settings?.accountToken != newSettings.accountToken) {
            accountNumberNotifier.notify(newSettings.accountToken)
        }

        if (settings?.tunnelOptions?.dnsOptions != newSettings.tunnelOptions.dnsOptions) {
            dnsOptionsNotifier.notify(newSettings.tunnelOptions.dnsOptions)
        }

        if (settings?.relaySettings != newSettings.relaySettings) {
            relaySettingsNotifier.notify(newSettings.relaySettings)
        }

        settings = newSettings
    }
}

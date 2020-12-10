package net.mullvad.mullvadvpn.ui.serviceconnection

import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.service.Event
import net.mullvad.talpid.util.EventNotifier

class SettingsListener(val eventDispatcher: EventDispatcher) {
    val accountNumberNotifier = EventNotifier<String?>(null)
    val dnsOptionsNotifier = EventNotifier<DnsOptions?>(null)
    val relaySettingsNotifier = EventNotifier<RelaySettings?>(null)
    val settingsNotifier = EventNotifier<Settings?>(null)

    private var settings by settingsNotifier.notifiable()

    init {
        eventDispatcher.registerHandler(Event.Type.SettingsUpdate) { event: Event.SettingsUpdate ->
            event.settings?.let { newSettings ->
                handleNewSettings(newSettings)
            }
        }
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

package net.mullvad.mullvadvpn.service

import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.talpid.util.EventNotifier

class SettingsListener(val daemon: MullvadDaemon, val initialSettings: Settings) {
    val accountNumberNotifier = EventNotifier(initialSettings.accountToken)
    val dnsOptionsNotifier = EventNotifier(initialSettings.tunnelOptions.dnsOptions)
    val relaySettingsNotifier = EventNotifier(initialSettings.relaySettings)
    val settingsNotifier: EventNotifier<Settings> = EventNotifier(initialSettings)

    var settings by settingsNotifier.notifiable()
        private set

    init {
        daemon.onSettingsChange.subscribe(this) { maybeSettings ->
            maybeSettings?.let { settings -> handleNewSettings(settings) }
        }
    }

    fun onDestroy() {
        daemon.onSettingsChange.unsubscribe(this)

        accountNumberNotifier.unsubscribeAll()
        dnsOptionsNotifier.unsubscribeAll()
        relaySettingsNotifier.unsubscribeAll()
        settingsNotifier.unsubscribeAll()
    }

    fun subscribe(id: Any, listener: (Settings) -> Unit) {
        settingsNotifier.subscribe(id, listener)
    }

    fun unsubscribe(id: Any) {
        settingsNotifier.unsubscribe(id)
    }

    private fun handleNewSettings(newSettings: Settings) {
        synchronized(this) {
            if (settings.accountToken != newSettings.accountToken) {
                accountNumberNotifier.notify(newSettings.accountToken)
            }

            if (settings.tunnelOptions.dnsOptions != newSettings.tunnelOptions.dnsOptions) {
                dnsOptionsNotifier.notify(newSettings.tunnelOptions.dnsOptions)
            }

            if (settings.relaySettings != newSettings.relaySettings) {
                relaySettingsNotifier.notify(newSettings.relaySettings)
            }

            settings = newSettings
        }
    }
}

package net.mullvad.mullvadvpn.service

import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.util.Intermittent
import net.mullvad.talpid.util.EventNotifier

class SettingsListener(val initialSettings: Settings, val daemon: Intermittent<MullvadDaemon>) {
    val accountNumberNotifier = EventNotifier(initialSettings.accountToken)
    val dnsOptionsNotifier = EventNotifier(initialSettings.tunnelOptions.dnsOptions)
    val relaySettingsNotifier = EventNotifier(initialSettings.relaySettings)
    val settingsNotifier: EventNotifier<Settings> = EventNotifier(initialSettings)

    var settings by settingsNotifier.notifiable()
        private set

    init {
        daemon.registerListener(this) { newDaemon ->
            if (newDaemon != null) {
                registerListener(newDaemon)
                fetchInitialSettings(newDaemon)
            }
        }
    }

    fun onDestroy() {
        daemon.unregisterListener(this)

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

    private fun registerListener(daemon: MullvadDaemon) {
        daemon.onSettingsChange.subscribe(this) { maybeSettings ->
            synchronized(this) {
                maybeSettings?.let { newSettings -> handleNewSettings(newSettings) }
            }
        }
    }

    private fun fetchInitialSettings(daemon: MullvadDaemon) {
        synchronized(this) {
            daemon.getSettings()?.let { newSettings ->
                handleNewSettings(newSettings)
            }
        }
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

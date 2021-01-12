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
        daemon.registerListener(this) { maybeNewDaemon ->
            maybeNewDaemon?.let { newDaemon ->
                newDaemon.onSettingsChange.subscribe(this@SettingsListener) { maybeSettings ->
                    synchronized(this@SettingsListener) {
                        maybeSettings?.let { newSettings -> handleNewSettings(newSettings) }
                    }
                }

                synchronized(this@SettingsListener) {
                    newDaemon.getSettings()?.let { newSettings ->
                        handleNewSettings(newSettings)
                    }
                }
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

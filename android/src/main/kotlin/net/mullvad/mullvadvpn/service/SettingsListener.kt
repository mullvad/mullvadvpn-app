package net.mullvad.mullvadvpn.service

import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.talpid.util.EventNotifier

class SettingsListener(val daemon: MullvadDaemon) {
    var settings: Settings = daemon.getSettings()
        private set(value) {
            settingsNotifier.notify(value)
            field = value
        }

    private val settingsNotifier: EventNotifier<Settings> = EventNotifier(settings)

    private val listenerId = daemon.onSettingsChange.subscribe { maybeSettings ->
        maybeSettings?.let { settings -> handleNewSettings(settings) }
    }

    var onAccountNumberChange: ((String?) -> Unit)? = null
        set(value) {
            synchronized(this) {
                field = value
                value?.invoke(settings.accountToken)
            }
        }

    var onRelaySettingsChange: ((RelaySettings?) -> Unit)? = null
        set(value) {
            synchronized(this) {
                field = value
                value?.invoke(settings.relaySettings)
            }
        }

    fun onDestroy() {
        if (listenerId != -1) {
            daemon.onSettingsChange.unsubscribe(listenerId)
        }

        settingsNotifier.unsubscribeAll()
    }

    fun subscribe(listener: (Settings) -> Unit): Int {
        return settingsNotifier.subscribe(listener)
    }

    fun unsubscribe(id: Int) {
        settingsNotifier.unsubscribe(id)
    }

    private fun handleNewSettings(newSettings: Settings) {
        synchronized(this) {
            if (settings.accountToken != newSettings.accountToken) {
                onAccountNumberChange?.invoke(newSettings.accountToken)
            }

            if (settings.relaySettings != newSettings.relaySettings) {
                onRelaySettingsChange?.invoke(newSettings.relaySettings)
            }

            settings = newSettings
        }
    }
}

package net.mullvad.mullvadvpn.dataproxy

import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.service.MullvadDaemon

class SettingsListener(val daemon: MullvadDaemon) {
    private val listenerId = daemon.onSettingsChange.subscribe { maybeSettings ->
        maybeSettings?.let { settings -> handleNewSettings(settings) }
    }

    private var settings: Settings? = null

    var onAccountNumberChange: ((String?) -> Unit)? = null
        set(value) {
            synchronized(this) {
                field = value
                value?.invoke(settings?.accountToken)
            }
        }

    var onRelaySettingsChange: ((RelaySettings?) -> Unit)? = null
        set(value) {
            synchronized(this) {
                field = value
                value?.invoke(settings?.relaySettings)
            }
        }

    fun onDestroy() {
        if (listenerId != -1) {
            daemon.onSettingsChange.unsubscribe(listenerId)
        }
    }

    private fun handleNewSettings(newSettings: Settings) {
        synchronized(this) {
            if (settings?.accountToken != newSettings.accountToken) {
                onAccountNumberChange?.invoke(newSettings.accountToken)
            }

            if (settings?.relaySettings != newSettings.relaySettings) {
                onRelaySettingsChange?.invoke(newSettings.relaySettings)
            }

            settings = newSettings
        }
    }
}

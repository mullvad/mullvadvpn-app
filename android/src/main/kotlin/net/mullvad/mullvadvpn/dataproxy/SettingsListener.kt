package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.ui.MainActivity

class SettingsListener(val parentActivity: MainActivity) {
    private lateinit var daemon: MullvadDaemon

    private val setUpJob = setUp()

    private var listenerId = -1
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
        setUpJob.cancel()

        if (listenerId != -1) {
            daemon.onSettingsChange.unsubscribe(listenerId)
        }
    }

    private fun setUp() = GlobalScope.launch(Dispatchers.Default) {
        daemon = parentActivity.daemon.await()

        listenerId = daemon.onSettingsChange.subscribe { maybeSettings ->
            maybeSettings?.let { settings -> handleNewSettings(settings) }
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

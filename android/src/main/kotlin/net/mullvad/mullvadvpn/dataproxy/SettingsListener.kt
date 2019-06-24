package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope

import net.mullvad.mullvadvpn.MainActivity
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.MullvadDaemon

class SettingsListener(val parentActivity: MainActivity) {
    private lateinit var daemon: MullvadDaemon

    private val setUpJob = setUp()

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

        if (::daemon.isInitialized) {
            daemon.onSettingsChange = null
        }
    }

    private fun setUp() = GlobalScope.launch(Dispatchers.Default) {
        daemon = parentActivity.asyncDaemon.await()
        daemon.onSettingsChange = { settings -> handleNewSettings(settings) }
        fetchInitialSettings()
    }

    private fun fetchInitialSettings() {
        val initialSettings = daemon!!.getSettings()

        synchronized(this) {
            if (settings == null) {
                handleNewSettings(initialSettings)
            }
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

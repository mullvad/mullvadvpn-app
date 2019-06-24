package net.mullvad.mullvadvpn.dataproxy

import net.mullvad.mullvadvpn.MainActivity
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.MullvadDaemon

class SettingsListener(val parentActivity: MainActivity) {
    private var daemon: MullvadDaemon? = null
    private val setUpJob = setUp()

    private var settings: Settings? = null

    fun onDestroy() {
        setUpJob.cancel()
        daemon?.onSettingsChange = null
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
                settings = initialSettings
            }
        }
    }

    private fun handleNewSettings(newSettings: Settings) {
        synchronized(this) {
            settings = newSettings
        }
    }
}

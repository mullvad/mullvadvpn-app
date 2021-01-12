package net.mullvad.mullvadvpn.service.endpoint

import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.util.Intermittent
import net.mullvad.talpid.util.EventNotifier

class SettingsListener(val daemon: Intermittent<MullvadDaemon>) {
    val accountNumberNotifier = EventNotifier<String?>(null)
    val dnsOptionsNotifier = EventNotifier<DnsOptions?>(null)
    val relaySettingsNotifier = EventNotifier<RelaySettings?>(null)
    val settingsNotifier = EventNotifier<Settings?>(null)

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
        settingsNotifier.subscribe(id) { maybeSettings ->
            maybeSettings?.let { settings ->
                listener(settings)
            }
        }
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
}

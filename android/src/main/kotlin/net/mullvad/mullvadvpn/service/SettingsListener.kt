package net.mullvad.mullvadvpn.service

import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.talpid.util.EventNotifier

class SettingsListener {
    val accountNumberNotifier = EventNotifier<String?>(null)
    val dnsOptionsNotifier = EventNotifier<DnsOptions?>(null)
    val relaySettingsNotifier = EventNotifier<RelaySettings?>(null)
    val settingsNotifier = EventNotifier<Settings?>(null)

    var daemon by observable<MullvadDaemon?>(null) { _, oldDaemon, maybeNewDaemon ->
        oldDaemon?.onSettingsChange?.unsubscribe(this@SettingsListener)

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

    var settings by settingsNotifier.notifiable()
        private set

    fun onDestroy() {
        daemon = null

        accountNumberNotifier.unsubscribeAll()
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

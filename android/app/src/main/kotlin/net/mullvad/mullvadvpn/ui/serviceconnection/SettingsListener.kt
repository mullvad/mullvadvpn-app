package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.EventDispatcher
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.model.ObfuscationSettings
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.talpid.util.EventNotifier

class SettingsListener(private val connection: Messenger, eventDispatcher: EventDispatcher) {
    val relaySettingsNotifier = EventNotifier<RelaySettings?>(null)
    val settingsNotifier = EventNotifier<Settings?>(null)

    private var settings by settingsNotifier.notifiable()

    var allowLan: Boolean
        get() = settingsNotifier.latestEvent?.allowLan ?: false
        set(value) {
            connection.send(Request.SetAllowLan(value).message)
        }

    var autoConnect: Boolean
        get() = settingsNotifier.latestEvent?.autoConnect ?: false
        set(value) {
            connection.send(Request.SetAutoConnect(value).message)
        }

    var wireguardMtu: Int?
        get() = settingsNotifier.latestEvent?.tunnelOptions?.wireguard?.mtu
        set(value) {
            connection.send(Request.SetWireGuardMtu(value).message)
        }

    var wireguardQuantumResistant: QuantumResistantState
        get() =
            settingsNotifier.latestEvent?.tunnelOptions?.wireguard?.quantumResistant
                ?: QuantumResistantState.Off
        set(value) {
            connection.send(Request.SetWireGuardQuantumResistant(value).message)
        }

    var obfuscationSettings: ObfuscationSettings?
        get() = settingsNotifier.latestEvent?.obfuscationSettings
        set(value) {
            connection.send(Request.SetObfuscationSettings(value).message)
        }

    init {
        eventDispatcher.registerHandler(Event.SettingsUpdate::class, ::handleNewEvent)
    }

    fun onDestroy() {
        relaySettingsNotifier.unsubscribeAll()
        settingsNotifier.unsubscribeAll()
    }

    private fun handleNewEvent(event: Event.SettingsUpdate) {
        event.settings?.let { settings -> handleNewSettings(settings) }
    }

    private fun handleNewSettings(newSettings: Settings) {
        if (settings?.relaySettings != newSettings.relaySettings) {
            relaySettingsNotifier.notify(newSettings.relaySettings)
        }

        settings = newSettings
    }
}

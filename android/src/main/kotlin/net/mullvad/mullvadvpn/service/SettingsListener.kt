package net.mullvad.mullvadvpn.service

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.util.Intermittent
import net.mullvad.talpid.util.EventNotifier

class SettingsListener {
    private sealed class Command {
        class SetWireGuardMtu(val mtu: Int?) : Command()
    }

    private val availableDaemon = Intermittent<MullvadDaemon>()
    private val commandChannel = spawnActor()

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

        availableDaemon.spawnUpdate(maybeNewDaemon)
    }

    var settings by settingsNotifier.notifiable()
        private set

    var wireguardMtu: Int?
        get() = settingsNotifier.latestEvent?.tunnelOptions?.wireguard?.mtu
        set(value) = commandChannel.sendBlocking(Command.SetWireGuardMtu(value))

    fun onDestroy() {
        commandChannel.close()
        daemon = null

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

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.UNLIMITED) {
        try {
            while (true) {
                val command = channel.receive()

                when (command) {
                    is Command.SetWireGuardMtu -> {
                        availableDaemon.await().setWireguardMtu(command.mtu)
                    }
                }
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Closed sender, so stop the actor
        }
    }
}

package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.trySendBlocking
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.talpid.util.EventNotifier

class SettingsListener(endpoint: ServiceEndpoint) {
    private sealed class Command {
        class SetAllowLan(val allow: Boolean) : Command()
        class SetAutoConnect(val autoConnect: Boolean) : Command()
        class SetWireGuardMtu(val mtu: Int?) : Command()
    }

    private val commandChannel = spawnActor()
    private val daemon = endpoint.intermittentDaemon

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

        settingsNotifier.subscribe(this) { settings ->
            endpoint.sendEvent(Event.SettingsUpdate(settings))
        }

        endpoint.dispatcher.apply {
            registerHandler(Request.SetAllowLan::class) { request ->
                commandChannel.trySendBlocking(Command.SetAllowLan(request.allow))
            }

            registerHandler(Request.SetAutoConnect::class) { request ->
                commandChannel.trySendBlocking(Command.SetAutoConnect(request.autoConnect))
            }

            registerHandler(Request.SetWireGuardMtu::class) { request ->
                commandChannel.trySendBlocking(Command.SetWireGuardMtu(request.mtu))
            }
        }
    }

    fun onDestroy() {
        commandChannel.close()
        daemon.unregisterListener(this)

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
        daemon.onSettingsChange.subscribe(this, ::handleNewSettings)
    }

    private fun fetchInitialSettings(daemon: MullvadDaemon) {
        synchronized(this) {
            handleNewSettings(daemon.getSettings())
        }
    }

    private fun handleNewSettings(newSettings: Settings?) {
        if (newSettings != null) {
            synchronized(this) {
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

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.UNLIMITED) {
        try {
            for (command in channel) {
                when (command) {
                    is Command.SetAllowLan -> daemon.await().setAllowLan(command.allow)
                    is Command.SetAutoConnect -> daemon.await().setAutoConnect(command.autoConnect)
                    is Command.SetWireGuardMtu -> daemon.await().setWireguardMtu(command.mtu)
                }
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Closed sender, so stop the actor
        }
    }
}
